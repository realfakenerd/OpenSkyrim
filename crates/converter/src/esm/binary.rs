use crate::esm::{
    extractors::extract_subrecords,
    mmap_reader::EsmReader,
    records::RawRecord,
    types::{GroupHeader, RecordHeader, WorldReference},
};
use color_eyre::{Result, eyre::eyre};
use flate2::read::ZlibDecoder;
use nom::{
    IResult,
    bytes::complete::take,
    number::complete::{le_f32, le_i32, le_u16, le_u32},
};
use std::{io::Read, path::Path};

const FLAG_COMPRESSED: u32 = 0x00040000;

#[derive(Debug, Clone, Default)]
pub struct PluginMetadata {
    pub masters: Vec<String>,
    pub flags: u32,
}

pub fn parse_plugin_metadata(path: &Path) -> Result<PluginMetadata> {
    let reader = EsmReader::open(path)?;
    let data = reader.as_slice();
    let (rest, header) =
        parse_record_header(data).map_err(|error| eyre!("failed to parse TES4 header: {error}"))?;
    if &header.type_tag != b"TES4" || header.data_size as usize > rest.len() {
        return Err(eyre!("invalid TES4 record in {}", path.display()));
    }
    let payload = &rest[..header.data_size as usize];
    let decoded = if header.flags & FLAG_COMPRESSED != 0 {
        if payload.len() < 4 {
            return Err(eyre!("truncated compressed TES4 record"));
        }
        let expected = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
        let mut output = Vec::with_capacity(expected);
        ZlibDecoder::new(&payload[4..]).read_to_end(&mut output)?;
        if output.len() != expected {
            return Err(eyre!("TES4 decompressed size mismatch"));
        }
        output
    } else {
        payload.to_vec()
    };
    let masters = extract_subrecords(&decoded)
        .into_iter()
        .filter_map(|(tag, data)| {
            (tag == b"MAST").then(|| {
                String::from_utf8_lossy(&data)
                    .trim_end_matches('\0')
                    .to_string()
            })
        })
        .collect();
    Ok(PluginMetadata {
        masters,
        flags: header.flags,
    })
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
    let (input, _) = take(8usize)(input)?;

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
    current_worldspace: Option<u32>,
    records: &mut Vec<RawRecord>,
) -> Result<()> {
    let mut curr = input;

    while curr.len() >= 24 {
        let peek_tag = &curr[..4];
        if peek_tag == b"GRUP" {
            let (rest, group) =
                parse_group_header(curr).map_err(|e| eyre!("Failed to parse group header: {e}"))?;
            let group_data_size = group.data_size as usize;
            if group_data_size < 24 {
                return Err(eyre!("invalid GRUP size {}", group.data_size));
            }
            let content_size = group_data_size - 24;
            if content_size > rest.len() {
                return Err(eyre!(
                    "truncated GRUP payload: expected {content_size}, got {}",
                    rest.len()
                ));
            }
            let group_content = &rest[..content_size];

            let next_cell = match group.group_type {
                6 | 8 | 9 | 10 => Some(group.label),
                _ => current_cell,
            };
            let next_worldspace = match group.group_type {
                1 => Some(group.label),
                _ => current_worldspace,
            };

            parse_group(group_content, next_cell, next_worldspace, records)?;

            curr = &rest[content_size..];
        } else {
            let (rest, header) = parse_record_header(curr)
                .map_err(|e| eyre!("Failed to parse record header: {e}"))?;
            let record_len = header.data_size as usize;
            if record_len > rest.len() {
                return Err(eyre!(
                    "truncated {:?} record {:08x}: expected {record_len}, got {}",
                    header.type_tag,
                    header.form_id,
                    rest.len()
                ));
            }
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

                    decoder
                        .read_to_end(&mut decompressed_data)
                        .map_err(|error| {
                            eyre!(
                                "failed to decompress record {:08x}: {error}",
                                header.form_id
                            )
                        })?;
                    if decompressed_data.len() != decompressed_size {
                        return Err(eyre!(
                            "decompressed size mismatch for {:08x}: expected {decompressed_size}, got {}",
                            header.form_id,
                            decompressed_data.len()
                        ));
                    }
                    extract_subrecords(&decompressed_data)
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
                worldspace_form_id: current_worldspace,
                load_order: 0,
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

    let (after_header, header) =
        parse_record_header(data).map_err(|e| eyre!("Failed to parse record header: {e}"))?;
    if &header.type_tag != b"TES4" {
        return Err(eyre!("plugin does not start with TES4"));
    }
    let header_payload = header.data_size as usize;
    if header_payload > after_header.len() {
        return Err(eyre!("truncated TES4 payload"));
    }
    parse_group(&after_header[header_payload..], None, None, &mut records)?;

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
