use color_eyre::{
    Result,
    eyre::{WrapErr, ensure},
};
use project_wormhole_esm::structs::strings::{SizedString8, SizedString32, StringN};
use project_wormhole_nif::{
    nif_block::NifBlock,
    nif_file::{NifFile, nif_to_model, nif_to_static_model},
    nif_header::{Endianess, NifFileVersion, NifHeader},
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub struct MeshConverter;

#[derive(Debug, Clone, Default, Serialize)]
pub struct NifParseDiagnostics {
    pub block_count: usize,
    pub parsed_block_count: usize,
    pub geometry_block_count: usize,
    pub block_types: BTreeMap<String, usize>,
    pub fallback_blocks: BTreeMap<String, usize>,
    pub fallback_offsets: BTreeMap<String, Vec<usize>>,
}

impl MeshConverter {
    pub fn dependency_paths(nif_path: &Path) -> Vec<PathBuf> {
        find_skeleton(nif_path).into_iter().collect()
    }

    pub fn convert_nif_to_glb<P: AsRef<Path>>(nif_path: P, glb_output_path: P) -> Result<()> {
        let nif_path = nif_path.as_ref();
        let (nif, _) = open_nif_resilient(nif_path)?;
        let skeleton = if nif.has_skeleton() {
            let skeleton_path = find_skeleton(nif_path).ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "skinned NIF requires a skeleton, but none was found near {}",
                    nif_path.display()
                )
            })?;
            Some(open_nif_resilient(&skeleton_path)?.0)
        } else {
            None
        };
        let model = catch_unwind(AssertUnwindSafe(|| nif_to_model(&nif, skeleton.as_ref())))
            .map_err(|_| {
                color_eyre::eyre::eyre!("NIF model conversion panicked for {}", nif_path.display())
            })?
            .map_err(|error| color_eyre::eyre::eyre!("NIF model conversion failed: {error}"))?;
        model
            .validate()
            .map_err(|error| color_eyre::eyre::eyre!("invalid converted NIF model: {error}"))?;
        ensure!(
            !model.static_meshes.is_empty() || !model.skeletal_meshes.is_empty(),
            "NIF contains no supported mesh geometry"
        );
        let name = nif_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let specular_textures: Vec<_> = model
            .materials
            .iter()
            .map(|material| material.specular.clone())
            .collect();
        let output = glb_output_path.as_ref();
        let mut glb = catch_unwind(AssertUnwindSafe(|| model.to_glb(name.clone())))
            .map_err(|_| color_eyre::eyre::eyre!("NIF GLB export panicked"))?;
        if glb_bounds_from_bytes(&glb).is_err() {
            let static_model = nif_to_static_model(&nif)
                .map_err(|error| color_eyre::eyre::eyre!("static NIF fallback failed: {error}"))?;
            ensure!(
                !static_model.static_meshes.is_empty(),
                "NIF contains no supported mesh geometry"
            );
            glb = catch_unwind(AssertUnwindSafe(|| static_model.to_glb(name)))
                .map_err(|_| color_eyre::eyre::eyre!("static NIF GLB export panicked"))?;
        }
        let glb = rewrite_materials_and_texture_uris(glb, &specular_textures, output)?;
        ensure!(
            glb.len() >= 12 && &glb[..4] == b"glTF",
            "NIF exporter produced an invalid GLB header"
        );
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, glb).wrap_err_with(|| format!("failed to write {}", output.display()))
    }

    pub fn inspect_nif(path: &Path) -> Result<NifParseDiagnostics> {
        open_nif_resilient(path).map(|(_, diagnostics)| diagnostics)
    }

    /// Reads the accessor bounds written to a GLB and applies the complete glTF
    /// node hierarchy. This avoids loading vertex buffers merely to build the
    /// runtime spatial index.
    pub fn glb_bounds(path: &Path) -> Result<shared::Bounds3> {
        let bytes =
            fs::read(path).wrap_err_with(|| format!("failed to read {}", path.display()))?;
        glb_bounds_from_bytes(&bytes)
            .wrap_err_with(|| format!("failed to extract bounds from {}", path.display()))
    }

    /// Returns every external image URI referenced by a GLB. Embedded images
    /// have no URI and are intentionally omitted.
    pub fn glb_texture_uris(path: &Path) -> Result<Vec<String>> {
        let bytes =
            fs::read(path).wrap_err_with(|| format!("failed to read {}", path.display()))?;
        let document = glb_json_from_bytes(&bytes)
            .wrap_err_with(|| format!("failed to inspect textures in {}", path.display()))?;
        let mut uris = document
            .get("images")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|image| image.get("uri").and_then(serde_json::Value::as_str))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        uris.sort_by_key(|uri| uri.to_ascii_lowercase());
        uris.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        Ok(uris)
    }
}

