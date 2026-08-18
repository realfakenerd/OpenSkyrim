use project_wormhole_ba2::dev::Bounds;
use project_wormhole_shared::glam::{U16Vec3, U8Vec4, Vec2, Vec3, Vec4};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct BSTriShape {
    pub av: NiAVObject,
    pub bounding_sphere: Bounds,
    pub skin: u32,
    pub shader_property: u32,
    pub alpha_property: u32,
    pub vertex_desc: BSVertexDesc,
    pub num_triangles: u32,
    pub num_vertices: u16,
    pub data_size: u32,
    pub vertex_data: Vec<BSVertexData>,
    pub triangles: Vec<U16Vec3>,
}

/// Skyrim Special Edition appends full-precision dynamic positions to the
/// regular BSTriShape payload.
#[derive(Debug, Clone)]
pub struct BSDynamicTriShape {
    pub bs_tri_shape: BSTriShape,
    pub dynamic_data_size: u32,
    pub dynamic_vertices: Vec<Vec4>,
}

impl Parse<&[u8]> for BSDynamicTriShape {
    fn parse(i: &[u8]) -> IResult<&[u8], Self, nom::error::Error<&[u8]>> {
        // Official SSE dynamic shapes carry an additional two-float min/max
        // extension after NiBound which is not present on ordinary BSTriShape.
        let (i, mut bs_tri_shape) = parse_tri_shape(i, 8)?;
        let (mut data, dynamic_data_size) = le_u32(i)?;
        let vertex_count = dynamic_data_size as usize / 16;
        if dynamic_data_size as usize % 16 != 0
            || vertex_count != bs_tri_shape.num_vertices as usize
        {
            return Err(nom::Err::Failure(nom::error::Error::new(
                data,
                nom::error::ErrorKind::Verify,
            )));
        }
        let mut dynamic_vertices = Vec::with_capacity(vertex_count);
        for vertex in &mut bs_tri_shape.vertex_data {
            let (i, x) = le_f32(data)?;
            let (i, y) = le_f32(i)?;
            let (i, z) = le_f32(i)?;
            let (i, w) = le_f32(i)?;
            data = i;
            let position = Vec4::new(x, y, z, w);
            vertex.position = Some(position.truncate());
            dynamic_vertices.push(position);
        }
        Ok((
            data,
            Self {
                bs_tri_shape,
                dynamic_data_size,
                dynamic_vertices,
            },
        ))
    }
}

impl Parse<&[u8]> for BSTriShape {
    fn parse(i: &[u8]) -> IResult<&[u8], Self, nom::error::Error<&[u8]>> {
        // Most SSE files use NiBound (16 bytes), while a sizeable official
        // subset carries the additional 8-byte min/max extension. Select the
        // layout by validating the declared geometry payload size.
        parse_tri_shape(i, 0).or_else(|_| parse_tri_shape(i, 8))
    }
}

