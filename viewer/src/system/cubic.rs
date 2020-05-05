use amethyst::{core::Transform, ecs::prelude::*};
use rhombus_core::hex::coordinates::cubic::CubicVector;

pub struct CubicPositionSystem;

impl CubicPositionSystem {
    pub fn transform(position: CubicPosition, transform: &mut Transform) {
        let col = position.0.x() + (position.0.z() - (position.0.z() & 1)) / 2;
        let row = position.0.z();
        transform.set_translation_xyz(
            f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
            -row as f32 * 1.5,
            0.0,
        );
    }
}

#[derive(Debug, Clone, Copy, From)]
pub struct CubicPosition(CubicVector);

impl Component for CubicPosition {
    type Storage = DenseVecStorage<Self>;
}

impl<'s> System<'s> for CubicPositionSystem {
    type SystemData = (ReadStorage<'s, CubicPosition>, WriteStorage<'s, Transform>);

    fn run(&mut self, (positions, mut transforms): Self::SystemData) {
        for (position, transform) in (&positions, &mut transforms).join() {
            Self::transform(*position, transform);
        }
    }
}