fn glb_json_from_bytes(bytes: &[u8]) -> Result<serde_json::Value> {
    ensure!(
        bytes.len() >= 20 && &bytes[..4] == b"glTF",
        "invalid GLB container"
    );
    ensure!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()) == 2,
        "unsupported GLB version"
    );
    let declared_length = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    ensure!(declared_length == bytes.len(), "GLB length is inconsistent");
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    ensure!(&bytes[16..20] == b"JSON", "GLB JSON chunk is missing");
    let json_end = 20usize
        .checked_add(json_length)
        .ok_or_else(|| color_eyre::eyre::eyre!("GLB JSON range overflow"))?;
    let json = bytes
        .get(20..json_end)
        .ok_or_else(|| color_eyre::eyre::eyre!("truncated GLB JSON chunk"))?;
    serde_json::from_slice(json).wrap_err("invalid glTF JSON")
}

fn open_nif_resilient(path: &Path) -> Result<(NifFile, NifParseDiagnostics)> {
    let bytes = fs::read(path).wrap_err_with(|| format!("failed to read {}", path.display()))?;
    let (mut data, header) = parse_skyrim_header(&bytes, path)?;
    let block_count = usize::try_from(header.block_count)
        .wrap_err_with(|| format!("NIF block count is out of range in {}", path.display()))?;
    ensure!(
        header.block_type_index.len() == block_count
            && header.block_size_index.len() == block_count,
        "NIF header block tables are inconsistent in {}",
        path.display()
    );
    let mut diagnostics = NifParseDiagnostics {
        block_count,
        ..Default::default()
    };
    let mut blocks = Vec::with_capacity(block_count);
    for index in 0..block_count {
        let size = usize::try_from(header.block_size_index[index])
            .wrap_err("NIF block size is out of range")?;
        ensure!(
            data.len() >= size,
            "NIF block {index} is truncated in {}",
            path.display()
        );
        let (raw, remaining) = data.split_at(size);
        data = remaining;
        let block_type = header
            .get_block_type(index)
            .map_err(|_| {
                color_eyre::eyre::eyre!(
                    "NIF block {index} has an invalid type index in {}",
                    path.display()
                )
            })?
            .to_owned();
        *diagnostics
            .block_types
            .entry(block_type.clone())
            .or_default() += 1;
        let parsed = catch_unwind(AssertUnwindSafe(|| {
            NifBlock::parse(raw, block_type.clone())
        }));
        let block = match parsed {
            Ok(Ok((_, NifBlock::Unhandled))) => {
                diagnostics
                    .fallback_offsets
                    .entry(block_type.clone())
                    .or_default()
                    .push(0);
                *diagnostics.fallback_blocks.entry(block_type).or_default() += 1;
                NifBlock::Unhandled
            }
            Ok(Ok((_, block))) => {
                diagnostics.parsed_block_count += 1;
                if matches!(
                    &block,
                    NifBlock::BSTriShape(_)
                        | NifBlock::BSDynamicTriShape(_)
                        | NifBlock::BSSubIndexTriShape(_)
                        | NifBlock::NiTriShape(_)
                ) {
                    diagnostics.geometry_block_count += 1;
                }
                block
            }
            Ok(Err(error)) => {
                let offset = match error {
                    nom_derive::nom::Err::Error(error) | nom_derive::nom::Err::Failure(error) => {
                        raw.len().saturating_sub(error.input.len())
                    }
                    nom_derive::nom::Err::Incomplete(_) => raw.len(),
                };
                diagnostics
                    .fallback_offsets
                    .entry(block_type.clone())
                    .or_default()
                    .push(offset);
                *diagnostics.fallback_blocks.entry(block_type).or_default() += 1;
                NifBlock::Unhandled
            }
            Err(_) => {
                diagnostics
                    .fallback_offsets
                    .entry(block_type.clone())
                    .or_default()
                    .push(usize::MAX);
                *diagnostics.fallback_blocks.entry(block_type).or_default() += 1;
                NifBlock::Unhandled
            }
        };
        blocks.push(block);
    }
    Ok((NifFile { header, blocks }, diagnostics))
}

