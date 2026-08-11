use std::{fs::File, path::Path};

use color_eyre::eyre::Result;
use memmap2::Mmap;

pub struct EsmReader {
    _file: File,
    mmap: Mmap,
}

impl EsmReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { _file: file, mmap })
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }
}