fn parse_tri_shape(
    i: &[u8],
    bound_extension_size: usize,
) -> IResult<&[u8], BSTriShape, nom::error::Error<&[u8]>> {
    let (i, av) = NiAVObject::parse(i)?;
    let (i, bounding_sphere) = Bounds::parse(i)?;
    let (i, _) = take(bound_extension_size)(i)?;
    let (i, skin) = le_u32(i)?;
    let (i, shader_property) = le_u32(i)?;
    let (i, alpha_property) = le_u32(i)?;
    let (i, vertex_desc) = BSVertexDesc::parse(i)?;
    let (i, num_triangles) = le_u16(i)?;
    let num_triangles = u32::from(num_triangles);
    let (i, num_vertices) = le_u16(i)?;
    let (i, data_size) = le_u32(i)?;
    let expected_data_size = usize::from(vertex_desc.vertex_data_size)
        .saturating_mul(4)
        .saturating_mul(usize::from(num_vertices))
        .saturating_add((num_triangles as usize).saturating_mul(6));
    if data_size > 0 && data_size as usize != expected_data_size {
        return Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }
    let mut vertex_data = Vec::new();
    let mut data = i;

    let triangles = if data_size == 0 {
        vertex_data.resize_with(num_vertices as usize, BSVertexData::default);
        Vec::new()
    } else {
        let vertex_stride = usize::from(vertex_desc.vertex_data_size).saturating_mul(4);
        for _index in 0..num_vertices {
            let (i, raw_vertex) = take(vertex_stride)(data)?;
            let (_, bsdata) = BSVertexData::parse_with_flags(raw_vertex, &vertex_desc)?;
            data = i;
            vertex_data.push(bsdata);
        }
        let (i, triangles) = count(parse_u16_vec3, num_triangles as usize)(data)?;
        data = i;
        triangles
    };

    // SSE stores a secondary particle payload after ordinary geometry. Its
    // size is expressed as a count of 16-bit values.
    let data = if bound_extension_size > 0 && data.len() >= 4 {
        let (data, particle_data_size) = le_u32(data)?;
        let particle_bytes = (particle_data_size as usize).saturating_mul(2);
        let (data, _) = take(particle_bytes)(data)?;
        data
    } else {
        data
    };

    Ok((
        data,
        BSTriShape {
            av,
            bounding_sphere,
            skin,
            shader_property,
            alpha_property,
            vertex_desc,
            num_triangles,
            num_vertices,
            data_size,
            vertex_data,
            triangles,
        },
    ))
}

impl BSTriShape {
    // Get all indices from the triangles
    pub fn get_indices(&self) -> Vec<u16> {
        // Initialize indices vector
        let mut indices = Vec::new();

        // Iterate through all triangles
        for triangle in &self.triangles {
            // Push the indices to the vector
            indices.push(triangle.x);
            indices.push(triangle.y);
            indices.push(triangle.z);
        }

        // Return the indices
        indices
    }

    pub fn get_indices_as_bytes(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        for index in self.get_indices() {
            indices.extend_from_slice(&index.to_le_bytes());
        }
        indices
    }

    // Get the min and max vertices from the vertex data
    pub fn get_vertices_min_max(&self) -> (Vec3, Vec3) {
        // Initialize min and max to 0.0
        let mut min = Vec3::ZERO;
        let mut max = Vec3::ZERO;

        // Iterate through all vertices
        for vertex in &self.vertex_data {
            // If the vertex has a half vertex
            if let Some(v) = &vertex.position {
                // If min and max are still 0.0, set them to the first value
                if min.x == 0.0 && max.x == 0.0 {
                    let val = v.x;
                    min.x = val;
                    max.x = val;
                }

                // If min and max are still 0.0, set them to the first value
                if min.y == 0.0 && max.y == 0.0 {
                    let val = v.y;
                    min.y = val;
                    max.y = val;
                }

                // If min and max are still 0.0, set them to the first value
                if min.z == 0.0 && max.z == 0.0 {
                    let val = v.z;
                    min.z = val;
                    max.z = val;
                }

                // If the value is less than the min, set the min to the value
                if v.x < min.x {
                    min.x = v.x;
                }

                // If the value is less than the min, set the min to the value
                if v.y < min.y {
                    min.y = v.y;
                }

                // If the value is less than the min, set the min to the value
                if v.z < min.z {
                    min.z = v.z;
                }

                // If the value is greater than the max, set the max to the value
                if v.x > max.x {
                    max.x = v.x;
                }

                // If the value is greater than the max, set the max to the value
                if v.y > max.y {
                    max.y = v.y;
                }

                // If the value is greater than the max, set the max to the value
                if v.z > max.z {
                    max.z = v.z;
                }
            }
        }

        // Return the min and max
        (min, max)
    }

    // Get all vertices from the vertex data, converting them to f32 if they are f16
    pub fn get_vertices(&self) -> Vec<f32> {
        // Initialize vertices vector
        let mut vertices = Vec::new();

        // Iterate through all vertices
        for vertex in &self.vertex_data {
            // If the vertex has a half vertex
            if let Some(v) = &vertex.position {
                vertices.push(v.x);
                vertices.push(v.y);
                vertices.push(v.z);
            }
            // If the vertex has a full vertex
            else if let Some(v) = &vertex.position {
                vertices.push(v.x);
                vertices.push(v.y);
                vertices.push(v.z);
            }
        }

        // Return the vertices
        vertices
    }

