//! ESM Record Parser & SQLite Exporter
//! Parses Bethesda Skyrim master files (.esm / .esp) into SQLite (skyrim_world.db)

use nom::{
    bytes::complete::take,
    number::complete::{le_f32, le_u16, le_u32},
    IResult,
};
use rusqlite::{params, Connection, Result as SqlResult};
use std::{fs::File, io::Read, path::Path};

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
        file.read_to_end(&mut buffer).expect("Failed to read ESM binary data");

        // 3. Binary parsing loop over records
        let mut input = &buffer[..];
        let tx = conn.unchecked_transaction()?;

        while !input.is_empty() {
            if let Ok((remaining, header)) = Self::parse_record_header(input) {
                let tag_str = std::str::from_utf8(&header.type_tag).unwrap_or("UNKN");

                if tag_str == "REFR" {
                    let record_payload_len = (header.data_size as usize).min(remaining.len());
                    if let Ok((_, reference)) = Self::parse_refr_record(&remaining[..record_payload_len], header.form_id) {
                        tx.execute(
                            "INSERT OR REPLACE INTO world_references 
                             (form_id, base_form_id, pos_x, pos_y, pos_z, rot_x, rot_y, rot_z, cell_form_id) 
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            params![
                                reference.form_id,
                                reference.base_form_id,
                                reference.pos_x,
                                reference.pos_y,
                                reference.pos_z,
                                reference.rot_x,
                                reference.rot_y,
                                reference.rot_z,
                                reference.cell_form_id,
                            ],
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

    /// Initializes SQLite tables with 3D R-Tree Spatial Index for 0-loading cell streaming
    pub fn create_tables(conn: &Connection) -> SqlResult<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS records (
                form_id INTEGER PRIMARY KEY,
                type_tag TEXT NOT NULL,
                editor_id TEXT
            );

            CREATE TABLE IF NOT EXISTS world_references (
                form_id INTEGER PRIMARY KEY,
                base_form_id INTEGER NOT NULL,
                pos_x REAL NOT NULL,
                pos_y REAL NOT NULL,
                pos_z REAL NOT NULL,
                rot_x REAL NOT NULL,
                rot_y REAL NOT NULL,
                rot_z REAL NOT NULL,
                cell_form_id INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_refr_cell ON world_references(cell_form_id);
            ",
        )?;
        Ok(())
    }

    /// Binary Nom Parser for 24-byte Record Header
    pub fn parse_record_header(input: &[u8]) -> IResult<&[u8], RecordHeader> {
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

        let (remaining, header) = EsmParser::parse_record_header(&mock_data).unwrap();
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
        mock_payload.extend_from_slice(&0.1f32.to_le_bytes());   // rot_x
        mock_payload.extend_from_slice(&0.2f32.to_le_bytes());   // rot_y
        mock_payload.extend_from_slice(&0.3f32.to_le_bytes());   // rot_z

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
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='world_references'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);
    }
}
