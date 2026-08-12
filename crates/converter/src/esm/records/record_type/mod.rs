pub mod vmad;

/// 12-byte Object Bounds (OBND subrecord)
#[derive(Debug)]
pub struct SubrecordOBND {
    pub x1: i16,
    pub y1: i16,
    pub z1: i16,
    pub x2: i16,
    pub y2: i16,
    pub z2: i16,
}

/// Represents an Alternate Texture swap entry inside MODS / DMDS / MO2S subrecords.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlternateTexture {
    pub name_3d: String,
    pub texture_form_id: u32,
    pub index_3d: u32,
}

/// Model Texture Data (MODT / DMDT / MO2T subrecord)
/// Contains internal NIF texture hash / color data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubrecordMODT {
    pub raw_data: Vec<u8>,
}

/// Represents a complete Model asset specification (MODL + optional MODT + optional MODS)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelData {
    pub model_path: String,
    pub texture_data: Option<SubrecordMODT>,
    pub alternate_textures: Vec<AlternateTexture>,
}