    pub fn get_vertices_as_bytes(&self) -> Vec<u8> {
        let mut vertices = Vec::new();
        for vertex in self.get_vertices() {
            vertices.extend_from_slice(&vertex.to_le_bytes());
        }
        vertices
    }

    // Get all normals from the vertex data, if they exist
    pub fn get_normals(&self) -> Vec<f32> {
        // Initialize normals vector
        let mut normals = Vec::new();

        // Iterate through all vertices
        for vertex in &self.vertex_data {
            // If the vertex has a normal (it should)
            if let Some(v) = &vertex.normal {
                normals.push(v.x);
                normals.push(v.y);
                normals.push(v.z);
            }
        }

        // Return the normals vector
        normals
    }

    pub fn get_normals_as_bytes(&self) -> Vec<u8> {
        let mut normals = Vec::new();
        for normal in self.get_normals() {
            normals.extend_from_slice(&normal.to_le_bytes());
        }
        normals
    }

    // Get all UVs from the vertex data, if they exist
    pub fn get_uvs(&self) -> Vec<f32> {
        // Initialize uvs vector
        let mut uvs = Vec::new();

        // Iterate through all vertices
        for vertex in &self.vertex_data {
            // If the vertex has a uv, push it to the vector
            if let Some(v) = &vertex.uv {
                uvs.push(v.x);
                uvs.push(v.y);
            }
        }

        // Return the uvs vector
        uvs
    }

    pub fn get_uvs_as_bytes(&self) -> Vec<u8> {
        let mut uvs = Vec::new();
        for uv in self.get_uvs() {
            uvs.extend_from_slice(&uv.to_le_bytes());
        }
        uvs
    }

    pub fn get_bone_weights(&self) -> Vec<f32> {
        let mut weights = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(bone_weights) = &vertex.bone_weights {
                let lin: [f32; 4] = bone_weights.normalize().into();
                weights.extend_from_slice(lin.as_slice());
            }
        }

        weights
    }

    pub fn get_bone_weights_as_bytes(&self) -> Vec<u8> {
        let mut weights = Vec::new();
        for weight in self.get_bone_weights() {
            weights.extend_from_slice(&weight.to_le_bytes());
        }
        weights
    }

    pub fn get_bone_indices(&self) -> Vec<&U8Vec4> {
        let mut indices = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(bone_indices) = &vertex.bone_indices {
                indices.push(bone_indices);
            }
        }

        indices
    }

    pub fn get_bone_indices_as_bytes(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        for index in self.get_bone_indices() {
            let lin: [u8; 4] = index.clone().into();
            indices.extend_from_slice(lin.as_slice());
        }
        indices
    }

    pub fn get_vertex_positions(&self) -> Vec<Vec3> {
        let mut positions = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(v) = vertex.position {
                positions.push(v);
            }
        }

        positions
    }

    pub fn get_vertex_normals(&self) -> Vec<Vec3> {
        let mut normals = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(n) = vertex.normal {
                normals.push(n);
            }
        }

        normals
    }

    pub fn get_triangle_indices(&self) -> Vec<U16Vec3> {
        let halves = self.triangles.clone();
        halves
            .iter()
            .map(|x| U16Vec3 {
                x: x.x,
                y: x.y,
                z: x.z,
            })
            .collect::<Vec<U16Vec3>>()
    }

    pub fn get_vertex_uvs(&self) -> Vec<Vec2> {
        let mut uvs = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(v) = vertex.uv {
                uvs.push(v);
            }
        }

        uvs
    }

    pub fn get_vertex_weights(&self) -> Vec<Vec4> {
        let mut weights = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(w) = vertex.bone_weights {
                weights.push(w.normalize());
            }
        }

        weights
    }

    pub fn get_vertex_joints(&self) -> Vec<U8Vec4> {
        let mut joints = Vec::new();
        for vertex in &self.vertex_data {
            if let Some(j) = vertex.bone_indices {
                joints.push(j);
            }
        }

        joints
    }
}
