//! **TREE** records contain information on trees as well other flora that can be activated.
//! Records between the dark lines must all be present, or just PFPC.

use crate::esm::{
    extractors::SubrecordView,
    records::{
        EsmRecord, RawRecord,
        record_type::{ModelData, SubrecordOBND},
    },
};

/// Tree physics / animation parameters (CNAM subrecord, 48 bytes total: 12 x 4-byte floats)
#[derive(Debug)]
pub struct SubrecordCNAM {
    pub trunk_flexibility: f32,
    pub branch_flexibility: f32,
    pub unknown_floats: [f32; 8],
    pub leaf_amplitude: f32,
    pub leaf_frequency: f32,
}

/// Tree / Harvestable Flora Record (TREE)
#[derive(Debug)]
pub struct TreeRecord {
    pub form_id: u32,
    pub editor_id: Option<String>,
    pub object_bounds: Option<SubrecordOBND>,
    pub model: Option<ModelData>,
    pub result_item: Option<u32>,
    pub use_sound: Option<u32>,
    pub percent_chance: Option<[u8; 4]>,
    pub name: Option<String>,
    pub data: Option<SubrecordCNAM>,
}

impl EsmRecord for TreeRecord {
    const RECORD_TYPE: &'static [u8; 4] = b"TREE";
    fn parse(raw: &RawRecord) -> Option<Self> {
        if &raw.record_type != Self::RECORD_TYPE {
            return None;
        }

        let view = SubrecordView::new(&raw.subrecords);
        let editor_id = view.get_string(b"EDID");
        let object_bounds = view.get_obnd();

        let model = view.get_model();
        let result_item = view.get_form_id(b"PFIG");
        let use_sound = view.get_form_id(b"SNAM");
        let percent_chance = view
            .find(b"PFPC")
            .filter(|d| d.len() >= 4)
            .map(|d| [d[0], d[1], d[2], d[3]]);

        let name = view.get_string(b"FULL");
        let data = view.find(b"CNAM").filter(|d| d.len() >= 48).map(|d| {
            let mut floats = [0.0f32; 12];
            for (i, chunk) in d.chunks_exact(4).take(12).enumerate() {
                floats[i] = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }

            let mut unk = [0.0f32; 8];
            unk.copy_from_slice(&floats[2..10]);

            SubrecordCNAM {
                trunk_flexibility: floats[0],
                branch_flexibility: floats[1],
                unknown_floats: unk,
                leaf_amplitude: floats[10],
                leaf_frequency: floats[11],
            }
        });

        Some(Self {
            form_id: raw.form_id,
            editor_id,
            object_bounds,
            model,
            result_item,
            use_sound,
            percent_chance,
            name,
            data,
        })
    }
}