fn parse_skyrim_header<'a>(bytes: &'a [u8], path: &Path) -> Result<(&'a [u8], NifHeader)> {
    let mut cursor = NifCursor::new(bytes, path);
    let file_desc = cursor.line()?;
    ensure!(
        file_desc.starts_with("Gamebryo File Format"),
        "unsupported NIF signature in {}",
        path.display()
    );
    let nif_version = cursor.u32()?;
    let endian_type = cursor.u8()?;
    ensure!(
        endian_type == 1,
        "big-endian NIF is not supported in {}",
        path.display()
    );
    let user_version = cursor.u32()?;
    let block_count = cursor.u32()?;
    ensure!(
        block_count <= 1_000_000,
        "NIF block count exceeds the safety limit in {}",
        path.display()
    );
    let bethesda_version = cursor.u32()?;
    let author = cursor.sized_string8_optional()?;
    let process_script = cursor.sized_string8_optional()?;
    let export_script = cursor.sized_string8_optional()?;
    let block_type_count = usize::from(cursor.u16()?);
    let mut block_types = Vec::with_capacity(block_type_count);
    for _ in 0..block_type_count {
        block_types.push(SizedString32(cursor.sized_string32()?));
    }
    let block_count_usize = usize::try_from(block_count).wrap_err("NIF block count overflow")?;
    let mut block_type_index = Vec::with_capacity(block_count_usize);
    for _ in 0..block_count_usize {
        block_type_index.push(cursor.u16()?);
    }
    let mut block_size_index = Vec::with_capacity(block_count_usize);
    for _ in 0..block_count_usize {
        block_size_index.push(cursor.u32()?);
    }
    let string_count = cursor.u32()?;
    ensure!(
        string_count <= 1_000_000,
        "NIF string count exceeds the safety limit in {}",
        path.display()
    );
    let string_max_size = cursor.u32()?;
    let mut strings = Vec::with_capacity(usize::try_from(string_count)?);
    for _ in 0..string_count {
        strings.push(SizedString32(cursor.sized_string32()?));
    }
    let group_count = cursor.u32()?;
    ensure!(
        group_count <= 1_000_000,
        "NIF group count exceeds the safety limit in {}",
        path.display()
    );
    let mut groups = Vec::with_capacity(usize::try_from(group_count)?);
    for _ in 0..group_count {
        groups.push(cursor.u32()?);
    }
    let remaining = &bytes[cursor.position..];
    Ok((
        remaining,
        NifHeader {
            file_desc: StringN { value: file_desc },
            nif_version: NifFileVersion(nif_version),
            endian_type: Endianess::Little,
            user_version,
            block_count,
            bethesda_version,
            author,
            process_script,
            export_script,
            max_filepath: None,
            block_types,
            block_type_index,
            block_size_index,
            string_count,
            string_max_size,
            strings,
            groups,
        },
    ))
}

struct NifCursor<'a> {
    bytes: &'a [u8],
    position: usize,
    path: &'a Path,
}

impl<'a> NifCursor<'a> {
    fn new(bytes: &'a [u8], path: &'a Path) -> Self {
        Self {
            bytes,
            position: 0,
            path,
        }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| color_eyre::eyre::eyre!("NIF offset overflow"))?;
        ensure!(
            end <= self.bytes.len(),
            "truncated NIF header at byte {} in {}",
            self.position,
            self.path.display()
        );
        let result = &self.bytes[self.position..end];
        self.position = end;
        Ok(result)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn line(&mut self) -> Result<String> {
        let Some(length) = self.bytes[self.position..]
            .iter()
            .position(|byte| *byte == b'\n')
        else {
            color_eyre::eyre::bail!(
                "NIF header line is not terminated in {}",
                self.path.display()
            );
        };
        let value = String::from_utf8_lossy(self.take(length)?).into_owned();
        self.take(1)?;
        Ok(value)
    }

