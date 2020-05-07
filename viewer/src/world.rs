use crate::assets::RhombusViewerAssets;
use amethyst::core::Transform;
use rhombus_core::{
    dodec::coordinates::quadric::QuadricVector, hex::coordinates::cubic::CubicVector,
};

#[derive(Debug)]
pub struct RhombusViewerWorld {
    pub assets: RhombusViewerAssets,
}

impl RhombusViewerWorld {
    pub fn transform_cubic(&self, position: CubicPosition, transform: &mut Transform) {
        let col = position.pos().x() + (position.pos().z() - (position.pos().z() & 1)) / 2;
        let row = position.pos().z();
        let altitude = position.alt();
        transform.set_translation_xyz(
            f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
            -row as f32 * 1.5,
            altitude,
        );
    }

    pub fn transform_quadric(&self, position: QuadricPosition, transform: &mut Transform) {
        let col = position.0.x() + (position.0.z() - (position.0.z() & 1)) / 2;
        let row = position.0.z();
        let depth = position.0.t();
        let small2 = 1.0 / (2.0 * f32::sqrt(2.0));
        transform.set_translation_xyz(
            f32::sqrt(3.0) * ((col as f32) + ((row & 1) as f32 + depth as f32) / 2.0),
            -1.5 * row as f32 - depth as f32 / 2.0,
            -(1.0 + small2) * depth as f32,
        );
    }
}

#[derive(Debug, Clone, Copy, From)]
pub struct CubicPosition(CubicVector, f32);

impl CubicPosition {
    fn pos(&self) -> &CubicVector {
        &self.0
    }

    fn alt(&self) -> f32 {
        self.1
    }
}

#[derive(Debug, Clone, Copy, From)]
pub struct QuadricPosition(QuadricVector);
