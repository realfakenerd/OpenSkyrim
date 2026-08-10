//! ESM Record Parser & SQLite Exporter
//! Parses Bethesda Skyrim master files (.esm / .esp) into SQLite (skyrim_world.db)

use color_eyre::Result;
use nom::{
    IResult,
    bytes::complete::take,
    number::complete::{le_f32, le_i32, le_u16, le_u32},
};
use rkyv::{Archive, Deserialize, Serialize, rancor::Panic, to_bytes};
use rusqlite::{Connection, Result as SqlResult, Transaction, params};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::from_utf8,
};

#[derive(Debug, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchivedSubRecord {
    pub tag: [u8; 4],
    pub data: Vec<u8>,
}

#[derive(Debug, Archive, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchivedRecordData {
    pub subrecords: Vec<ArchivedSubRecord>,
}

#[derive(Debug, Clone)]
struct RawRecord {
    pub form_id: u32,
    pub record_type: [u8; 4],
    pub flags: u32,
    pub subrecords: Vec<(Vec<u8>, Vec<u8>)>,
    pub cell_form_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GroupHeader {
    data_size: u32,
    label: u32,
    group_type: i32,
}

/// Skyrim Record Header (24 Bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordHeader {
    pub type_tag: [u8; 4], // e.g. "TES4", "CELL", "REFR", "NPC_"
    pub data_size: u32,
    pub flags: u32,
    pub form_id: u32,
    pub version_control: u32,
    pub version: u16,
    pub unknown: u16,
}

/// Parsed Placement Reference (REFR)
#[derive(Debug, Clone, PartialEq)]
pub struct WorldReference {
    pub form_id: u32,
    pub base_form_id: u32, // Points to Mesh/NPC/Item template
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub cell_form_id: u32,
}

pub struct EsmParser;

impl RawRecord {
    pub fn is_deleted(&self) -> bool {
        self.flags & 0x00000020 != 0
    }
}

impl EsmParser {
    /// Parses a Skyrim.esm file and exports world data to skyrim_world.db
    pub fn parse_and_export<P: AsRef<Path>>(esm_path: P, db_output_path: P) -> SqlResult<()> {
        println!("Opening ESM file: {:?}", esm_path.as_ref());

        // 1. Initialize SQLite Database & R-Tree Spatial Index
        let conn = Connection::open(db_output_path)?;
        Self::create_tables(&conn)?;

        // 2. Read ESM binary file
        let mut file = File::open(esm_path).expect("Failed to open ESM file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read ESM binary data");

        // 3. Binary parsing loop over records
        let mut input = &buffer[..];
        let tx = conn.unchecked_transaction()?;

        while !input.is_empty() {
            if let Ok((remaining, header)) = parse_record_header(input) {
                let tag_str = std::str::from_utf8(&header.type_tag).unwrap_or("UNKN");

                if tag_str == "REFR" {
                    let record_payload_len = (header.data_size as usize).min(remaining.len());
                    if let Ok((_, reference)) =
                        Self::parse_refr_record(&remaining[..record_payload_len], header.form_id)
                    {
                        tx.execute(
                            "INSERT OR REPLACE INTO references
                            (cell_id, form_id, pos_x, pos_y, pos_z, rot_x, rot_y, rot_z, scale, data)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1.0, NULL)",
                            params![
                                reference.cell_form_id,
                                reference.form_id,
                                reference.pos_x,
                                reference.pos_y,
                                reference.pos_z,
                                reference.rot_x,
                                reference.rot_y,
                                reference.rot_z,
                            ],
                        )?;

                        let internal_id = tx.last_insert_rowid();

                        tx.execute(
                            "INSERT OR REPLACE INTO refs_rtree
                            (id, minX, maxX, minY, maxY, minZ, maxZ)
                            VALUES (?1, ?2, ?2, ?3, ?3, ?4, ?4)",
                            params![
                                internal_id,
                                reference.pos_x,
                                reference.pos_y,
                                reference.pos_z,
                            ],
                        )?;

                        tx.execute(
                            "INSERT OR REPLACE INTO formid_map
                            (form_id, plugin_name, internal_id, record_type)
                            VALUES (?1, 'Skyrim.esm', ?2, 'REFR')",
                            params![reference.form_id, internal_id],
                        )?;
                    }
                }

                // Advance slice past record payload
                let skip_len = (header.data_size as usize).min(remaining.len());
                input = &remaining[skip_len..];
            } else {
                break;
            }
        }

