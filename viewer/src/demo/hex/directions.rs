use crate::{
    demo::{Color, RhombusViewerAssets},
    system::cubic::CubicPositionSystem,
};
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
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            entities: Vec::new(),
        }
    }

    fn create_direction(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        direction: usize,
        length: usize,
        assets: &Arc<RhombusViewerAssets>,
        color: Color,
    ) {
        let mut origin = self.position;
        for _ in 0..length {
            origin = origin.neighbor(direction);
            let pos = origin.into();
            let mut transform = Transform::default();
            transform.set_scale(Vector3::new(0.3, 0.3, 1.0));
            CubicPositionSystem::transform(pos, &mut transform);
            let color_data = assets.color_data[&color].clone();
            self.entities.push(
                data.world
                    .create_entity()
                    .with(assets.hex_handle.clone())
                    .with(color_data.texture)
                    .with(color_data.material)
                    .with(transform)
                    .with(pos)
                    .build(),
            );
        }
    }
}

impl SimpleState for HexDirectionsDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();

        self.create_direction(&mut data, 0, 3, &assets, Color::Red);
        self.create_direction(&mut data, 3, 2, &assets, Color::Red);

        self.create_direction(&mut data, 1, 3, &assets, Color::Green);
        self.create_direction(&mut data, 4, 2, &assets, Color::Green);

        self.create_direction(&mut data, 2, 3, &assets, Color::Blue);
        self.create_direction(&mut data, 5, 2, &assets, Color::Blue);
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