    fn sized_string8_optional(&mut self) -> Result<Option<SizedString8>> {
        let length = usize::from(self.u8()?);
        let value = String::from_utf8_lossy(self.take(length)?)
            .trim_end_matches('\0')
            .to_owned();
        Ok((!value.is_empty()).then_some(SizedString8(value)))
    }

    fn sized_string32(&mut self) -> Result<String> {
        let length = usize::try_from(self.u32()?).wrap_err("NIF string length overflow")?;
        ensure!(
            length <= 16 * 1024 * 1024,
            "NIF string exceeds the safety limit in {}",
            self.path.display()
        );
        Ok(String::from_utf8_lossy(self.take(length)?)
            .trim_end_matches('\0')
            .to_owned())
    }
}

fn glb_bounds_from_bytes(glb: &[u8]) -> Result<shared::Bounds3> {
    ensure!(
        glb.len() >= 20 && &glb[..4] == b"glTF",
        "invalid GLB container"
    );
    let json_length = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
    ensure!(&glb[16..20] == b"JSON", "GLB JSON chunk is missing");
    let document: serde_json::Value = serde_json::from_slice(
        glb.get(20..20 + json_length)
            .ok_or_else(|| color_eyre::eyre::eyre!("truncated GLB JSON chunk"))?,
    )?;
    let nodes = document
        .get("nodes")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let roots = scene_roots(&document, nodes);
    let mut bounds = BoundsAccumulator::default();
    for root in roots {
        visit_node(&document, nodes, root, Mat4::IDENTITY, 0, &mut bounds)?;
    }
    bounds.finish()
}

fn scene_roots(document: &serde_json::Value, nodes: &[serde_json::Value]) -> Vec<usize> {
    if let Some(scenes) = document.get("scenes").and_then(serde_json::Value::as_array) {
        let scene_index = document
            .get("scene")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize;
        if let Some(scene_nodes) = scenes
            .get(scene_index)
            .and_then(|scene| scene.get("nodes"))
            .and_then(serde_json::Value::as_array)
        {
            return scene_nodes
                .iter()
                .filter_map(serde_json::Value::as_u64)
                .map(|index| index as usize)
                .collect();
        }
    }
    let mut children = vec![false; nodes.len()];
    for node in nodes {
        if let Some(indices) = node.get("children").and_then(serde_json::Value::as_array) {
            for index in indices.iter().filter_map(serde_json::Value::as_u64) {
                if let Some(child) = children.get_mut(index as usize) {
                    *child = true;
                }
            }
        }
    }
    children
        .iter()
        .enumerate()
        .filter_map(|(index, child)| (!child).then_some(index))
        .collect()
}

fn visit_node(
    document: &serde_json::Value,
    nodes: &[serde_json::Value],
    index: usize,
    parent: Mat4,
    depth: usize,
    bounds: &mut BoundsAccumulator,
) -> Result<()> {
    ensure!(depth <= nodes.len(), "cyclic glTF node hierarchy");
    let node = nodes
        .get(index)
        .ok_or_else(|| color_eyre::eyre::eyre!("glTF node {index} is out of range"))?;
    let transform = parent.mul(Mat4::from_node(node));
    if let Some(mesh_index) = node.get("mesh").and_then(serde_json::Value::as_u64) {
        accumulate_mesh(document, mesh_index as usize, transform, bounds)?;
    }
    if let Some(children) = node.get("children").and_then(serde_json::Value::as_array) {
        for child in children.iter().filter_map(serde_json::Value::as_u64) {
            visit_node(
                document,
                nodes,
                child as usize,
                transform,
                depth + 1,
                bounds,
            )?;
        }
    }
    Ok(())
}