        tx.commit()?;
        println!("ESM export complete.");
        Ok(())
    }

    pub fn convert_plugins(plugin_paths: &[PathBuf], db_path: &Path) -> Result<()> {
        let conn = Connection::open(db_path)?;
        Self::create_tables(&conn)?;

        let mut master: HashMap<u32, RawRecord> = HashMap::new();

        for path in plugin_paths {
            let records = parse_plugin_file(path)?;
            for rec in records {
                if rec.is_deleted() {
                    master.remove(&rec.form_id);
                } else {
                    master.insert(rec.form_id, rec);
                }
            }
        }

        export_to_db(&conn, &master)?;

        Ok(())
    }

    /// Initializes SQLite tables with 3D R-Tree Spatial Index for 0-loading cell streaming
    pub fn create_tables(conn: &Connection) -> SqlResult<()> {
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

    /// Parses REFR payload for Position and Base Form ID pointers
    pub fn parse_refr_record(input: &[u8], form_id: u32) -> IResult<&[u8], WorldReference> {
        let mut base_form_id = 0u32;
        let mut pos_x = 0.0f32;
        let mut pos_y = 0.0f32;
        let mut pos_z = 0.0f32;
        let mut rot_x = 0.0f32;
        let mut rot_y = 0.0f32;
        let mut rot_z = 0.0f32;

        let mut curr = input;
        while curr.len() >= 6 {
            let (next, sub_tag) = take(4usize)(curr)?;
            let (next, sub_len) = le_u16(next)?;
            let sub_tag_str = std::str::from_utf8(sub_tag).unwrap_or("");

            if sub_tag_str == "NAME" && sub_len == 4 {
                let (_, id) = le_u32(next)?;
                base_form_id = id;
            } else if sub_tag_str == "DATA" && sub_len >= 24 {
                let (rem, px) = le_f32(next)?;
                let (rem, py) = le_f32(rem)?;
                let (rem, pz) = le_f32(rem)?;
                let (rem, rx) = le_f32(rem)?;
                let (rem, ry) = le_f32(rem)?;
                let (_, rz) = le_f32(rem)?;
                pos_x = px;
                pos_y = py;
                pos_z = pz;
                rot_x = rx;
                rot_y = ry;
                rot_z = rz;
            }

            let advance = (sub_len as usize).min(next.len());
            curr = &next[advance..];
        }

        Ok((
            curr,
            WorldReference {
                form_id,
                base_form_id,
                pos_x,
                pos_y,
                pos_z,
                rot_x,
                rot_y,
                rot_z,
                cell_form_id: 0,
            },
        ))
    }
}

/// Binary Nom Parser for 24-byte Record Header
fn parse_record_header(input: &[u8]) -> IResult<&[u8], RecordHeader> {
    let (input, type_tag_bytes) = take(4usize)(input)?;
    let (input, data_size) = le_u32(input)?;
    let (input, flags) = le_u32(input)?;
    let (input, form_id) = le_u32(input)?;
    let (input, version_control) = le_u32(input)?;
    let (input, version) = le_u16(input)?;
    let (input, unknown) = le_u16(input)?;

    let mut type_tag = [0u8; 4];
    type_tag.copy_from_slice(type_tag_bytes);

    Ok((
        input,
        RecordHeader {
            type_tag,
            data_size,
            flags,
            form_id,
            version_control,
            version,
            unknown,
        },
    ))
}

