use color_eyre::{
    Result,
    eyre::{WrapErr, ensure},
};
use ddsfile::Dds;
use memmap2::Mmap;
use std::{
    ffi::c_void,
    fs::{self, File},
    io::Cursor,
    path::Path,
    sync::Once,
};

const KTX2_IDENTIFIER: &[u8; 12] = b"\xABKTX 20\xBB\r\n\x1A\n";
const FLAG_THREADED: u32 = 1 << 9;
const FLAG_KTX2: u32 = 1 << 11;
const FLAG_SRGB: u32 = 1 << 13;
const FLAG_GENERATE_MIPS_CLAMP: u32 = 1 << 14;
const FLAG_UASTC: u32 = 1 << 17;
const UASTC_LEVEL_DEFAULT: u8 = 2;
const ETC1S_QUALITY_DEFAULT: u8 = 192;
static BASIS_INIT: Once = Once::new();

unsafe extern "C" {
    fn opensky_basis_compress_ktx2(
        rgba: *const u8,
        width: u32,
        height: u32,
        flags_and_quality: u32,
        uastc_rdo_quality: f32,
        size: *mut usize,
    ) -> *mut c_void;
    fn opensky_basis_free(data: *mut c_void);
}

pub struct TextureConverter;

impl TextureConverter {
    pub fn convert_dds_to_ktx2(input: &Path, output: &Path, normal_map: bool) -> Result<()> {
        Self::convert_dds_to_ktx2_with_options(
            input,
            output,
            normal_map,
            ETC1S_QUALITY_DEFAULT,
            UASTC_LEVEL_DEFAULT,
        )
    }

    pub fn convert_dds_to_ktx2_with_options(
        input: &Path,
        output: &Path,
        normal_map: bool,
        etc1s_quality: u8,
        uastc_level: u8,
    ) -> Result<()> {
        let file =
            File::open(input).wrap_err_with(|| format!("failed to open {}", input.display()))?;
        let mmap = unsafe { Mmap::map(&file) }
            .wrap_err_with(|| format!("failed to memory-map {}", input.display()))?;

        let ktx2 = Self::convert_with_options(&mmap, normal_map, etc1s_quality, uastc_level)?;
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, ktx2).wrap_err_with(|| format!("failed to write {}", output.display()))
    }

    pub fn convert(dds_bytes: &[u8], normal_map: bool) -> Result<Vec<u8>> {
        Self::convert_with_options(
            dds_bytes,
            normal_map,
            ETC1S_QUALITY_DEFAULT,
            UASTC_LEVEL_DEFAULT,
        )
    }

    pub fn convert_with_options(
        dds_bytes: &[u8],
        normal_map: bool,
        etc1s_quality: u8,
        uastc_level: u8,
    ) -> Result<Vec<u8>> {
        let dds = Dds::read(Cursor::new(dds_bytes)).wrap_err("invalid DDS")?;
        ensure!(
            dds.get_depth() <= 1,
            "volume DDS textures are not supported"
        );
        ensure!(
            dds.get_num_array_layers() <= 1,
            "DDS texture arrays are not supported"
        );
        let image =
            image_dds::image_from_dds(&dds, 0).wrap_err("DDS pixel format cannot be decoded")?;
        let width = image.width();
        let height = image.height();
        let rgba = image.into_raw();
        let generate_mips = dds.get_num_mipmap_levels() > 1;
        let result = encode_basis_ktx2(
            width,
            height,
            &rgba,
            normal_map,
            generate_mips,
            etc1s_quality,
            uastc_level,
        )?;
        validate_ktx2(&result, normal_map)?;
        Ok(result)
    }

    pub fn is_normal_map(path: &Path) -> bool {
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        stem.ends_with("_n") || stem.ends_with("_normal") || stem.contains("normalmap")
    }
}

fn encode_basis_ktx2(
    width: u32,
    height: u32,
    rgba: &[u8],
    normal_map: bool,
    generate_mips: bool,
    etc1s_quality: u8,
    uastc_level: u8,
) -> Result<Vec<u8>> {
    ensure!(
        width > 0 && height > 0,
        "texture dimensions must be non-zero"
    );
    ensure!(
        rgba.len() == width as usize * height as usize * 4,
        "RGBA payload size mismatch"
    );
    BASIS_INIT.call_once(basis_universal::encoder_init);
    ensure!(etc1s_quality > 0, "ETC1S quality must be greater than zero");
    ensure!(uastc_level <= 4, "UASTC level must be between 0 and 4");
    let mut flags = FLAG_KTX2 | FLAG_THREADED;
    if generate_mips {
        flags |= FLAG_GENERATE_MIPS_CLAMP;
    }
    if normal_map {
        flags |= FLAG_UASTC | u32::from(uastc_level);
    } else {
        flags |= FLAG_SRGB | u32::from(etc1s_quality);
    }
    let mut size = 0usize;
    // SAFETY: the encoder copies the complete RGBA slice during this call. The
    // returned allocation is owned by Basis and freed after copying below.
    let data =
        unsafe { opensky_basis_compress_ktx2(rgba.as_ptr(), width, height, flags, 0.0, &mut size) };
    ensure!(
        !data.is_null() && size > 0,
        "Basis Universal compression failed"
    );
    // SAFETY: a successful encoder call returns exactly `size` initialized bytes.
    let output = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size).to_vec() };
    // SAFETY: `data` was allocated by basis_compress and has not been freed yet.
    unsafe { opensky_basis_free(data) };
    Ok(output)
}

fn validate_ktx2(bytes: &[u8], normal_map: bool) -> Result<()> {
    ensure!(
        bytes.starts_with(KTX2_IDENTIFIER),
        "encoder did not produce KTX2"
    );
    let reader = ktx2::Reader::new(bytes)
        .map_err(|error| color_eyre::eyre::eyre!("generated invalid KTX2: {error:?}"))?;
    ensure!(
        reader.header().pixel_width > 0,
        "KTX2 has invalid dimensions"
    );
    if !normal_map {
        ensure!(
            reader.header().supercompression_scheme == Some(ktx2::SupercompressionScheme::BasisLZ),
            "ETC1S output is not BasisLZ-supercompressed"
        );
    }
    ensure!(
        reader.levels().next().is_some(),
        "KTX2 contains no image levels"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_basis_lz_ktx2() {
        let pixels = [255, 0, 0, 255].repeat(16);
        let bytes = encode_basis_ktx2(4, 4, &pixels, false, false, 192, 2).unwrap();
        validate_ktx2(&bytes, false).unwrap();
        assert!(bytes.len() < pixels.len() + 256);
    }

    #[test]
    fn creates_uastc_normal_map_ktx2() {
        let pixels = [128, 128, 255, 255].repeat(16);
        let bytes = encode_basis_ktx2(4, 4, &pixels, true, false, 192, 2).unwrap();
        validate_ktx2(&bytes, true).unwrap();
    }

    #[test]
    fn generates_mipmap_chain_for_mipped_source() {
        let pixels = [64, 128, 192, 255].repeat(64);
        let bytes = encode_basis_ktx2(8, 8, &pixels, false, true, 192, 2).unwrap();
        let reader = ktx2::Reader::new(&bytes).unwrap();
        assert_eq!(reader.header().level_count, 4);
        assert_eq!(reader.levels().count(), 4);
    }
}
