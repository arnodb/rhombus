use amethyst::{core::Transform, ecs::prelude::*};
use rhombus_core::hex::coordinates::cubic::CubicVector;

pub struct CubicPositionSystem;

impl CubicPositionSystem {
    pub fn transform(position: CubicPosition, transform: &mut Transform) {
        let col = position.pos().x() + (position.pos().z() - (position.pos().z() & 1)) / 2;
        let row = position.pos().z();
        let altitude = position.alt();
        transform.set_translation_xyz(
            f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
            -row as f32 * 1.5,
            altitude,
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