fn accumulate_mesh(
    document: &serde_json::Value,
    mesh_index: usize,
    transform: Mat4,
    output: &mut BoundsAccumulator,
) -> Result<()> {
    let meshes = document
        .get("meshes")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| color_eyre::eyre::eyre!("GLB has no meshes"))?;
    let accessors = document
        .get("accessors")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| color_eyre::eyre::eyre!("GLB has no accessors"))?;
    let primitives = meshes
        .get(mesh_index)
        .and_then(|mesh| mesh.get("primitives"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| color_eyre::eyre::eyre!("mesh {mesh_index} has no primitives"))?;
    for primitive in primitives {
        let Some(accessor_index) = primitive
            .pointer("/attributes/POSITION")
            .and_then(serde_json::Value::as_u64)
        else {
            continue;
        };
        let accessor = accessors
            .get(accessor_index as usize)
            .ok_or_else(|| color_eyre::eyre::eyre!("POSITION accessor is out of range"))?;
        let min = json_vec3(accessor.get("min"))?;
        let max = json_vec3(accessor.get("max"))?;
        for x in [min[0], max[0]] {
            for y in [min[1], max[1]] {
                for z in [min[2], max[2]] {
                    output.include(transform.transform([x, y, z]));
                }
            }
        }
    }
    Ok(())
}

fn json_vec3(value: Option<&serde_json::Value>) -> Result<[f64; 3]> {
    let values = value
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| color_eyre::eyre::eyre!("POSITION accessor has no min/max"))?;
    ensure!(values.len() >= 3, "POSITION accessor min/max is not a vec3");
    Ok([
        values[0]
            .as_f64()
            .ok_or_else(|| color_eyre::eyre::eyre!("invalid bound"))?,
        values[1]
            .as_f64()
            .ok_or_else(|| color_eyre::eyre::eyre!("invalid bound"))?,
        values[2]
            .as_f64()
            .ok_or_else(|| color_eyre::eyre::eyre!("invalid bound"))?,
    ])
}

#[derive(Clone, Copy)]
struct Mat4([f64; 16]);

impl Mat4 {
    const IDENTITY: Self = Self([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);

    fn from_node(node: &serde_json::Value) -> Self {
        if let Some(matrix) = node.get("matrix").and_then(serde_json::Value::as_array)
            && matrix.len() == 16
        {
            let mut output = [0.0; 16];
            for (target, source) in output.iter_mut().zip(matrix) {
                *target = source.as_f64().unwrap_or(0.0);
            }
            return Self(output);
        }
        let t = array_or(node.get("translation"), [0.0, 0.0, 0.0]);
        let r = array_or(node.get("rotation"), [0.0, 0.0, 0.0, 1.0]);
        let s = array_or(node.get("scale"), [1.0, 1.0, 1.0]);
        let [x, y, z, w] = r;
        let mut matrix = [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
            0.0,
            2.0 * (x * y - z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + x * w),
            0.0,
            2.0 * (x * z + y * w),
            2.0 * (y * z - x * w),
            1.0 - 2.0 * (x * x + y * y),
            0.0,
            t[0],
            t[1],
            t[2],
            1.0,
        ];
        for row in 0..4 {
            matrix[row] *= s[0];
            matrix[4 + row] *= s[1];
            matrix[8 + row] *= s[2];
        }
        Self(matrix)
    }

    fn mul(self, rhs: Self) -> Self {
        let mut output = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                output[column * 4 + row] = (0..4)
                    .map(|axis| self.0[axis * 4 + row] * rhs.0[column * 4 + axis])
                    .sum();
            }
        }
        Self(output)
    }

    fn transform(self, point: [f64; 3]) -> [f64; 3] {
        [
            self.0[0] * point[0] + self.0[4] * point[1] + self.0[8] * point[2] + self.0[12],
            self.0[1] * point[0] + self.0[5] * point[1] + self.0[9] * point[2] + self.0[13],
            self.0[2] * point[0] + self.0[6] * point[1] + self.0[10] * point[2] + self.0[14],
        ]
    }
}

fn array_or<const N: usize>(value: Option<&serde_json::Value>, fallback: [f64; N]) -> [f64; N] {
    let Some(values) = value.and_then(serde_json::Value::as_array) else {
        return fallback;
    };
    std::array::from_fn(|index| {
        values
            .get(index)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(fallback[index])
    })
}

