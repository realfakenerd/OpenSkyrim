use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct RawRecord {
    pub form_id: u32,
    pub record_type: [u8; 4],
    pub flags: u32,
    pub subrecords: Vec<(Vec<u8>, Vec<u8>)>,
    pub cell_form_id: Option<u32>,
}

impl RawRecord {
    pub fn is_deleted(&self) -> bool {
        self.flags & 0x00000020 != 0
    }
}

#[derive(Debug, Clone)]
pub struct GroupHeader {
    pub data_size: u32,
    pub label: u32,
    pub group_type: i32,
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

pub type CellInfoMap = HashMap<u32, (Option<i32>, Option<i32>, Option<String>)>;
