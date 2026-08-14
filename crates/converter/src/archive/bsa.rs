use color_eyre::{
    Result,
    eyre::{WrapErr, bail, ensure},
};
use flate2::read::ZlibDecoder;
use std::io::Read;

const ARCHIVE_COMPRESSED: u32 = 0x0004;
const ARCHIVE_EMBED_FILE_NAMES: u32 = 0x0100;
const FILE_COMPRESSION_TOGGLE: u32 = 0x4000_0000;
const FILE_SIZE_MASK: u32 = 0x3fff_ffff;

#[derive(Debug)]
struct FileRecord {
    folder: String,
    size_flags: u32,
    offset: u32,
}

pub(crate) fn read_entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    ensure!(bytes.len() >= 36, "truncated BSA header");
    ensure!(&bytes[..4] == b"BSA\0", "invalid BSA magic");
    let version = u32_at(bytes, 4)?;
    ensure!(
        matches!(version, 104 | 105),
        "unsupported BSA version {version}"
    );
    let archive_flags = u32_at(bytes, 12)?;
    let folder_count = u32_at(bytes, 16)? as usize;
    let file_count = u32_at(bytes, 20)? as usize;
    let folder_record_size = if version >= 105 { 24 } else { 16 };
    let folder_table_end = 36usize
        .checked_add(
            folder_count
                .checked_mul(folder_record_size)
                .ok_or_else(|| color_eyre::eyre::eyre!("BSA folder count overflow"))?,
        )
        .ok_or_else(|| color_eyre::eyre::eyre!("BSA folder table overflow"))?;
    ensure!(
        folder_table_end <= bytes.len(),
        "truncated BSA folder table"
    );

    let mut folder_file_counts = Vec::with_capacity(folder_count);
    for index in 0..folder_count {
        let base = 36 + index * folder_record_size;
        folder_file_counts.push(u32_at(bytes, base + 8)? as usize);
    }

    let mut cursor = folder_table_end;
    let mut records = Vec::with_capacity(file_count);
    for count in folder_file_counts {
        let name_len = *bytes
            .get(cursor)
            .ok_or_else(|| color_eyre::eyre::eyre!("missing BSA folder name"))?
            as usize;
        cursor += 1;
        ensure!(
            cursor + name_len <= bytes.len(),
            "truncated BSA folder name"
        );
        let folder_bytes = &bytes[cursor..cursor + name_len];
        let folder = String::from_utf8_lossy(folder_bytes)
            .trim_end_matches('\0')
            .to_string();
        cursor += name_len;
        for _ in 0..count {
            ensure!(cursor + 16 <= bytes.len(), "truncated BSA file record");
            records.push(FileRecord {
                folder: folder.clone(),
                size_flags: u32_at(bytes, cursor + 8)?,
                offset: u32_at(bytes, cursor + 12)?,
            });
            cursor += 16;
        }
    }
    ensure!(
        records.len() == file_count,
        "BSA file count mismatch: header={file_count}, records={}",
        records.len()
    );

    let mut names = Vec::with_capacity(file_count);
    for _ in 0..file_count {
        let tail = bytes
            .get(cursor..)
            .ok_or_else(|| color_eyre::eyre::eyre!("missing BSA filename table"))?;
        let end = tail
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| color_eyre::eyre::eyre!("unterminated BSA filename"))?;
        names.push(String::from_utf8_lossy(&tail[..end]).to_string());
        cursor += end + 1;
    }

    records
        .into_iter()
        .zip(names)
        .map(|(record, file_name)| {
            let full_name = if record.folder.is_empty() {
                file_name
            } else {
                format!("{}/{}", record.folder, file_name)
            };
            let stored_size = (record.size_flags & FILE_SIZE_MASK) as usize;
            let start = record.offset as usize;
            let end = start
                .checked_add(stored_size)
                .ok_or_else(|| color_eyre::eyre::eyre!("BSA file range overflow"))?;
            let mut payload = bytes.get(start..end).ok_or_else(|| {
                color_eyre::eyre::eyre!("BSA file payload out of bounds: {full_name}")
            })?;
            if archive_flags & ARCHIVE_EMBED_FILE_NAMES != 0 {
                let embedded_len = *payload
                    .first()
                    .ok_or_else(|| color_eyre::eyre::eyre!("missing embedded BSA filename"))?
                    as usize;
                ensure!(
                    payload.len() > embedded_len,
                    "truncated embedded BSA filename"
                );
                payload = &payload[embedded_len + 1..];
            }
            let compressed = (archive_flags & ARCHIVE_COMPRESSED != 0)
                ^ (record.size_flags & FILE_COMPRESSION_TOGGLE != 0);
            let data = if compressed {
                decompress(payload, version)
                    .wrap_err_with(|| format!("failed to decompress {full_name}"))?
            } else {
                payload.to_vec()
            };
            Ok((full_name, data))
        })
        .collect()
}

fn decompress(payload: &[u8], version: u32) -> Result<Vec<u8>> {
    ensure!(payload.len() >= 4, "compressed BSA payload is truncated");
    let expected = u32_at(payload, 0)? as usize;
    let compressed = &payload[4..];
    let decoded = if version >= 105 {
        lz4_flex::block::decompress(compressed, expected)
            .or_else(|_| {
                let mut output = Vec::with_capacity(expected);
                ZlibDecoder::new(compressed)
                    .read_to_end(&mut output)
                    .map(|_| output)
                    .map_err(|_| lz4_flex::block::DecompressError::OutputTooSmall {
                        expected,
                        actual: 0,
                    })
            })
            .map_err(|error| color_eyre::eyre::eyre!("LZ4/zlib decoding failed: {error}"))?
    } else {
        let mut output = Vec::with_capacity(expected);
        ZlibDecoder::new(compressed).read_to_end(&mut output)?;
        output
    };
    if decoded.len() != expected {
        bail!(
            "decompressed size mismatch: expected {expected}, got {}",
            decoded.len()
        );
    }
    Ok(decoded)
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32> {
    let raw: [u8; 4] = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| color_eyre::eyre::eyre!("truncated u32 at offset {offset}"))?
        .try_into()
        .unwrap();
    Ok(u32::from_le_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_uncompressed_skyrim_bsa_fixture() {
        let folder = b"scripts\0";
        let name = b"hello.pex\0";
        let payload = b"PEX";
        let folder_table_end = 36 + 16;
        let names_start = folder_table_end + 1 + folder.len() + 16;
        let payload_offset = names_start + name.len();
        let mut bytes = vec![0u8; payload_offset];
        bytes[..4].copy_from_slice(b"BSA\0");
        bytes[4..8].copy_from_slice(&104u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&36u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&1u32.to_le_bytes());
        bytes[24..28].copy_from_slice(&(folder.len() as u32).to_le_bytes());
        bytes[28..32].copy_from_slice(&(name.len() as u32).to_le_bytes());
        bytes[44..48].copy_from_slice(&1u32.to_le_bytes());
        let mut cursor = folder_table_end;
        bytes[cursor] = folder.len() as u8;
        cursor += 1;
        bytes[cursor..cursor + folder.len()].copy_from_slice(folder);
        cursor += folder.len();
        bytes[cursor + 8..cursor + 12].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes[cursor + 12..cursor + 16].copy_from_slice(&(payload_offset as u32).to_le_bytes());
        cursor += 16;
        bytes[cursor..cursor + name.len()].copy_from_slice(name);
        bytes.extend_from_slice(payload);

        let entries = read_entries(&bytes).unwrap();
        assert_eq!(
            entries,
            vec![("scripts/hello.pex".into(), payload.to_vec())]
        );
    }
}