#[derive(Default)]
struct BoundsAccumulator {
    min: [f64; 3],
    max: [f64; 3],
    populated: bool,
}

impl BoundsAccumulator {
    fn include(&mut self, point: [f64; 3]) {
        if !self.populated {
            self.min = point;
            self.max = point;
            self.populated = true;
        } else {
            for (axis, value) in point.into_iter().enumerate() {
                self.min[axis] = self.min[axis].min(value);
                self.max[axis] = self.max[axis].max(value);
            }
        }
    }

    fn finish(self) -> Result<shared::Bounds3> {
        ensure!(self.populated, "GLB contains no bounded POSITION accessor");
        let bounds = shared::Bounds3 {
            min: self.min.map(|value| value as f32),
            max: self.max.map(|value| value as f32),
        };
        ensure!(
            bounds.is_finite_and_ordered(),
            "GLB contains invalid bounds"
        );
        Ok(bounds)
    }
}

fn rewrite_materials_and_texture_uris(
    glb: Vec<u8>,
    specular_textures: &[Option<String>],
    glb_output_path: &Path,
) -> Result<Vec<u8>> {
    ensure!(
        glb.len() >= 20 && &glb[..4] == b"glTF",
        "invalid GLB container"
    );
    let json_length = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
    ensure!(&glb[16..20] == b"JSON", "GLB JSON chunk is missing");
    let json_end = 20usize
        .checked_add(json_length)
        .ok_or_else(|| color_eyre::eyre::eyre!("GLB JSON range overflow"))?;
    let json_bytes = glb
        .get(20..json_end)
        .ok_or_else(|| color_eyre::eyre::eyre!("truncated GLB JSON chunk"))?;
    let mut document: serde_json::Value =
        serde_json::from_slice(json_bytes).wrap_err("NIF exporter produced invalid glTF JSON")?;
    if let Some(images) = document
        .get_mut("images")
        .and_then(|value| value.as_array_mut())
    {
        for image in images {
            if let Some(uri) = image.get_mut("uri").and_then(|value| value.as_str()) {
                let mut path = PathBuf::from(uri.replace('\\', "/"));
                if path.extension().is_some_and(|extension| {
                    extension.to_string_lossy().eq_ignore_ascii_case("dds")
                }) {
                    path.set_extension("ktx2");
                    image["uri"] = serde_json::Value::String(runtime_texture_uri(
                        glb_output_path,
                        &path.to_string_lossy().replace('\\', "/"),
                    ));
                }
            }
        }
    }
    rewrite_materials(&mut document, specular_textures, glb_output_path);
    let mut json = serde_json::to_vec(&document)?;
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    let suffix = glb
        .get(json_end..)
        .ok_or_else(|| color_eyre::eyre::eyre!("invalid GLB suffix"))?;
    let total_length = 20usize
        .checked_add(json.len())
        .and_then(|length| length.checked_add(suffix.len()))
        .ok_or_else(|| color_eyre::eyre::eyre!("GLB size overflow"))?;
    let mut output = Vec::with_capacity(total_length);
    output.extend_from_slice(&glb[..8]);
    let total_length = u32::try_from(total_length).wrap_err("GLB exceeds 4 GiB")?;
    let json_length = u32::try_from(json.len()).wrap_err("GLB JSON exceeds 4 GiB")?;
    output.extend_from_slice(&total_length.to_le_bytes());
    output.extend_from_slice(&json_length.to_le_bytes());
    output.extend_from_slice(b"JSON");
    output.extend_from_slice(&json);
    output.extend_from_slice(suffix);
    Ok(output)
}

fn runtime_texture_uri(glb_output_path: &Path, texture_path: &str) -> String {
    let components: Vec<_> = glb_output_path.components().collect();
    let Some(meshes_index) = components.iter().rposition(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("meshes")
    }) else {
        return texture_path.to_owned();
    };
    let directories_below_meshes = components
        .len()
        .saturating_sub(meshes_index)
        .saturating_sub(2);
    format!(
        "{}{}",
        "../".repeat(directories_below_meshes.saturating_add(1)),
        texture_path
    )
}

