use super::prelude::*;

#[derive(Debug, Clone)]
pub struct BSSubIndexTriShape {
    pub bs_tri_shape: BSTriShape,
    pub num_primitives: u32,
    pub num_segments: u32,
    pub total_segments: u32,
    pub segment: Vec<BSGeometrySegmentData>,
    pub segment_data: Option<BSGeometrySegmentSharedData>,
}

impl Parse<&[u8]> for BSSubIndexTriShape {
    fn parse(i: &[u8]) -> IResult<&[u8], Self, nom::error::Error<&[u8]>> {
        let (i, bs_tri_shape) = BSTriShape::parse(i)?;
        //println!("{:#?}", bs_tri_shape);
        let (i, num_primitives) = le_u32(i)?;
        let (i, num_segments) = le_u32(i)?;
        let (i, total_segments) = le_u32(i)?;

        println!(
            "Primitives: {}, Segments: {}, Total: {}",
            num_primitives, num_segments, total_segments
        );

        let (i, segment) = count(BSGeometrySegmentData::parse, num_segments as usize)(i)?;

        let mut data = i;
        let segment_data = if !i.is_empty() {
            let (i, segment_data) = BSGeometrySegmentSharedData::parse(data)?;
            data = i;
            Some(segment_data)
        } else {
            None
        };

        Ok((
            data,
            BSSubIndexTriShape {
                bs_tri_shape,
                num_primitives,
                num_segments,
                total_segments,
                segment,
                segment_data,
            },
        ))
    }
}

#[derive(Debug, NomLE, Clone)]
pub struct BSGeometrySubSegment {
    pub start_index: u32,
    pub num_primitives: u32,
    pub parent_array_index: u32,
    pub unused: u32,
}

#[derive(Debug, NomLE, Clone)]
pub struct BSGeometryPerSegmentSharedData {
    pub user_index: u32,
    pub bone_id: u32,
    #[nom(LengthCount = "le_u32")]
    pub cut_offsets: Vec<f32>,
}

#[derive(Debug, NomLE, Clone)]
pub struct BSGeometrySegmentData {
    pub start_index: u32,
    pub num_primitives: u32,
    pub parent_array_index: u32,
    #[nom(LengthCount = "le_u32")]
    pub sub_segment: Vec<BSGeometrySubSegment>,
}

#[derive(Debug, NomLE, Clone)]
pub struct BSGeometrySegmentSharedData {
    pub num_segments: u32,
    pub total_segments: u32,
    #[nom(Count = "num_segments")]
    pub segment_starts: Vec<u32>,
    #[nom(Count = "total_segments")]
    pub per_segment_data: Vec<BSGeometryPerSegmentSharedData>,
    pub ssf_file: SizedString16,
}
