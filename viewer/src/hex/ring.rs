use crate::{assets::Color, world::RhombusViewerWorld};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::is_key_down,
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::sync::Arc;

pub struct HexRingDemo {
    position: CubicVector,
    rings: Vec<usize>,
    entities: Vec<Entity>,
}

impl HexRingDemo {
    pub fn new() -> Self {
        Self {
            position: CubicVector::default(),
            rings: vec![2],
            entities: Vec::new(),
        }
    }
}

impl SimpleState for HexRingDemo {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        for radius in &self.rings {
            for hex in self.position.ring_iter(*radius) {
                let mut transform = Transform::default();
                transform.set_scale(Vector3::new(0.8, 0.08, 0.8));
                let pos = (hex, 0.0).into();
                world.transform_cubic(pos, &mut transform);
                let color_data = world.assets.color_data[&Color::Red].clone();
                self.entities.push(
                    data.world
                        .create_entity()
                        .with(world.assets.hex_handle.clone())
                        .with(color_data.texture)
                        .with(color_data.material)
                        .with(transform)
                        .build(),
                );
            }
        }
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let result = data.world.delete_entities(self.entities.as_slice());
        self.entities.clear();
        result.expect("delete entities");
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Pop
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}
