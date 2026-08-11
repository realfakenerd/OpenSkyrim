//! **TES4** is the header record for the mod file. It contains info like author, description, file type, and masters list.
//!
//! Record flags indicate the following:
//! - 0x1: if set, the file is treated as a master, regardless of what the file extension says.
//! - 0x80: whether the file has localized string tables. If this flag is not set, lstrings are treated as zstrings.
//! - 0x200: if set, the file is treated as a light master.

use crate::esm::{
    extractors::SubrecordView,
    records::{EsmRecord, RawRecord},
};

/// TES4 Record Header Flags
pub mod flags {
    pub const IS_MASTER: u32 = 0x0001;
    pub const LOCALIZED: u32 = 0x0080;
    pub const IS_LIGHT_MASTER: u32 = 0x0200;
}

/// 12-byte HEDR header Subrecord
#[derive(Debug)]
pub struct SubrecordHEDR {
    pub version: f32,
    pub num_records: u32,
    pub next_object_id: u32,
}

/// Represents a Master File dependency (MAST + DATA pair)
#[derive(Debug)]
pub struct MasterDependency {
    pub name: String,
    pub data_size: u64,
}

/// Typed representation of the TES4 File Header Record
#[derive(Debug)]
pub struct Tes4Record {
    pub flags: u32,
    pub header: SubrecordHEDR,
    pub author: Option<String>,
    pub description: Option<String>,
    pub masters: Vec<MasterDependency>,
    pub overrides: Vec<u32>,
    pub num_taggables: Option<u32>,
    pub master_count: Option<u32>,
}

impl Tes4Record {
    pub fn is_master(&self) -> bool {
        (self.flags & flags::IS_MASTER) != 0
    }

    pub fn is_localized(&self) -> bool {
        (self.flags & flags::LOCALIZED) != 0
    }

    pub fn is_light_master(&self) -> bool {
        (self.flags & flags::IS_LIGHT_MASTER) != 0
    }
}

impl EsmRecord for Tes4Record {
    const RECORD_TYPE: &'static [u8; 4] = b"TES4";

    fn parse(raw: &RawRecord) -> Option<Self> {
        if &raw.record_type != Self::RECORD_TYPE {
            return None;
        }

        let view = SubrecordView::new(&raw.subrecords);

        let hedr_raw = view.find(b"HEDR")?;
        if hedr_raw.len() < 12 {
            return None;
        }

        let header = SubrecordHEDR {
            version: f32::from_le_bytes([hedr_raw[0], hedr_raw[1], hedr_raw[2], hedr_raw[3]]),
            num_records: u32::from_le_bytes([hedr_raw[4], hedr_raw[5], hedr_raw[6], hedr_raw[7]]),
            next_object_id: u32::from_le_bytes([
                hedr_raw[8],
                hedr_raw[9],
                hedr_raw[10],
                hedr_raw[11],
            ]),
        };

        let author = view.get_string(b"CNAM");
        let description = view.get_string(b"SNAM");

        let mut masters = Vec::new();
        let mut last_mast_name: Option<String> = None;

        for (tag, data) in &raw.subrecords {
            if tag.len() >= 4 {
                match &tag[..4] {
                    b"MAST" => {
                        if let Some(name) = last_mast_name.take() {
                            masters.push(MasterDependency { name, data_size: 0 });
                        }
                        last_mast_name =
                            Some(String::from_utf8_lossy(data).trim_matches('\0').to_string());
                    }
                    b"DATA" => {
                        if let Some(name) = last_mast_name.take() {
                            let data_size = if data.len() >= 8 {
                                u64::from_le_bytes([
                                    data[0], data[1], data[2], data[3], data[4], data[5], data[6],
                                    data[7],
                                ])
                            } else {
                                0
                            };
                            masters.push(MasterDependency { name, data_size });
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(name) = last_mast_name {
            masters.push(MasterDependency { name, data_size: 0 });
        }

        let mut overrides = Vec::new();
        if let Some(onam_raw) = view.find(b"ONAM") {
            for chunk in onam_raw.chunks_exact(4) {
                overrides.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
        }

        let num_taggables = view.get_i32(b"INTV").map(|v| v as u32);
        let master_count = view.get_i32(b"INCC").map(|v| v as u32);

        Some(Self {
            flags: raw.flags,
            header,
            author,
            description,
            masters,
            overrides,
            num_taggables,
            master_count,
        })
    }
}