fn rewrite_materials(
    document: &mut serde_json::Value,
    specular_textures: &[Option<String>],
    glb_output_path: &Path,
) {
    let mut additions = Vec::new();
    if let Some(materials) = document
        .get_mut("materials")
        .and_then(serde_json::Value::as_array_mut)
    {
        for (index, material) in materials.iter_mut().enumerate() {
            let diffuse = material
                .pointer("/extensions/KHR_materials_pbrSpecularGlossiness/diffuseTexture")
                .cloned();
            let pbr = material
                .as_object_mut()
                .expect("glTF material must be an object")
                .entry("pbrMetallicRoughness")
                .or_insert_with(|| serde_json::json!({}));
            if let Some(diffuse) = diffuse {
                pbr["baseColorTexture"] = diffuse;
            }
            if let Some(Some(specular)) = specular_textures.get(index) {
                additions.push((
                    index,
                    runtime_texture_uri(glb_output_path, &with_ktx2_extension(specular)),
                ));
            }
            if let Some(extensions) = material
                .get_mut("extensions")
                .and_then(serde_json::Value::as_object_mut)
            {
                extensions.remove("KHR_materials_pbrSpecularGlossiness");
                if extensions.is_empty() {
                    material.as_object_mut().unwrap().remove("extensions");
                }
            }
        }
    }
    for (material_index, uri) in additions {
        let image_index = document
            .as_object_mut()
            .unwrap()
            .entry("images")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap()
            .len();
        document["images"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({ "uri": uri }));
        let texture_index = document
            .as_object_mut()
            .unwrap()
            .entry("textures")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .unwrap()
            .len();
        document["textures"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({ "source": image_index }));
        document["materials"][material_index]["pbrMetallicRoughness"]["metallicRoughnessTexture"] =
            serde_json::json!({ "index": texture_index });
    }
    if let Some(extensions_used) = document
        .get_mut("extensionsUsed")
        .and_then(serde_json::Value::as_array_mut)
    {
        extensions_used.retain(|extension| extension != "KHR_materials_pbrSpecularGlossiness");
        if extensions_used.is_empty() {
            document.as_object_mut().unwrap().remove("extensionsUsed");
        }
    }
}

fn with_ktx2_extension(uri: &str) -> String {
    let mut path = PathBuf::from(uri.replace('\\', "/"));
    path.set_extension("ktx2");
    path.to_string_lossy().replace('\\', "/")
}

