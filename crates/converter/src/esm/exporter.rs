use crate::esm::{
    extractors::{extract_cell_info, extract_land_data, serialize_subrecords},
    records::RawRecord,
};
use rusqlite::{Connection, Result, Transaction, params};
use std::{collections::HashMap, str::from_utf8};

/// Initializes SQLite tables with 3D R-Tree Spatial Index for 0-loading cell streaming
pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
            -- 1. Plugins

            CREATE TABLE IF NOT EXISTS plugins (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                priority INTEGER NOT NULL,
                checksum BLOB NOT NULL
            );

            -- 2. Records
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY,
                form_id INTEGER NOT NULL,
                record_type TEXT NOT NULL,
                data BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_records_formid ON records(form_id);
            CREATE INDEX IF NOT EXISTS idx_records_type ON records(record_type);

            -- 3. Worldspaces
            CREATE TABLE IF NOT EXISTS worldspaces (
                id INTEGER PRIMARY KEY,
                editor_id TEXT NOT NULL,
                parent_world INTEGER,
                flags INTEGER NOT NULL
            );

            -- 4. Cells
            CREATE TABLE IF NOT EXISTS cells (
                id INTEGER PRIMARY KEY,
                worldspace_id INTEGER NOT NULL,
                grid_x INTEGER,
                grid_y INTEGER,
                interior_name TEXT,
                flags INTEGER NOT NULL,
                data BLOB
            );

            -- 5. References
            CREATE TABLE IF NOT EXISTS references (
                id INTEGER PRIMARY KEY,
                cell_id INTEGER NOT NULL,
                form_id INTEGER NOT NULL,
                pos_x REAL NOT NULL,
                pos_y REAL NOT NULL,
                pos_z REAL NOT NULL,
                rot_x REAL NOT NULL,
                rot_y REAL NOT NULL,
                rot_z REAL NOT NULL,
                scale REAL NOT NULL DEFAULT 1.0,
                data BLOB
            );

            -- 6. Spatial R-Tree Index (Virtual Table)
            CREATE VIRTUAL TABLE IF NOT EXISTS refs_rtree USING rtree(
                id,
                minX, maxX,
                minY, maxY,
                minZ, maxZ
            );

            -- 7. Land (Terrain)
            CREATE TABLE IF NOT EXISTS land (
                cell_id INTEGER PRIMARY KEY,
                heightmap BLOB NOT NULL,
                vtext BLOB,
                vclr BLOB
            );

            -- 8. LOD (Level of Details)
            CREATE TABLE IF NOT EXISTS lod (
                cell_id INTEGER NOT NULL,
                lod_level INTEGER NOT NULL,
                mesh_data BLOB NOT NULL,
                PRIMARY KEY (cell_id, lod_level)
            );

            -- 9. Scripts
            CREATE TABLE IF NOT EXISTS scripts (
                form_id INTEGER PRIMARY KEY,
                script_name TEXT NOT NULL,
                bytecode BLOB NOT NULL,
                properties BLOB
            );

            -- 10. FormID Map (Bridge FormID 32-bit -> Internal ID 64-bit)
            CREATE TABLE IF NOT EXISTS formid_map (
                form_id INTEGER NOT NULL,
                plugin_name TEXT NOT NULL,
                internal_id INTEGER NOT NULL,
                record_type TEXT NOT NULL,
                PRIMARY KEY (form_id, plugin_name)
            );

            CREATE TABLE IF NOT EXISTS conversion_cache (
                plugin_path TEXT PRIMARY KEY,
                file_hash BLOB NOT NULL,
                last_converted INTEGER NOT NULL
            );
            ",
    )?;
    Ok(())
}

