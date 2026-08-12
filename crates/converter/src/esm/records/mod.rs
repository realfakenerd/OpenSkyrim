pub mod aact;
pub mod achr;
pub mod clfm;
pub mod record_type;
pub mod tes4;
pub mod tree;

#[derive(Debug, Clone)]
pub struct RawRecord {
    pub form_id: u32,
    pub record_type: [u8; 4],
    pub flags: u32,
    pub subrecords: Vec<(Vec<u8>, Vec<u8>)>,
    pub cell_form_id: Option<u32>,
}

pub trait EsmRecord: Sized {
    const RECORD_TYPE: &'static [u8; 4];
    fn parse(raw: &RawRecord) -> Option<Self>;
}
