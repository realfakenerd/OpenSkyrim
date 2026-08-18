use project_wormhole_ba2::dev::{ensure_texture_parent, normalize_esm_path};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct BSShaderTextureSet {
    pub diffuse: Option<String>,
    pub normal: Option<String>,
    pub glow: Option<String>,
    pub specular: Option<String>,
}

impl Parse<&[u8]> for BSShaderTextureSet {
    fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, string_count) = le_u32(i)?;
        let (i, textures) = count(SizedString32::parse, string_count as usize)(i)?;

        let fixed: Vec<Option<String>> = textures
            .into_iter()
            .map(|texture| {
                let mut fixed_path = normalize_esm_path(&texture.0);
                ensure_texture_parent(&mut fixed_path);
                (fixed_path != "textures/").then_some(fixed_path)
            })
            .collect();

        // BSShaderTextureSet has semantic slots; diffuse names commonly end
        // in "rocks01.dds" rather than "_d.dds", so suffix classification
        // silently discarded most official Skyrim base-color textures.
        let tset = BSShaderTextureSet {
            diffuse: fixed.first().cloned().flatten(),
            normal: fixed.get(1).cloned().flatten(),
            glow: fixed.get(2).cloned().flatten(),
            specular: fixed.get(7).cloned().flatten(),
        };

        Ok((i, tset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_skyrim_texture_slots_without_filename_suffix_guessing() {
        let values = [
            "landscape/rocks01.dds",
            "landscape/rocks01_n.dds",
            "landscape/rocks01_g.dds",
            "",
            "",
            "",
            "",
            "landscape/rocks01_s.dds",
            "",
        ];
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(values.len() as u32).to_le_bytes());
        for value in values {
            bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
            bytes.extend_from_slice(value.as_bytes());
        }
        let (_, set) = BSShaderTextureSet::parse(&bytes).unwrap();
        assert_eq!(
            set.diffuse.as_deref(),
            Some("textures/landscape/rocks01.dds")
        );
        assert_eq!(
            set.normal.as_deref(),
            Some("textures/landscape/rocks01_n.dds")
        );
        assert_eq!(
            set.glow.as_deref(),
            Some("textures/landscape/rocks01_g.dds")
        );
        assert_eq!(
            set.specular.as_deref(),
            Some("textures/landscape/rocks01_s.dds")
        );
    }
}
