use color_eyre::{
    Result,
    eyre::{WrapErr, ensure},
};
use project_wormhole_nif::nif_file::{NifFile, nif_to_model};
use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub struct MeshConverter;

impl MeshConverter {
    pub fn dependency_paths(nif_path: &Path) -> Vec<PathBuf> {
        find_skeleton(nif_path).into_iter().collect()
    }

    pub fn convert_nif_to_glb<P: AsRef<Path>>(nif_path: P, glb_output_path: P) -> Result<()> {
        let nif_path = nif_path.as_ref();
        let path_string = nif_path
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("NIF path is not valid UTF-8"))?;
        let nif = catch_unwind(AssertUnwindSafe(|| NifFile::open(path_string)))
            .map_err(|_| color_eyre::eyre::eyre!("NIF parser rejected {}", nif_path.display()))?
            .wrap_err_with(|| format!("failed to parse {}", nif_path.display()))?;
        let skeleton = if nif.has_skeleton() {
            let skeleton_path = find_skeleton(nif_path).ok_or_else(|| {
                color_eyre::eyre::eyre!(
                    "skinned NIF requires a skeleton, but none was found near {}",
                    nif_path.display()
                )
            })?;
            let skeleton_path_string = skeleton_path
                .to_str()
                .ok_or_else(|| color_eyre::eyre::eyre!("skeleton path is not valid UTF-8"))?;
            Some(
                catch_unwind(AssertUnwindSafe(|| NifFile::open(skeleton_path_string)))
                    .map_err(|_| {
                        color_eyre::eyre::eyre!(
                            "NIF parser rejected skeleton {}",
                            skeleton_path.display()
                        )
                    })?
                    .wrap_err_with(|| {
                        format!("failed to parse skeleton {}", skeleton_path.display())
                    })?,
            )
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
        let glb = rewrite_materials_and_texture_uris(model.to_glb(name), &specular_textures)?;
        ensure!(
            glb.len() >= 12 && &glb[..4] == b"glTF",
            "NIF exporter produced an invalid GLB header"
        );
        let output = glb_output_path.as_ref();
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, glb).wrap_err_with(|| format!("failed to write {}", output.display()))
    }
}

fn rewrite_materials_and_texture_uris(
    glb: Vec<u8>,
    specular_textures: &[Option<String>],
) -> Result<Vec<u8>> {
    ensure!(
        glb.len() >= 20 && &glb[..4] == b"glTF",
        "invalid GLB container"
    );
    let json_length = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
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
                    image["uri"] =
                        serde_json::Value::String(path.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    rewrite_materials(&mut document, specular_textures);
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

fn rewrite_materials(document: &mut serde_json::Value, specular_textures: &[Option<String>]) {
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
                additions.push((index, with_ktx2_extension(specular)));
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
        let rewritten = rewrite_materials_and_texture_uris(glb, &[]).unwrap();
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
        let rewritten =
            rewrite_materials_and_texture_uris(glb, &[Some("textures/a_s.dds".to_owned())])
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
        assert_eq!(document["images"][1]["uri"], "textures/a_s.ktx2");
        assert!(document.get("extensionsUsed").is_none());
    }
}
