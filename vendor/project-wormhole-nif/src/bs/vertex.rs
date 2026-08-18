use gltf::image::Data;
use project_wormhole_shared::glam::{U16Vec2, U8Vec3, U8Vec4, Vec2, Vec3, Vec4};

use super::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct BSVertexData {
    pub position: Option<Vec3>,
    pub bitangent_x: Option<f32>,
    pub uv: Option<Vec2>,
    pub normal: Option<Vec3>,
    pub bitangent_y: Option<f32>,
    pub tangent: Option<U8Vec3>,
    pub bitangent_z: Option<f32>,
    pub vertex_colors: Option<U8Vec4>,
    pub bone_weights: Option<Vec4>,
    pub bone_indices: Option<U8Vec4>,
    pub eye_data: Option<f32>,
}

// pub position: Option<Vec3<f32>>,
//     pub bitangent_x: Option<f32>,
//     pub uv: Option<Vec2<f32>>,
//     pub normal: Option<Vec3<f32>>,
//     pub bitangent_y: Option<f32>,
//     pub tangent: Option<Vec3<u8>>,
//     pub bitangent_z: Option<f32>,
//     pub vertex_colors: Option<Vec4<u8>>,
//     pub bone_weights: Option<Vec4<f32>>,
//     pub bone_indices: Option<Vec4<u8>>,
//     pub eye_data: Option<f32>,

impl BSVertexData {
    pub fn parse_with_flags<'a>(
        i: &'a [u8],
        flags: &BSVertexDesc,
    ) -> IResult<&'a [u8], Self, nom::error::Error<&'a [u8]>> {
        // Initialize the data to parse
        let mut data = i;

        // Initialize the vertex data as not existing
        let mut position = None;
        let mut bitangent_x = None;
        let mut uv = None;
        let mut normal = None;
        let mut bitangent_y = None;
        let mut tangent = None;
        let mut bitangent_z = None;
        let mut vertex_colors = None;
        let mut bone_weights = None;
        let mut bone_indices = None;
        let mut eye_data = None;

        // Check if data contains vertex positions
        if flags.has_vertex() {
            // Skyrim Special Edition stores BSTriShape positions and the X
            // bitangent as full-precision floats. The descriptor's legacy
            // precision bit is not reliable for distinguishing Skyrim from
            // the half-float Fallout 4 layout.
            let (i, positions) = parse_vec3(data)?;
            let (i, bita_x) = le_f32(i)?;
            position = Some(positions);
            bitangent_x = Some(bita_x);
            data = i;
        }

        // Check if data contains UVs
        if flags.has_uv() {
            // Parse the UVs, always half precision
            //let (i, value) = U16Vec2::parse(data)?;

            let (i, v1) = le_u16(data)?;
            let (i, v2) = le_u16(i)?;

            // Convert the half precision UVs to full precision
            let value = Vec2 {
                x: f16::from_le_bytes(v1.to_le_bytes()).to_f32(),
                y: f16::from_le_bytes(v2.to_le_bytes()).to_f32(),
            };

            // Set the UVs
            uv = Some(value);

            // Set the data to the next position
            data = i;
        }

        // Check if data contains normals
        if flags.has_normal() {
            // Parse the normals
            //let (i, packed_normals) = Vec3::<u8>::parse(data)?;

            let (i, v1) = le_u8(data)?;
            let (i, v2) = le_u8(i)?;
            let (i, v3) = le_u8(i)?;

            let packed_normals = U8Vec3 {
                x: v1,
                y: v2,
                z: v3,
            };

            // Parse the bitangent y
            let (i, bita_y) = le_u8(i)?;

            // Set the normals, converting from packed u8 to f32
            normal = Some(unpack_u8_normal_vector(packed_normals));

            // Set the bitangent y
            bitangent_y = Some(bita_y as f32);

            // Set the data to the next position
            data = i;

            // Check if data contains tangents
            if flags.has_tangent() {
                // Parse the tangents

                let (i, v1) = le_u8(data)?;
                let (i, v2) = le_u8(i)?;
                let (i, v3) = le_u8(i)?;

                // Parse the bitangent z
                let (i, bita_z) = le_u8(i)?;

                // Set the tangents
                tangent = Some(U8Vec3 {
                    x: v1,
                    y: v2,
                    z: v3,
                });

                // Set the bitangent z
                bitangent_z = Some(bita_z as f32);

                // Set the data to the next position
                data = i;
            }
        }

        // Check if data contains vertex colors
        // TODO: Unsure what they are using vertex colors for
        if flags.has_vertex_colors() {
            // Parse the vertex colors
            //let (i, value) = Vec4::<u8>::parse(data)?;

            let (i, v1) = le_u8(data)?;
            let (i, v2) = le_u8(i)?;
            let (i, v3) = le_u8(i)?;
            let (i, v4) = le_u8(i)?;

            // Set the vertex colors
            vertex_colors = Some(U8Vec4 {
                x: v1,
                y: v2,
                z: v3,
                w: v4,
            });

            // Set the data to the next position
            data = i;
        }

        // Check if data contains skin data
        if flags.has_skin_data() {
            // Parse the bone weights, always half precision
            //let (i, bw) = Vec4::<u16>::parse(data)?;

            let (i, v1) = le_u16(data)?;
            let (i, v2) = le_u16(i)?;
            let (i, v3) = le_u16(i)?;
            let (i, v4) = le_u16(i)?;

            // Convert the half precision bone weights to full precision
            let value = Vec4::new(
                f16::from_le_bytes(v1.to_le_bytes()).to_f32(),
                f16::from_le_bytes(v2.to_le_bytes()).to_f32(),
                f16::from_le_bytes(v3.to_le_bytes()).to_f32(),
                f16::from_le_bytes(v4.to_le_bytes()).to_f32(),
            );

            // Set the bone weights
            bone_weights = Some(value);

            // Set the data to the next position
            data = i;

            // Parse the bone indices
            //let (i, bid) = Vec4::<u8>::parse(data)?;

            let (i, v1) = le_u8(data)?;
            let (i, v2) = le_u8(i)?;
            let (i, v3) = le_u8(i)?;
            let (i, v4) = le_u8(i)?;

            // Set the bone indices
            bone_indices = Some(U8Vec4::new(v1, v2, v3, v4));

            // Set the data to the next position
            data = i;
        }

        // Check if data contains eye data
        if flags.has_eye_data() {
            // Parse the eye data
            let (i, eye) = le_f32(data)?;

            // Set the eye data
            eye_data = Some(eye);

            // Set the data to the next position
            data = i;
        }

        // Return leftover data and the parsed vertex data
        Ok((
            data,
            BSVertexData {
                position,
                bitangent_x,
                uv,
                normal,
                bitangent_y,
                tangent,
                bitangent_z,
                vertex_colors,
                bone_weights,
                bone_indices,
                eye_data,
            },
        ))
    }
}

