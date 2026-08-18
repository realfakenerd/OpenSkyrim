use crate::dev::*;
use log::error;

use nom::number::complete::{le_u16, le_u32, le_u8};
use nom_derive::{NomLE, Parse};

#[derive(Debug, NomLE)]
pub struct NifHeader {
    pub file_desc: StringN,
    pub nif_version: NifFileVersion,
    pub endian_type: Endianess,
    pub user_version: u32,
    pub block_count: u32,
    pub bethesda_version: u32,
    #[nom(
        Map = "|x| if x.0.len() > 0 { Some(x) } else { None }",
        Parse = "SizedString8::parse"
    )]
    pub author: Option<SizedString8>,
    #[nom(
        Map = "|x| if x.0.len() > 0 { Some(x) } else { None }",
        Parse = "SizedString8::parse"
    )]
    pub process_script: Option<SizedString8>,
    #[nom(
        Map = "|x| if x.0.len() > 0 { Some(x) } else { None }",
        Parse = "SizedString8::parse"
    )]
    pub export_script: Option<SizedString8>,
    #[nom(
        Map = "|x| if x.0.len() > 0 { Some(x) } else { None }",
        Parse = "SizedString8::parse"
    )]
    pub max_filepath: Option<SizedString8>,
    #[nom(LengthCount = "le_u16")]
    pub block_types: Vec<SizedString32>,
    #[nom(Count = "block_count")]
    pub block_type_index: Vec<u16>,
    #[nom(Count = "block_count")]
    pub block_size_index: Vec<u32>,
    pub string_count: u32,
    pub string_max_size: u32,
    #[nom(Count = "string_count")]
    pub strings: Vec<SizedString32>,
    #[nom(LengthCount = "le_u32")]
    pub groups: Vec<u32>,
}

impl NifHeader {
    pub fn get_block_type(&self, index: usize) -> Result<&str, ()> {
        if index < self.block_type_index.len() {
            let i = self.block_type_index[index] as usize;
            if i < self.block_types.len() {
                Ok(&self.block_types[i].0)
            } else {
                error!("Block type index out of range: {}", i);
                Err(())
            }
        } else {
            error!("Block type index out of range: {}", index);
            Err(())
        }
    }

    pub fn get_block_size(&self, index: usize) -> Result<u32, ()> {
        if index < self.block_size_index.len() {
            Ok(self.block_size_index[index])
        } else {
            error!("Block size index out of range: {}", index);
            Err(())
        }
    }

    pub fn get_string(&self, index: usize) -> Result<&str, String> {
        if index < self.strings.len() {
            Ok(&self.strings[index].0)
        } else {
            Err(format!("String index out of range: {}", index))
        }
    }
}

#[derive(NomLE, PartialEq)]
pub struct NifFileVersion(pub u32);

impl NifFileVersion {
    pub fn major(&self) -> u8 {
        (self.0 >> 24) as u8
    }
    pub fn minor(&self) -> u8 {
        (self.0 >> 16) as u8
    }
    pub fn build(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub fn patch(&self) -> u8 {
        (self.0 >> 0) as u8
    }
}

impl std::fmt::Debug for NifFileVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self, self.0)
    }
}

impl std::fmt::Display for NifFileVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major(),
            self.minor(),
            self.build(),
            self.patch()
        )
    }
}

#[derive(Debug)]
pub enum Endianess {
    Big,
    Little,
}

impl Parse<&[u8]> for Endianess {
    fn parse(i: &[u8]) -> nom::IResult<&[u8], Self, nom::error::Error<&[u8]>> {
        let (i, endian_type) = le_u8(i)?;
        match endian_type {
            0x00 => Ok((i, Endianess::Big)),
            0x01 => Ok((i, Endianess::Little)),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}

pub trait NifHeaderTraits {
    fn get_block_type(&self, index: usize) -> Result<&str, ()>;
    fn get_block_size(&self, index: usize) -> Result<u32, ()>;
    fn get_string(&self, index: usize) -> Result<&str, String>;
}