pub fn export_to_db(conn: &Connection, master: &HashMap<u32, RawRecord>) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    let mut cell_info: HashMap<u32, (Option<i32>, Option<i32>, Option<String>)> = HashMap::new();

    for (&form_id, record) in master {
        let type_str = from_utf8(&record.record_type).unwrap_or("UNKN");

        match type_str {
            "CELL" => {
                let (grid_x, grid_y, interior_name) = extract_cell_info(&record.subrecords);
                insert_cell(&tx, form_id, grid_x, grid_y, interior_name.as_deref())?;

                cell_info.insert(form_id, (grid_x, grid_y, interior_name));
            }
            "REFR" | "ACHR" | "ACRE" | "PGRE" | "PMIS" => {
                let cell_id = record.cell_form_id.unwrap_or(0);
                insert_reference(&tx, form_id, cell_id, &record.subrecords)?;
            }
            "NPC_" | "WEAP" | "SPEL" | "QUST" | "FACT" | "DIAL" => {
                let blob = serialize_subrecords(&record.subrecords);
                tx.execute(
                    "INSERT OR REPLACE INTO records (id, form_id, record_type, data)
                        VALUES (NULL, ?1, ?2, ?3)",
                    params![form_id, type_str, blob],
                )?;

                let internal_id = tx.last_insert_rowid();
                tx.execute("
                        INSERT OR REPLACE INTO formid_map (form_id, plugin_name, internal_id, record_type)
                        VALUES (?1, 'merged', ?2, ?3)",
                        params![form_id, internal_id, type_str])?;
            }
            "LAND" => {
                let (heightmap, vtex, vclr) = extract_land_data(&record.subrecords);
                tx.execute(
                    "INSERT OR REPLACE INTO land (cell_id, heightmap, vtex, vclr)
                        VALUES (?1, ?2, ?3, ?4)",
                    params![form_id, heightmap, vtex, vclr],
                )?;
            }
            _ => {}
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn insert_reference(
    tx: &Transaction,
    form_id: u32,
    cell_id: u32,
    subs: &[(Vec<u8>, Vec<u8>)],
) -> Result<()> {
    let mut pos = [0.0f32; 3];
    let mut rot = [0.0f32; 3];
    let mut scale = 1.0f32;
    for (tag, data) in subs {
        match from_utf8(tag).unwrap_or("") {
            "DATA" if data.len() >= 4 => {
                pos[0] = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                pos[1] = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                pos[2] = f32::from_le_bytes([data[8], data[9], data[10], data[11]]);
                rot[0] = f32::from_le_bytes([data[12], data[13], data[14], data[15]]);
                rot[1] = f32::from_le_bytes([data[16], data[17], data[18], data[19]]);
                rot[2] = f32::from_le_bytes([data[20], data[21], data[22], data[23]]);
            }
            "XSCL" if data.len() >= 4 => {
                scale = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            }
            _ => {}
        }
    }

    let blob = serialize_subrecords(subs);

    tx.execute(
                "INSERT INTO references (id, cell_id, form_id, pos_x, pos_y, pos_z, rot_x, rot_y, rot_z, scale, data)
                 VALUES (NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![cell_id, form_id, pos[0], pos[1], pos[2], rot[0], rot[1], rot[2], scale, blob],
            )?;

    let internal_id = tx.last_insert_rowid();

    tx.execute(
        "INSERT INTO refs_rtree (id, minX, maxX, minY, maxY, minZ, maxZ)
                 VALUES (?1, ?2, ?2, ?3, ?3, ?4, ?4)",
        params![internal_id, pos[0], pos[1], pos[2]],
    )?;

    tx.execute(
        "INSERT OR REPLACE INTO formid_map (form_id, plugin_name, internal_id, record_type)
                 VALUES (?1, 'merged', ?2, 'REFR')",
        params![form_id, internal_id],
    )?;

    Ok(())
}

pub fn insert_cell(
    tx: &Transaction,
    form_id: u32,
    grid_x: Option<i32>,
    grid_y: Option<i32>,
    interior_name: Option<&str>,
) -> Result<i64> {
    tx.execute(
            "INSERT OR IGNORE INTO cells (id, worldspace_id, grid_x, grid_y, interior_name, flags, data)
            VALUES (?1, 0, ?2, ?3, ?4, 0, NULL)",
            params![form_id, grid_x, grid_y, interior_name])?;

    let cell_id = tx.query_row("SELECT id FROM cells WHERE id = ?1", [form_id], |row| {
        row.get(0)
    })?;

    Ok(cell_id)
}
