//! **AACT** records hold information about **Actions**. The only known use of them at the moment is as the root of IDLE record trees.
//!
//! They may be similar functionality to keywords attached to objects by the engine since some of these "idle" animations check for attacking or similar without any obvious means to actually activate such activity.
//!
//! Note: In the Skyrim and Dawnguard master files, there is an **AACT** entry which has zero size, and thus does not have an **EDID** field. Most **AACT** records do contain one, though.

use crate::esm::{
    extractors::SubrecordView,
    records::{EsmRecord, RawRecord},
};

/// Maker Color (CNAM subrecord) in AACT
pub struct SubrecordCNAM {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// Action Record (AACT)
pub struct AactRecord {
    pub form_id: u32,
    pub editor_id: Option<String>,
    pub color: Option<SubrecordCNAM>,
}

impl EsmRecord for AactRecord {
    const RECORD_TYPE: &'static [u8; 4] = b"AACT";

    fn parse(raw: &RawRecord) -> Option<Self> {
        if &raw.record_type != Self::RECORD_TYPE {
            return None;
        }

        let view = SubrecordView::new(&raw.subrecords);
        let editor_id = view.get_string(b"EDID");
        let color = view
            .find(b"CNAM")
            .filter(|d| d.len() >= 3)
            .map(|d| SubrecordCNAM {
                red: d[0],
                green: d[1],
                blue: d[2],
            });

        Some(Self {
            form_id: raw.form_id,
            editor_id,
            color,
        })
    }
}