fn parse_group_header(input: &[u8]) -> IResult<&[u8], GroupHeader> {
    let (input, _type_tag) = take(4usize)(input)?;
    let (input, data_size) = le_u32(input)?;
    let (input, label) = le_u32(input)?;
    let (input, group_type) = le_i32(input)?;

    Ok((
        input,
        GroupHeader {
            data_size,
            label,
            group_type,
        },
    ))
}

#[allow(unused_mut)]
fn parse_plugin_file(path: &Path) -> Result<Vec<RawRecord>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let data = &buffer[..];

    let mut records = Vec::new();

    let (remaining, _) = parse_record_header(data)?;
    parse_group(remaining, None, &mut records)?;

    Ok(records)
}

fn parse_group(
    input: &[u8],
    current_cell: Option<u32>,
    records: &mut Vec<RawRecord>,
) -> Result<()> {
    let mut curr = input;

    while curr.len() >= 24 {
        let peek_tag = &curr[..4];
        if peek_tag == b"GRUP" {
            let (rest, group) = parse_group_header(curr)?;
            let group_data_size = group.data_size as usize;
            let group_content = &rest[..group_data_size.min(rest.len())];

            let next_cell = match group.group_type {
                6 | 8 | 9 | 10 => Some(group.label),
                _ => current_cell,
            };

            parse_group(group_content, next_cell, records)?;

            curr = &rest[group_content.len()..];
        } else {
            let (rest, header) = parse_record_header(curr)?;
            let record_len = (header.data_size as usize).min(rest.len());
            let record_data = &rest[..record_len];
            let subrecords = extract_subrecords(record_data);
            records.push(RawRecord {
                form_id: header.form_id,
                record_type: header.type_tag,
                flags: header.flags,
                subrecords,
                cell_form_id: current_cell,
            });
            curr = &rest[record_len..];
        }
    }

    Ok(())
}

fn extract_subrecords(data: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut subs = Vec::new();
    let mut curr = data;

    while curr.len() >= 6 {
        let tag = &curr[..4];
        let len = u16::from_le_bytes([curr[4], curr[5]]) as usize;
        curr = &curr[6..];

        let payload_len = len.min(curr.len());
        let payload = curr[..payload_len].to_vec();

        subs.push((tag.to_vec(), payload));

        curr = &curr[payload_len..];
    }

    subs
}

