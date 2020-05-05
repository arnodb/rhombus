use amethyst::{core::Transform, ecs::prelude::*};
use rhombus_core::dodec::coordinates::quadric::QuadricVector;

pub struct QuadricPositionSystem;

impl QuadricPositionSystem {
    pub fn transform(position: QuadricPosition, transform: &mut Transform) {
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
pub struct QuadricPosition(QuadricVector);

impl Component for QuadricPosition {
    type Storage = DenseVecStorage<Self>;
}

impl<'s> System<'s> for QuadricPositionSystem {
    type SystemData = (
        ReadStorage<'s, QuadricPosition>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (positions, mut transforms): Self::SystemData) {
        for (position, transform) in (&positions, &mut transforms).join() {
            Self::transform(*position, transform);
        }
    }
}
