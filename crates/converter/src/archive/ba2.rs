use color_eyre::{
    Result,
    eyre::{bail, ensure},
};
use flate2::read::ZlibDecoder;
use std::io::Read;

#[derive(Debug)]
struct GeneralRecord {
    offset: u64,
    packed_size: u32,
    unpacked_size: u32,
}

pub(crate) fn read_entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    ensure!(bytes.len() >= 24, "truncated BA2 header");
    ensure!(&bytes[..4] == b"BTDX", "invalid BA2 magic");
    let version = u32_at(bytes, 4)?;
    ensure!(
        matches!(version, 1 | 2 | 3 | 7 | 8),
        "unsupported BA2 version {version}"
    );
    ensure!(
        &bytes[8..12] == b"GNRL",
        "only general BA2 archives are supported"
    );
    let count = u32_at(bytes, 12)? as usize;
    let names_offset = u64_at(bytes, 16)? as usize;
    let records_end = 24usize
        .checked_add(
            count
                .checked_mul(36)
                .ok_or_else(|| color_eyre::eyre::eyre!("BA2 record count overflow"))?,
        )
        .ok_or_else(|| color_eyre::eyre::eyre!("BA2 record table overflow"))?;
    ensure!(
        records_end <= bytes.len() && names_offset <= bytes.len(),
        "truncated BA2 tables"
    );

    let mut records = Vec::with_capacity(count);
    for index in 0..count {
        let base = 24 + index * 36;
        records.push(GeneralRecord {
            offset: u64_at(bytes, base + 16)?,
            packed_size: u32_at(bytes, base + 24)?,
            unpacked_size: u32_at(bytes, base + 28)?,
        });
    }

    let mut cursor = names_offset;
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        let length = u16_at(bytes, cursor)? as usize;
        cursor += 2;
        let name = bytes
            .get(cursor..cursor + length)
            .ok_or_else(|| color_eyre::eyre::eyre!("truncated BA2 filename"))?;
        names.push(
            String::from_utf8_lossy(name)
                .trim_end_matches('\0')
                .to_string(),
        );
        cursor += length;
    }

    records
        .into_iter()
        .zip(names)
        .map(|(record, name)| {
            let stored_size = if record.packed_size == 0 {
                record.unpacked_size
            } else {
                record.packed_size
            } as usize;
            let start = usize::try_from(record.offset)
                .map_err(|_| color_eyre::eyre::eyre!("BA2 offset does not fit usize"))?;
            let payload = bytes
                .get(
                    start
                        ..start
                            .checked_add(stored_size)
                            .ok_or_else(|| color_eyre::eyre::eyre!("BA2 range overflow"))?,
                )
                .ok_or_else(|| color_eyre::eyre::eyre!("BA2 payload out of bounds: {name}"))?;
            if record.packed_size == 0 {
                return Ok((name, payload.to_vec()));
            }
            let mut decoded = Vec::with_capacity(record.unpacked_size as usize);
            ZlibDecoder::new(payload).read_to_end(&mut decoded)?;
            if decoded.len() != record.unpacked_size as usize {
                bail!("BA2 decompressed size mismatch for {name}");
            }
            Ok((name, decoded))
        })
        .collect()
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or_else(|| color_eyre::eyre::eyre!("truncated u16 at {offset}"))?
            .try_into()
            .unwrap(),
    ))
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| color_eyre::eyre::eyre!("truncated u32 at {offset}"))?
            .try_into()
            .unwrap(),
    ))
}

fn u64_at(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| color_eyre::eyre::eyre!("truncated u64 at {offset}"))?
            .try_into()
            .unwrap(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_uncompressed_general_ba2_fixture() {
        let name = b"textures/test.dds";
        let payload = b"DDS ";
        let names_offset = 24 + 36;
        let payload_offset = names_offset + 2 + name.len();
        let mut bytes = vec![0u8; payload_offset];
        bytes[..4].copy_from_slice(b"BTDX");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..12].copy_from_slice(b"GNRL");
        bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
        bytes[16..24].copy_from_slice(&(names_offset as u64).to_le_bytes());
        bytes[40..48].copy_from_slice(&(payload_offset as u64).to_le_bytes());
        bytes[52..56].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes[names_offset..names_offset + 2].copy_from_slice(&(name.len() as u16).to_le_bytes());
        bytes[names_offset + 2..payload_offset].copy_from_slice(name);
        bytes.extend_from_slice(payload);
        let entries = read_entries(&bytes).unwrap();
        assert_eq!(
            entries,
            vec![("textures/test.dds".into(), payload.to_vec())]
        );
    }
}
