use project_wormhole_shared::glam::{self, Mat3};

use super::dev::*;

#[derive(Debug, NomLE, PartialEq, Clone)]
pub struct NifRotation(pub BSMatrix3);

impl Default for NifRotation {
    fn default() -> Self {
        Self(BSMatrix3(Mat3::from_cols_array(&[
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
        ])))
    }
}

#[derive(Debug, NomLE, PartialEq, Clone)]
pub struct NifScale(pub f32);

impl Default for NifScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl From<NifScale> for [f32; 3] {
    fn from(scale: NifScale) -> [f32; 3] {
        [scale.0, scale.0, scale.0]
    }
}

#[derive(Debug, NomLE, PartialEq, Clone)]
pub struct NifTranslation(pub BSVec3);

impl Default for NifTranslation {
    fn default() -> Self {
        Self(BSVec3(glam::Vec3::ZERO))
    }
}
