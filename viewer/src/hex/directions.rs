use crate::{assets::Color, world::RhombusViewerWorld};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::is_key_down,
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::{ops::Deref, sync::Arc};

pub struct HexDirectionsDemo {
    position: CubicVector,
    entities: Vec<Entity>,
}

impl HexDirectionsDemo {
    pub fn new() -> Self {
        Self {
            position: CubicVector::new(0, 0, 0),
            entities: Vec::new(),
        }
    }

    fn create_direction(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        direction: usize,
        length: usize,
        color: Color,
    ) {
        let mut origin = self.position;
        for _ in 0..length {
            origin = origin.neighbor(direction);
            let mut transform = Transform::default();
            transform.set_scale(Vector3::new(0.3, 0.3, 0.1));
            let pos = (origin, 0.0).into();
            world.transform_cubic(pos, &mut transform);
            let color_data = world.assets.color_data[&color].clone();
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

impl SimpleState for HexDirectionsDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = data
            .world
            .read_resource::<Arc<RhombusViewerWorld>>()
            .deref()
            .clone();

        self.create_direction(&mut data, &world, 0, 3, Color::Red);
        self.create_direction(&mut data, &world, 3, 2, Color::Red);

        self.create_direction(&mut data, &world, 1, 3, Color::Green);
        self.create_direction(&mut data, &world, 4, 2, Color::Green);

        self.create_direction(&mut data, &world, 2, 3, Color::Blue);
        self.create_direction(&mut data, &world, 5, 2, Color::Blue);
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