fn find_skeleton(nif_path: &Path) -> Option<PathBuf> {
    let parent = nif_path.parent()?;
    let mut candidates = vec![
        parent.join("skeleton.nif"),
        parent.join("skeleton_female.nif"),
        parent.join("character assets").join("skeleton.nif"),
    ];
    if let Some(grandparent) = parent.parent() {
        candidates.extend([
            grandparent.join("skeleton.nif"),
            grandparent.join("character assets").join("skeleton.nif"),
        ]);
    }
    if let Some((actors_root, actor_name)) = actor_root(nif_path) {
        let actor_dir = actors_root.join(actor_name);
        candidates.extend([
            actor_dir.join("character assets").join("skeleton.nif"),
            actor_dir
                .join("character assets female")
                .join("skeleton_female.nif"),
        ]);
        if let Some(found) = WalkDir::new(&actor_dir)
            .max_depth(4)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.into_path())
            .find(|path| {
                path.is_file()
                    && path.file_name().is_some_and(|name| {
                        name.to_string_lossy()
                            .to_ascii_lowercase()
                            .starts_with("skeleton")
                            && path.extension().is_some_and(|ext| {
                                ext.to_string_lossy().eq_ignore_ascii_case("nif")
                            })
                    })
            })
        {
            candidates.push(found);
        }
    }
    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn actor_root(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let components: Vec<_> = path.components().collect();
    let index = components.iter().position(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("actors")
    })?;
    let actor = components.get(index + 1)?.as_os_str();
    let mut root = PathBuf::new();
    for component in &components[..=index] {
        root.push(component.as_os_str());
    }
    Some((root, PathBuf::from(actor)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_nif_without_panicking() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("bad.nif");
        let output = dir.path().join("bad.glb");
        fs::write(&input, b"not a nif").unwrap();
        assert!(MeshConverter::convert_nif_to_glb(&input, &output).is_err());
        assert!(!output.exists());
    }

    #[test]
    fn derives_actor_root_case_insensitively() {
        let (root, actor) = actor_root(Path::new(
            "vfs/Meshes/Actors/Dragon/character assets/dragon.nif",
        ))
        .unwrap();
        assert_eq!(root, PathBuf::from("vfs/Meshes/Actors"));
        assert_eq!(actor, PathBuf::from("Dragon"));
    }

    #[test]
    fn rewrites_glb_dds_uris_to_pipeline_ktx2_paths() {
        let mut json =
            br#"{"asset":{"version":"2.0"},"images":[{"uri":"textures/a.dds"}]}"#.to_vec();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let total = 20 + json.len();
        let mut glb = b"glTF".to_vec();
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);
        let rewritten =
            rewrite_materials_and_texture_uris(glb, &[], Path::new("meshes/a.glb")).unwrap();
        assert!(rewritten.windows(6).any(|window| window == b"a.ktx2"));
    }

    #[test]
    fn maps_nif_materials_to_core_metallic_roughness() {
        let mut json = br#"{"asset":{"version":"2.0"},"extensionsUsed":["KHR_materials_pbrSpecularGlossiness"],"images":[{"uri":"textures/a.dds"}],"textures":[{"source":0}],"materials":[{"extensions":{"KHR_materials_pbrSpecularGlossiness":{"diffuseTexture":{"index":0}}}}]}"#.to_vec();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let total = 20 + json.len();
        let mut glb = b"glTF".to_vec();
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);
        let rewritten = rewrite_materials_and_texture_uris(
            glb,
            &[Some("textures/a_s.dds".to_owned())],
            Path::new("meshes/a.glb"),
        )
        .unwrap();
        let length = u32::from_le_bytes(rewritten[12..16].try_into().unwrap()) as usize;
        let document: serde_json::Value =
            serde_json::from_slice(&rewritten[20..20 + length]).unwrap();
        assert_eq!(
            document["materials"][0]["pbrMetallicRoughness"]["baseColorTexture"]["index"],
            0
        );
        assert_eq!(
            document["materials"][0]["pbrMetallicRoughness"]["metallicRoughnessTexture"]["index"],
            1
        );
        assert_eq!(document["images"][1]["uri"], "../textures/a_s.ktx2");
        assert!(document.get("extensionsUsed").is_none());
    }

    #[test]
    fn extracts_bounds_with_node_transform() {
        let mut json = br#"{
            "asset":{"version":"2.0"},
            "scene":0,
            "scenes":[{"nodes":[0]}],
            "nodes":[{"mesh":0,"translation":[10,20,30],"scale":[2,3,4]}],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],
            "accessors":[{"min":[-1,-2,-3],"max":[1,2,3]}]
        }"#
        .to_vec();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let total = 20 + json.len();
        let mut glb = b"glTF".to_vec();
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);
        let bounds = glb_bounds_from_bytes(&glb).unwrap();
        assert_eq!(bounds.min, [8.0, 14.0, 18.0]);
        assert_eq!(bounds.max, [12.0, 26.0, 42.0]);
    }

    #[test]
    fn lists_external_glb_textures_deterministically() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mesh.glb");
        let mut json = br#"{
            "asset":{"version":"2.0"},
            "images":[
                {"uri":"../textures/B.ktx2"},
                {"bufferView":0,"mimeType":"image/png"},
                {"uri":"../textures/a.ktx2"},
                {"uri":"../textures/A.ktx2"}
            ]
        }"#
        .to_vec();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let total = 20 + json.len();
        let mut glb = b"glTF".to_vec();
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);
        fs::write(&path, glb).unwrap();

        assert_eq!(
            MeshConverter::glb_texture_uris(&path).unwrap(),
            vec!["../textures/a.ktx2", "../textures/B.ktx2"]
        );
    }
}
