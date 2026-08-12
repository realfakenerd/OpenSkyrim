//! **CLFM** records contain Color (Form?) data.
//!
//! Subrecords:
//! - `EDID`: Editor ID (zstring)
//! - `FULL`: Full / Display Name (lstring / string)
//! - `CNAM`: RGB color (4 bytes: Red, Green, Blue, Unused/Alpha)
//! - `FNAM`: Color Flags (uint32: bit 0 = Playable)/!

use crate::esm::{
    extractors::SubrecordView,
    records::{EsmRecord, RawRecord},
};

/// RGBA Color Data (CNAM subrecord, 4 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubrecordCNAM {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub unused: u8,
}

/// Color Flags (FNAM subrecord, uint32)
pub mod flags {
    pub const PLAYABLE: u32 = 0x0001;
}

/// Color Form Record (CLFM)
#[derive(Debug, Clone, PartialEq)]
pub struct ClfmRecord {
    pub form_id: u32,
    pub flags: u32,
    pub editor_id: Option<String>,
    pub full_name: Option<String>,
    pub color: Option<SubrecordCNAM>,
    pub color_flags: Option<u32>,
}

impl ClfmRecord {
    /// Returns true if the playable flag is set in FNAM
    pub fn is_playable(&self) -> bool {
        self.color_flags
            .map(|f| f & flags::PLAYABLE != 0)
            .unwrap_or(false)
    }
}

impl EsmRecord for ClfmRecord {
    const RECORD_TYPE: &'static [u8; 4] = b"CLFM";

    fn parse(raw: &RawRecord) -> Option<Self> {
        if &raw.record_type != Self::RECORD_TYPE {
            return None;
        }

        let view = SubrecordView::new(&raw.subrecords);

        let editor_id = view.get_string(b"EDID");
        let full_name = view.get_string(b"FULL");
        let color = view
            .find(b"CNAM")
            .filter(|d| d.len() >= 4)
            .map(|d| SubrecordCNAM {
                red: d[0],
                green: d[1],
                blue: d[2],
                unused: d[3],
            });
        let color_flags = view.get_i32(b"FNAM").map(|v| v as u32);

        Some(Self {
            form_id: raw.form_id,
            flags: raw.flags,
            editor_id,
            full_name,
            color,
            color_flags,
        })
    }
}