#[derive(NomLE, Clone)]
pub struct BSVertexDesc {
    pub data: u64,

    #[nom(Value = "((data & 0xF) >> 0x00).try_into().unwrap()")]
    pub vertex_data_size: u8,

    #[nom(Value = "((data & 0xF0) >> 0x04).try_into().unwrap()")]
    pub dynamic_vertex_size: u8,

    #[nom(Value = "((data & 0xF00) >> 0x08).try_into().unwrap()")]
    pub uv1_offset: u8,

    #[nom(Value = "((data & 0xF000) >> 0x0C).try_into().unwrap()")]
    pub uv2_offset: u8,

    #[nom(Value = "((data & 0xF0000) >> 0x10).try_into().unwrap()")]
    pub normal_offset: u8,

    #[nom(Value = "((data & 0xF00000) >> 0x14).try_into().unwrap()")]
    pub tangent_offset: u8,

    #[nom(Value = "((data & 0xF000000) >> 0x18).try_into().unwrap()")]
    pub color_offset: u8,

    #[nom(Value = "((data & 0xF0000000) >> 0x1C).try_into().unwrap()")]
    pub skinning_data_offset: u8,

    #[nom(Value = "((data & 0xF00000000) >> 0x20).try_into().unwrap()")]
    pub landscape_data_offset: u8,

    #[nom(Value = "((data & 0xF000000000) >> 0x24).try_into().unwrap()")]
    pub eye_data_offset: u8,

    #[nom(Value = "((data & 0xFFF00000000000) >> 0x2C).try_into().unwrap()")]
    pub flags: u16,
}

impl BSVertexDesc {
    pub fn full_prescision(&self) -> bool {
        self.flags & 0x0400 != 0
    }

    pub fn has_vertex(&self) -> bool {
        self.flags & 0x0001 != 0
    }

    pub fn has_tangent(&self) -> bool {
        self.flags & 0x0010 != 0
    }

    pub fn has_uv(&self) -> bool {
        self.flags & 0x0002 != 0
    }

    pub fn has_normal(&self) -> bool {
        self.flags & 0x0008 != 0
    }

    pub fn has_vertex_colors(&self) -> bool {
        self.flags & 0x0020 != 0
    }

    pub fn has_skin_data(&self) -> bool {
        self.flags & 0x0040 != 0
    }

    pub fn has_eye_data(&self) -> bool {
        self.flags & 0x0100 != 0
    }
}

impl std::fmt::Debug for BSVertexDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BSVertexDesc {{ full precision: {}, vertex: {}, tangent: {}, uv: {}, normal: {}, vertex colors: {}, skin data: {}, eye data: {} }}",
            self.full_prescision(),
            self.has_vertex(),
            self.has_tangent(),
            self.has_uv(),
            self.has_normal(),
            self.has_vertex_colors(),
            self.has_skin_data(),
            self.has_eye_data(),
        )
    }
}

/// Dumb way to save a little storage space, quality of games will suffer for it.
/// Makes shading in games have artifacts
#[inline]
pub fn unpack_u8_normal(value: u8) -> f32 {
    value as f32 / 255.0 * 2.0 - 1.0
}

/// Convert a packed U8Vec3 to a BSVec3
#[inline]
pub fn unpack_u8_normal_vector(value: U8Vec3) -> Vec3 {
    Vec3 {
        x: unpack_u8_normal(value.x),
        y: unpack_u8_normal(value.y),
        z: unpack_u8_normal(value.z),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skyrim_positions_are_full_precision_without_legacy_flag() {
        let descriptor = BSVertexDesc {
            data: 0,
            vertex_data_size: 4,
            dynamic_vertex_size: 0,
            uv1_offset: 0,
            uv2_offset: 0,
            normal_offset: 0,
            tangent_offset: 0,
            color_offset: 0,
            skinning_data_offset: 0,
            landscape_data_offset: 0,
            eye_data_offset: 0,
            flags: 0x0001,
        };
        assert!(!descriptor.full_prescision());
        let bytes = [1.25_f32, -2.5, 300.0, 0.75]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let (_, vertex) = BSVertexData::parse_with_flags(&bytes, &descriptor).unwrap();
        assert_eq!(vertex.position, Some(Vec3::new(1.25, -2.5, 300.0)));
        assert_eq!(vertex.bitangent_x, Some(0.75));
    }
}