fn export_to_db(conn: &Connection, master: &HashMap<u32, RawRecord>) -> SqlResult<()> {
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

fn extract_land_data(subs: &[(Vec<u8>, Vec<u8>)]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut vhgt = Vec::new();
    let mut vtex = Vec::new();
    let mut vclr = Vec::new();

    for (tag, data) in subs {
        match from_utf8(tag).unwrap_or("") {
            "VHGT" => vhgt = data.clone(),
            "VTEX" => vtex = data.clone(),
            "VCLR" => vclr = data.clone(),
            _ => {}
        }
    }

    (vhgt, vtex, vclr)
}

fn serialize_subrecords(subs: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let record_data = ArchivedRecordData {
        subrecords: subs
            .iter()
            .map(|(tag, data)| {
                let mut tag_arr = [0u8; 4];
                tag_arr.copy_from_slice(&tag[..4.min(tag.len())]);
                ArchivedSubRecord {
                    tag: tag_arr,
                    data: data.clone(),
                }
            })
            .collect(),
    };

    to_bytes::<Panic>(&record_data).unwrap().to_vec()
}

fn insert_reference(
    tx: &Transaction,
    form_id: u32,
    cell_id: u32,
    subs: &[(Vec<u8>, Vec<u8>)],
) -> SqlResult<()> {
    let mut pos = [0.0f32; 3];
    let mut rot = [0.0f32; 3];
    let mut scale = 1.0f32;
    let mut cell_form_id = 0;

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
            "CELL" if data.len() >= 4 => {
                cell_form_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
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

fn insert_cell(
    tx: &Transaction,
    form_id: u32,
    grid_x: Option<i32>,
    grid_y: Option<i32>,
    interior_name: Option<&str>,
) -> SqlResult<i64> {
    tx.execute(
            "INSERT OR IGNORE INTO cells (id, worldspace_id, grid_x, grid_y, interior_name, flags, data)
            VALUES (?1, 0, ?2, ?3, ?4, 0, NULL)",
            params![form_id, grid_x, grid_y, interior_name])?;

    let cell_id = tx.query_row("SELECT id FROM cells WHERE id = ?1", [form_id], |row| {
        row.get(0)
    })?;

    Ok(cell_id)
}

fn extract_cell_info(subs: &[(Vec<u8>, Vec<u8>)]) -> (Option<i32>, Option<i32>, Option<String>) {
    let mut grid_x: Option<i32> = None;
    let mut grid_y: Option<i32> = None;
    let mut interior_name: Option<String> = None;

    for (tag, data) in subs {
        match from_utf8(tag).unwrap_or("") {
            "XCLC" if data.len() >= 8 => {
                grid_x = Some(i32::from_le_bytes([data[0], data[1], data[2], data[3]]));
                grid_y = Some(i32::from_le_bytes([data[4], data[5], data[6], data[7]]));
            }
            "EDID" => {
                interior_name = Some(String::from_utf8_lossy(data).to_string());
            }
            _ => {}
        }
    }

    (grid_x, grid_y, interior_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_parse_record_header() {
        let mut mock_data = Vec::new();
        mock_data.extend_from_slice(b"TES4"); // type_tag
        mock_data.extend_from_slice(&100u32.to_le_bytes()); // data_size
        mock_data.extend_from_slice(&0u32.to_le_bytes()); // flags
        mock_data.extend_from_slice(&0x00000001u32.to_le_bytes()); // form_id
        mock_data.extend_from_slice(&0u32.to_le_bytes()); // version_control
        mock_data.extend_from_slice(&44u16.to_le_bytes()); // version
        mock_data.extend_from_slice(&0u16.to_le_bytes()); // unknown

        let (remaining, header) = parse_record_header(&mock_data).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(header.type_tag, *b"TES4");
        assert_eq!(header.data_size, 100);
        assert_eq!(header.form_id, 0x00000001);
        assert_eq!(header.version, 44);
    }

    #[test]
    fn test_parse_refr_record() {
        let mut mock_payload = Vec::new();

        // Sub-record NAME (Base Form ID)
        mock_payload.extend_from_slice(b"NAME");
        mock_payload.extend_from_slice(&4u16.to_le_bytes());
        mock_payload.extend_from_slice(&0x00012345u32.to_le_bytes());

        // Sub-record DATA (Position & Rotation: 6 x f32 = 24 bytes)
        mock_payload.extend_from_slice(b"DATA");
        mock_payload.extend_from_slice(&24u16.to_le_bytes());
        mock_payload.extend_from_slice(&100.0f32.to_le_bytes()); // pos_x
        mock_payload.extend_from_slice(&200.0f32.to_le_bytes()); // pos_y
        mock_payload.extend_from_slice(&300.0f32.to_le_bytes()); // pos_z
        mock_payload.extend_from_slice(&0.1f32.to_le_bytes()); // rot_x
        mock_payload.extend_from_slice(&0.2f32.to_le_bytes()); // rot_y
        mock_payload.extend_from_slice(&0.3f32.to_le_bytes()); // rot_z

        let (_, refr) = EsmParser::parse_refr_record(&mock_payload, 0x00099999).unwrap();
        assert_eq!(refr.form_id, 0x00099999);
        assert_eq!(refr.base_form_id, 0x00012345);
        assert_eq!(refr.pos_x, 100.0);
        assert_eq!(refr.pos_y, 200.0);
        assert_eq!(refr.pos_z, 300.0);
    }

    #[test]
    fn test_sqlite_create_tables() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(EsmParser::create_tables(&conn).is_ok());

        let table_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='references'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);
    }
}
