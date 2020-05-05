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

pub struct DodecSphereDemo {
    position: QuadricVector,
    spheres: Vec<usize>,
    entities: Vec<Entity>,
}

impl DodecSphereDemo {
    pub fn new(position: QuadricVector) -> Self {
        Self {
            position,
            spheres: vec![2],
            entities: Vec::new(),
        }
    }
}

impl SimpleState for DodecSphereDemo {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();

        for radius in &self.spheres {
            for dodec in self.position.sphere_iter(*radius) {
                let pos = dodec.into();
                let mut transform = Transform::default();
                transform.set_scale(Vector3::new(0.8, 0.8, 0.8));
                QuadricPositionSystem::transform(pos, &mut transform);
                let color_data = assets.color_data[&Color::Red].clone();
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
