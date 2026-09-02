use bevy::prelude::Resource;
use color_eyre::{Result, eyre::WrapErr};
use crossbeam_channel::{Receiver, Sender, bounded};
use std::time::Instant;
use std::{path::Path, thread};
use turso::{Builder, Connection, params};

async fn open_read_only(path: &Path) -> Result<Connection> {
    Ok(Builder::new_local(&path.to_string_lossy())
        .build()
        .await?
        .connect()?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellKey {
    Exterior {
        worldspace_id: u32,
        grid_x: i32,
        grid_y: i32,
    },
    Interior(u32),
}

#[derive(Debug, Clone)]
pub struct ReferenceRow {
    pub form_id: u32,
    pub cell_id: u32,
    pub base_form_id: u32,
    pub model_path: Option<String>,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: f32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub bounds_valid: bool,
}

#[derive(Debug, Clone)]
pub struct CellPayload {
    pub generation: u64,
    pub key: CellKey,
    pub cell_id: u32,
    pub references: Vec<ReferenceRow>,
}

#[derive(Debug)]
pub enum DatabaseRequest {
    Load {
        generation: u64,
        key: CellKey,
        queued_at: Instant,
    },
    Shutdown,
}

#[derive(Debug)]
pub struct DatabaseResponse {
    pub generation: u64,
    pub key: CellKey,
    pub result: std::result::Result<CellPayload, String>,
    pub query_micros: u64,
    pub queue_wait_micros: u64,
    pub total_request_micros: u64,
    pub row_count: usize,
}

#[derive(Resource)]
pub struct WorldDatabase {
    requests: Sender<DatabaseRequest>,
    responses: Receiver<DatabaseResponse>,
}

#[derive(Resource, Default)]
pub struct AssetCatalog {
    landscape_diffuse: std::collections::HashMap<u32, String>,
    water_flow: std::collections::HashMap<u32, String>,
}

impl AssetCatalog {
    pub fn open(path: &Path) -> Result<Self> {
        tokio::runtime::Runtime::new()?.block_on(Self::open_async(path))
    }

    async fn open_async(path: &Path) -> Result<Self> {
        let connection = open_read_only(path).await?;
        let mut rows = connection
            .query(
                "SELECT l.id,t.diffuse_path FROM landscape_textures l JOIN texture_sets t ON t.id=l.texture_set_id WHERE t.diffuse_path IS NOT NULL",
                (),
            )
            .await?;
        let mut landscape_diffuse = std::collections::HashMap::new();
        while let Some(row) = rows.next().await? {
            let (id, path) = (row.get::<u32>(0)?, row.get::<String>(1)?);
            if let Some(path) = converted_texture_path(path) {
                landscape_diffuse.insert(id, path);
            }
        }
        let mut rows = connection
            .query(
                "SELECT id,flow_normal_path FROM waters WHERE flow_normal_path IS NOT NULL AND flow_normal_path <> ''",
                (),
            )
            .await?;
        let mut water_flow = std::collections::HashMap::new();
        while let Some(row) = rows.next().await? {
            let (id, path) = (row.get::<u32>(0)?, row.get::<String>(1)?);
            if let Some(path) = converted_texture_path(path) {
                water_flow.insert(id, path);
            }
        }
        Ok(Self {
            landscape_diffuse,
            water_flow,
        })
    }

    pub fn landscape_diffuse(&self, form_id: u32) -> Option<&str> {
        self.landscape_diffuse.get(&form_id).map(String::as_str)
    }

    pub fn water_flow(&self, form_id: u32) -> Option<&str> {
        self.water_flow.get(&form_id).map(String::as_str)
    }
}

fn converted_texture_path(path: String) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let without_prefix = normalized
        .strip_prefix("textures/")
        .or_else(|| normalized.strip_prefix("Textures/"))
        .unwrap_or(&normalized);
    if without_prefix.is_empty() {
        return None;
    }
    let mut converted = std::path::PathBuf::from("textures").join(without_prefix);
    converted.set_extension("ktx2");
    Some(converted.to_string_lossy().replace('\\', "/"))
}

impl WorldDatabase {
    pub fn open(path: &Path) -> Result<Self> {
        tokio::runtime::Runtime::new()?.block_on(validate(path))?;
        let path = path.to_owned();
        let (request_tx, request_rx) = bounded(128);
        let (response_tx, response_rx) = bounded(128);
        thread::Builder::new()
            .name("openskyrim-world-db".into())
            .spawn(move || worker(path, request_rx, response_tx))
            .wrap_err("failed to start world database worker")?;
        Ok(Self {
            requests: request_tx,
            responses: response_rx,
        })
    }

    pub fn request(&self, request: DatabaseRequest) -> Result<()> {
        self.requests
            .send(request)
            .wrap_err("world database worker stopped")
    }

    pub fn try_response(&self) -> Option<DatabaseResponse> {
        self.responses.try_recv().ok()
    }
}

impl Drop for WorldDatabase {
    fn drop(&mut self) {
        let _ = self.requests.try_send(DatabaseRequest::Shutdown);
    }
}

async fn validate(path: &Path) -> Result<()> {
    let connection = open_read_only(path)
        .await
        .wrap_err_with(|| format!("failed to open {}", path.display()))?;
    let version: u32 = connection
        .query("SELECT version FROM schema_info LIMIT 1", ())
        .await
        .wrap_err("world database has no schema version")?
        .next()
        .await
        .wrap_err("world database has no schema version")?
        .ok_or_else(|| color_eyre::eyre::eyre!("world database has no schema version"))?
        .get(0)?;
    color_eyre::eyre::ensure!(
        version == shared::WORLD_DATABASE_SCHEMA_VERSION,
        "world database schema {version} is unsupported; reconvert assets for version {}",
        shared::WORLD_DATABASE_SCHEMA_VERSION
    );
    Ok(())
}

fn worker(
    path: std::path::PathBuf,
    requests: Receiver<DatabaseRequest>,
    responses: Sender<DatabaseResponse>,
) {
    let Ok(runtime) = tokio::runtime::Runtime::new() else {
        return;
    };
    let connection = match runtime.block_on(open_read_only(&path)) {
        Ok(connection) => connection,
        Err(_) => return,
    };
    while let Ok(request) = requests.recv() {
        let DatabaseRequest::Load {
            generation,
            key,
            queued_at,
        } = request
        else {
            break;
        };
        let queue_wait_micros = elapsed_micros(queued_at);
        let started = Instant::now();
        let result = runtime
            .block_on(load_cell(&connection, generation, key))
            .map_err(|error| format!("{error:#}"));
        let query_micros = elapsed_micros(started);
        let row_count = result
            .as_ref()
            .map_or(0, |payload| payload.references.len());
        if responses
            .send(DatabaseResponse {
                generation,
                key,
                result,
                query_micros,
                queue_wait_micros,
                total_request_micros: elapsed_micros(queued_at),
                row_count,
            })
            .is_err()
        {
            break;
        }
    }
}

fn elapsed_micros(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

async fn load_cell(connection: &Connection, generation: u64, key: CellKey) -> Result<CellPayload> {
    let cell_id: u32 = match key {
        CellKey::Exterior {
            worldspace_id,
            grid_x,
            grid_y,
        } => connection
            .query(
                "SELECT id FROM cells WHERE worldspace_id=?1 AND grid_x=?2 AND grid_y=?3",
                params![worldspace_id, grid_x, grid_y],
            )
            .await?
            .next()
            .await?
            .ok_or_else(|| color_eyre::eyre::eyre!("cell not found"))?
            .get(0)?,
        CellKey::Interior(cell_id) => cell_id,
    };
    let references = match key {
        CellKey::Exterior {
            worldspace_id,
            grid_x,
            grid_y,
        } => {
            let sql =
        "SELECT r.id,r.cell_id,r.base_form_id,s.model_path,r.pos_x,r.pos_y,r.pos_z,r.rot_x,r.rot_y,r.rot_z,r.scale,
                COALESCE(s.bounds_min_x,-64),COALESCE(s.bounds_min_y,-64),COALESCE(s.bounds_min_z,-64),
                COALESCE(s.bounds_max_x,64),COALESCE(s.bounds_max_y,64),COALESCE(s.bounds_max_z,64),
                COALESCE(s.bounds_valid,0)
         FROM \"references\" r LEFT JOIN statics s ON s.id=r.base_form_id
         WHERE r.is_exterior=1 AND r.worldspace_id=?1 AND r.pos_x>=?2 AND r.pos_x<?3 AND r.pos_y>=?4 AND r.pos_y<?5";
            let min_x = grid_x as f64 * 4096.0;
            let min_y = grid_y as f64 * 4096.0;
            let mut rows = connection
                .query(
                    sql,
                    params![worldspace_id, min_x, min_x + 4096.0, min_y, min_y + 4096.0],
                )
                .await?;
            let mut references = Vec::new();
            while let Some(row) = rows.next().await? {
                references.push(map_reference(&row)?);
            }
            references
        }
        CellKey::Interior(_) => {
            let sql =
        "SELECT r.id,r.cell_id,r.base_form_id,s.model_path,r.pos_x,r.pos_y,r.pos_z,r.rot_x,r.rot_y,r.rot_z,r.scale,
                COALESCE(s.bounds_min_x,-64),COALESCE(s.bounds_min_y,-64),COALESCE(s.bounds_min_z,-64),
                COALESCE(s.bounds_max_x,64),COALESCE(s.bounds_max_y,64),COALESCE(s.bounds_max_z,64),
                COALESCE(s.bounds_valid,0)
         FROM \"references\" r LEFT JOIN statics s ON s.id=r.base_form_id WHERE r.cell_id=?1";
            let mut rows = connection.query(sql, params![cell_id]).await?;
            let mut references = Vec::new();
            while let Some(row) = rows.next().await? {
                references.push(map_reference(&row)?);
            }
            references
        }
    };
    Ok(CellPayload {
        generation,
        key,
        cell_id,
        references,
    })
}

fn map_reference(row: &turso::Row) -> turso::Result<ReferenceRow> {
    Ok(ReferenceRow {
        form_id: row.get(0)?,
        cell_id: row.get(1)?,
        base_form_id: row.get(2)?,
        model_path: row.get(3)?,
        position: [
            row.get::<f64>(4)? as f32,
            row.get::<f64>(5)? as f32,
            row.get::<f64>(6)? as f32,
        ],
        rotation: [
            row.get::<f64>(7)? as f32,
            row.get::<f64>(8)? as f32,
            row.get::<f64>(9)? as f32,
        ],
        scale: row.get::<f64>(10)? as f32,
        bounds_min: [
            row.get::<f64>(11)? as f32,
            row.get::<f64>(12)? as f32,
            row.get::<f64>(13)? as f32,
        ],
        bounds_max: [
            row.get::<f64>(14)? as f32,
            row.get::<f64>(15)? as f32,
            row.get::<f64>(16)? as f32,
        ],
        bounds_valid: row.get::<i64>(17)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn open_in_memory() -> Connection {
        Builder::new_local(":memory:")
            .build()
            .await
            .unwrap()
            .connect()
            .unwrap()
    }

    async fn fixture(connection: &Connection) {
        connection
            .execute_batch(
                r#"CREATE TABLE schema_info(version INTEGER NOT NULL);
                INSERT INTO schema_info VALUES(3);
                CREATE TABLE cells(id INTEGER PRIMARY KEY,worldspace_id INTEGER,grid_x INTEGER,grid_y INTEGER);
                CREATE TABLE statics(id INTEGER PRIMARY KEY,model_path TEXT,bounds_min_x REAL,bounds_min_y REAL,bounds_min_z REAL,bounds_max_x REAL,bounds_max_y REAL,bounds_max_z REAL,bounds_valid INTEGER NOT NULL);
                CREATE TABLE "references"(id INTEGER PRIMARY KEY,cell_id INTEGER,worldspace_id INTEGER,base_form_id INTEGER,is_exterior INTEGER,pos_x REAL,pos_y REAL,pos_z REAL,rot_x REAL,rot_y REAL,rot_z REAL,scale REAL);
                INSERT INTO cells VALUES(10,60,2,-3);
                INSERT INTO statics VALUES(20,'architecture/wall.nif',-1,-2,-3,1,2,3,1);
                INSERT INTO "references" VALUES(30,10,60,20,1,8200,-12200,50,0,0,0,1);
                INSERT INTO "references" VALUES(31,99,60,20,1,8250,-12150,55,0,0,0,1);"#,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn loads_exterior_cell_through_spatial_index() {
        let connection = open_in_memory().await;
        fixture(&connection).await;
        let payload = load_cell(
            &connection,
            9,
            CellKey::Exterior {
                worldspace_id: 60,
                grid_x: 2,
                grid_y: -3,
            },
        )
        .await
        .unwrap();
        assert_eq!(payload.generation, 9);
        assert_eq!(payload.cell_id, 10);
        assert_eq!(payload.references.len(), 2);
        assert!(
            payload
                .references
                .iter()
                .any(|reference| reference.cell_id == 99)
        );
        assert_eq!(
            payload.references[0].model_path.as_deref(),
            Some("architecture/wall.nif")
        );
        assert_eq!(payload.references[0].bounds_max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn catalog_rewrites_landscape_texture_paths() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("world.db");
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let connection = Builder::new_local(&path.to_string_lossy())
                .build()
                .await
                .unwrap()
                .connect()
                .unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE texture_sets(id INTEGER PRIMARY KEY,diffuse_path TEXT); CREATE TABLE landscape_textures(id INTEGER PRIMARY KEY,texture_set_id INTEGER); CREATE TABLE waters(id INTEGER PRIMARY KEY,flow_normal_path TEXT); INSERT INTO texture_sets VALUES(2,'textures/land/grass.dds'); INSERT INTO landscape_textures VALUES(1,2); INSERT INTO waters VALUES(9,'textures/water/flow.dds');",
                )
                .await
                .unwrap();
        });
        let catalog = AssetCatalog::open(&path).unwrap();
        assert_eq!(
            catalog.landscape_diffuse(1),
            Some("textures/land/grass.ktx2")
        );
        assert_eq!(catalog.water_flow(9), Some("textures/water/flow.ktx2"));
    }

    #[tokio::test]
    async fn rejects_previous_database_schema() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("old.db");
        let connection = Builder::new_local(&path.to_string_lossy())
            .build()
            .await
            .unwrap()
            .connect()
            .unwrap();
        connection
            .execute_batch(
                "CREATE TABLE schema_info(version INTEGER); INSERT INTO schema_info VALUES(2);",
            )
            .await
            .unwrap();
        drop(connection);
        assert!(validate(&path).await.is_err());
    }
}
