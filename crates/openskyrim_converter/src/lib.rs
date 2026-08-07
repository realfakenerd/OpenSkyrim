//! OpenSkyrim Converter Library
//! Handles offline asset transformation: .nif -> .glb, .dds -> KTX2, .esm -> SQLite, .pex -> Luau.

pub mod esm_parser;
pub mod mesh;
pub mod script;
pub mod texture;

pub use esm_parser::EsmParser;

pub struct AssetPipeline;

impl AssetPipeline {
    pub fn run() {
        println!("OpenSkyrim Converter Pipeline initialized.");
    }
}
