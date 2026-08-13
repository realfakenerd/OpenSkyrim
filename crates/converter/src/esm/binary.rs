use crate::esm::{
    extractors::extract_subrecords,
    mmap_reader::EsmReader,
    types::{GroupHeader, RawRecord, RecordHeader, WorldReference},
};
use color_eyre::{eyre::eyre, Result};
use flate2::read::ZlibDecoder;
use nom::{
    bytes::complete::take,
    number::complete::{le_f32, le_i32, le_u16, le_u32},
    IResult,
};
use std::{io::Read, path::Path};

const FLAG_COMPRESSED: u32 = 0x00040000;

pub trait EsmRecord: Sized {
    const RECORD_TYPE: &'static [u8; 4];
    fn parse(raw: &RawRecord) -> Option<Self>;
}

/// For STAT / MSTT / FURN
pub struct StaticRecord {
    pub form_id: u32,
    pub editor_id: Option<String>,
    pub model_path: Option<String>,
}
/// Binary Nom Parser for 24-byte Record Header
pub fn parse_record_header(input: &[u8]) -> IResult<&[u8], RecordHeader> {
    let (input, type_tag_bytes) = take(4usize)(input)?;
    let (input, data_size) = le_u32(input)?;
    let (input, flags) = le_u32(input)?;
    let (input, form_id) = le_u32(input)?;
    let (input, version_control) = le_u32(input)?;
    let (input, version) = le_u16(input)?;
    let (input, unknown) = le_u16(input)?;

    let mut type_tag = [0u8; 4];
    type_tag.copy_from_slice(type_tag_bytes);

    Ok((
        input,
        RecordHeader {
            type_tag,
            data_size,
            flags,
            form_id,
            version_control,
            version,
            unknown,
        },
    ))
}

pub fn parse_group_header(input: &[u8]) -> IResult<&[u8], GroupHeader> {
    let (input, _type_tag) = take(4usize)(input)?;
    let (input, data_size) = le_u32(input)?;
    let (input, label) = le_u32(input)?;
    let (input, group_type) = le_i32(input)?;

    Ok((
        input,
        GroupHeader {
            data_size,
            label,
            group_type,
        },
    ))
}

pub fn parse_group(
    input: &[u8],
    current_cell: Option<u32>,
    records: &mut Vec<RawRecord>,
) -> Result<()> {
    let mut curr = input;

    while curr.len() >= 24 {
        let peek_tag = &curr[..4];
        if peek_tag == b"GRUP" {
            let (rest, group) =
                parse_group_header(curr).map_err(|e| eyre!("Failed to parse group header: {e}"))?;
            let group_data_size = group.data_size as usize;
            let group_content = &rest[..group_data_size.min(rest.len())];

            let next_cell = match group.group_type {
                6 | 8 | 9 | 10 => Some(group.label),
                _ => current_cell,
            };

            parse_group(group_content, next_cell, records)?;

            curr = &rest[group_content.len()..];
        } else {
            let (rest, header) = parse_record_header(curr)
                .map_err(|e| eyre!("Failed to parse record header: {e}"))?;
            let record_len = (header.data_size as usize).min(rest.len());
            let raw_payload = &rest[..record_len];
            let subrecords = if header.flags & FLAG_COMPRESSED != 0 {
                if raw_payload.len() < 4 {
                    Vec::new()
                } else {
                    let decompressed_size = u32::from_le_bytes([
                        raw_payload[0],
                        raw_payload[1],
                        raw_payload[2],
                        raw_payload[3],
                    ]) as usize;

                    let compressed_bytes = &raw_payload[4..];
                    let mut decoder = ZlibDecoder::new(compressed_bytes);
                    let mut decompressed_data = Vec::with_capacity(decompressed_size);

                    if decoder.read_to_end(&mut decompressed_data).is_ok() {
                        extract_subrecords(&decompressed_data)
                    } else {
                        eprintln!("Failed to decompress record FormID: {:08x}", header.form_id);
                        Vec::new()
                    }
                }
            } else {
                extract_subrecords(raw_payload)
            };

            records.push(RawRecord {
                form_id: header.form_id,
                record_type: header.type_tag,
                flags: header.flags,
                subrecords,
                cell_form_id: current_cell,
            });
            curr = &rest[record_len..];
        }
    }

    Ok(())
}

pub fn parse_plugin_file(path: &Path) -> Result<Vec<RawRecord>> {
    let reader = EsmReader::open(path)?;
    let data = reader.as_slice();

    let mut records = Vec::new();

    let (remaining, _) =
        parse_record_header(data).map_err(|e| eyre!("Failed to parse record header: {e}"))?;
    parse_group(remaining, None, &mut records)?;

    Ok(records)
}

/// Parses REFR payload for Position and Base Form ID pointers
pub fn parse_refr_record(input: &[u8], form_id: u32) -> IResult<&[u8], WorldReference> {
    let mut base_form_id = 0u32;
    let mut pos_x = 0.0f32;
    let mut pos_y = 0.0f32;
    let mut pos_z = 0.0f32;
    let mut rot_x = 0.0f32;
    let mut rot_y = 0.0f32;
    let mut rot_z = 0.0f32;

    let mut curr = input;
    while curr.len() >= 6 {
        let (next, sub_tag) = take(4usize)(curr)?;
        let (next, sub_len) = le_u16(next)?;
        let sub_tag_str = std::str::from_utf8(sub_tag).unwrap_or("");

        if sub_tag_str == "NAME" && sub_len == 4 {
            let (_, id) = le_u32(next)?;
            base_form_id = id;
        } else if sub_tag_str == "DATA" && sub_len >= 24 {
            let (rem, px) = le_f32(next)?;
            let (rem, py) = le_f32(rem)?;
            let (rem, pz) = le_f32(rem)?;
            let (rem, rx) = le_f32(rem)?;
            let (rem, ry) = le_f32(rem)?;
            let (_, rz) = le_f32(rem)?;
            pos_x = px;
            pos_y = py;
            pos_z = pz;
            rot_x = rx;
            rot_y = ry;
            rot_z = rz;
        }

        let advance = (sub_len as usize).min(next.len());
        curr = &next[advance..];
    }

    Ok((
        curr,
        WorldReference {
            form_id,
            base_form_id,
            pos_x,
            pos_y,
            pos_z,
            rot_x,
            rot_y,
            rot_z,
            cell_form_id: 0,
        },
    ))
}
