use crate::{
    demo::{Color, RhombusViewerAssets},
    system::quadric::QuadricPositionSystem,
};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::is_key_down,
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::dodec::coordinates::quadric::QuadricVector;
use std::{ops::Deref, sync::Arc};

pub struct DodecDirectionsDemo {
    position: QuadricVector,
    entities: Vec<Entity>,
}

impl DodecDirectionsDemo {
    pub fn new(position: QuadricVector) -> Self {
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
            transform.set_scale(Vector3::new(0.3, 0.3, 0.3));
            QuadricPositionSystem::transform(pos, &mut transform);
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

impl SimpleState for DodecDirectionsDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();

        self.create_direction(&mut data, 0, 3, &assets, Color::Red);
        self.create_direction(&mut data, 6, 2, &assets, Color::Red);

        self.create_direction(&mut data, 1, 3, &assets, Color::Green);
        self.create_direction(&mut data, 7, 2, &assets, Color::Green);

        self.create_direction(&mut data, 2, 3, &assets, Color::Blue);
        self.create_direction(&mut data, 8, 2, &assets, Color::Blue);

        self.create_direction(&mut data, 3, 3, &assets, Color::Yellow);
        self.create_direction(&mut data, 9, 2, &assets, Color::Yellow);

        self.create_direction(&mut data, 4, 3, &assets, Color::Magenta);
        self.create_direction(&mut data, 10, 2, &assets, Color::Magenta);

        self.create_direction(&mut data, 5, 3, &assets, Color::Cyan);
        self.create_direction(&mut data, 11, 2, &assets, Color::Cyan);
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
