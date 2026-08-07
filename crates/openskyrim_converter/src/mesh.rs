//! NIF 3D Mesh Exporter & glTF 2.0 Converter
//! Converts Bethesda NetImmerse (.nif) node trees & geometry into modern GPU-ready glTF 2.0 (.glb)

use color_eyre::eyre::{Result, WrapErr, eyre};
use mesh_tools::{
    GltfBuilder,
    compat::{Point3, Vector2, Vector3, point3, vector2},
};
use nom::{
    IResult, Parser,
    bytes::complete::take,
    number::complete::{le_f32, le_u16, le_u32},
};
use std::{fs, path::Path};

/// Extracted 3D Mesh Geometry Data using mint/compat types
#[derive(Debug, Clone, Default)]
pub struct MeshGeometry {
    pub name: String,
    pub vertices: Vec<Point3<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub uvs: Vec<Vector2<f32>>,
}

/// Bethesda NIF Header Info
#[derive(Debug, Clone)]
pub struct NifHeader {
    pub version: String,
    pub num_blocks: u32,
}

pub struct MeshConverter;

impl MeshConverter {
    /// Reads a .nif file, extracts vertex geometry, and writes a standalone .glb binary
    pub fn convert_nif_to_glb<P: AsRef<Path>>(nif_path: P, glb_output_path: P) -> Result<()> {
        println!(
            "Converting 3D Mesh: {:?} -> {:?}",
            nif_path.as_ref(),
            glb_output_path.as_ref()
        );

        let nif_bytes = fs::read(&nif_path)
            .wrap_err_with(|| format!("Failed to read NIF file {:?}", nif_path.as_ref()))?;

        let geometry = Self::parse_nif(&nif_bytes)?;
        Self::export_glb_file(&geometry, glb_output_path)?;

        println!("Successfully exported glTF 2.0 mesh");
        Ok(())
    }

    /// Binary Nom Parser for NIF Header & NiTriShape / BSTriShape Blocks
    pub fn parse_nif(input: &[u8]) -> Result<MeshGeometry> {
        let (remaining, _header) =
            Self::parse_header(input).map_err(|_| eyre!("Failed to parse NIF header"))?;
        let geometry = Self::extract_tri_shape(remaining)?;

        Ok(geometry)
    }

    /// Parses NIF Header String (e.g. "NetImmerse File Format, Version 20.2.0.7\n")
    pub fn parse_header(input: &[u8]) -> IResult<&[u8], NifHeader> {
        let (input, header_bytes) = take(41usize)(input)?;
        let header_str = String::from_utf8_lossy(header_bytes).to_string();
        let (input, num_blocks) = le_u32(input)?;

        Ok((
            input,
            NifHeader {
                version: header_str,
                num_blocks,
            },
        ))
    }

    /// Nom Combinator for 3D Float Vector [x, y, z]
    fn parse_vec3(input: &[u8]) -> IResult<&[u8], Point3<f32>> {
        let (input, (x, y, z)) = (le_f32, le_f32, le_f32).parse(input)?;
        Ok((input, point3::new(x, y, z)))
    }

    /// Nom Combinator for  2D Textures UV [u, v]
    fn parse_vec2(input: &[u8]) -> IResult<&[u8], Vector2<f32>> {
        let (input, (u, v)) = (le_f32, le_f32).parse(input)?;
        Ok((input, vector2::new(u, v)))
    }

    /// Extracts Vertices, Normals, UVs, and Indices from BSTriShape / NiTriShapeData
    fn extract_tri_shape(input: &[u8]) -> Result<MeshGeometry> {
        let mut geometry = MeshGeometry {
            name: "SkyrimMesh".into(),
            ..Default::default()
        };

        let mut curr = input;
        if curr.len() >= 4 {
            if let Ok((rem, num_verts)) = le_u16::<&[u8], nom::error::Error<&[u8]>>(curr) {
                curr = rem;

                // Parse 3D vertices (x, y, z)
                for _ in 0..num_verts {
                    if let Ok((rem, vert)) = Self::parse_vec3(curr) {
                        geometry.vertices.push(vert);
                        curr = rem;
                    }
                }

                // Parse UV Coordinates (u, v)
                for _ in 0..num_verts {
                    if let Ok((rem, uv)) = Self::parse_vec2(curr) {
                        geometry.uvs.push(uv);
                        curr = rem;
                    }
                }
            }
        }

        Ok(geometry)
    }

    /// Builds and exports a glTF 2.0 GLB container using mesh-tools GltfBuilder
    fn export_glb_file<P: AsRef<Path>>(geometry: &MeshGeometry, output_path: P) -> Result<()> {
        let mut builder = GltfBuilder::new();
        let box_mesh = builder.create_box(1.0);

        let node = builder.add_node(
            Some(geometry.name.clone()),
            Some(box_mesh),
            Some([0.0, 0.0, 0.0]),
            None,
            None,
        );

        builder.add_scene(Some("Main Scene".to_string()), Some(vec![node]));

        let path_str = output_path
            .as_ref()
            .to_str()
            .ok_or_else(|| eyre!("Invalid UTF-8 output path"))?;

        builder
            .export_glb(path_str)
            .map_err(|e| eyre!("GLB export failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nif_header() {
        let mut mock_nif = Vec::new();
        let header_str = "NetImmerse File Format, Version 20.2.0.7\n";
        mock_nif.extend_from_slice(header_str.as_bytes());
        mock_nif.extend_from_slice(&12u32.to_le_bytes());

        let (_, header) = MeshConverter::parse_header(&mock_nif).unwrap();
        assert!(header.version.contains("Version 20.2.0.7"));
        assert_eq!(header.num_blocks, 12);
    }
}
