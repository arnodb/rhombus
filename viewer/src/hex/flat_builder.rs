use crate::{assets::Color, world::RhombusViewerWorld};
use amethyst::{
    core::{
        math::Vector3,
        transform::{Parent, Transform},
    },
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum HexState {
    Open,
    Wall,
}

struct HexData {
    state: HexState,
    entity: Entity,
}

pub struct HexFlatBuilderDemo {
    position: CubicVector,
    world: BTreeMap<(isize, isize), HexData>,
    direction: usize,
    pointer_entities: Vec<Entity>,
}

impl HexFlatBuilderDemo {
    pub fn new() -> Self {
        Self {
            position: CubicVector::new(0, 0, 0),
            world: BTreeMap::new(),
            direction: 0,
            pointer_entities: Vec::new(),
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn set_pointer_rot_trans_transform(
        transform: &mut Transform,
        position: CubicVector,
        direction: usize,
        world: &Arc<RhombusViewerWorld>,
    ) {
        let pos = (position, 0.5).into();
        world.transform_cubic(pos, transform);
        transform.set_rotation_y_axis(-(direction as f32) * std::f32::consts::PI / 3.0);
    }

    fn create_pointer(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
        direction: usize,
    ) -> [Entity; 2] {
        let mut transform = Transform::default();
        Self::set_pointer_rot_trans_transform(&mut transform, position, direction, world);
        let pointer_rot_trans = data.world.create_entity().with(transform).build();

        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.3, 0.1, 0.3));
        transform.set_translation_x(0.7);
        let color_data = world.assets.color_data[&Color::Cyan].clone();
        let pointer = data
            .world
            .create_entity()
            .with(Parent {
                entity: pointer_rot_trans,
            })
            .with(world.assets.pointer_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .build();

        world.follow(data, pointer_rot_trans, Some(pointer_rot_trans));

        [pointer, pointer_rot_trans]
    }

    fn create_ground(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.1, 0.8));
        let pos = (position, 0.1).into();
        world.transform_cubic(pos, &mut transform);
        let color_data = world.assets.color_data[&Color::White].clone();
        data.world
            .create_entity()
            .with(world.assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .build()
    }

    fn create_wall(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.3, 0.8));
        let pos = (position, 0.3).into();
        world.transform_cubic(pos, &mut transform);
        let color_data = world.assets.color_data[&Color::Red].clone();
        data.world
            .create_entity()
            .with(world.assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .build()
    }

    fn set_direction(
        &mut self,
        direction: usize,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        if self.direction != direction {
            let mut transform_storage = data.world.write_storage::<Transform>();
            if let Some(transform) = transform_storage.get_mut(self.pointer_entities[1]) {
                Self::set_pointer_rot_trans_transform(transform, self.position, direction, world);
            }
            self.direction = direction;
        }
    }

    fn set_position(
        &mut self,
        position: CubicVector,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        if self.position != position {
            let mut transform_storage = data.world.write_storage::<Transform>();
            if let Some(transform) = transform_storage.get_mut(self.pointer_entities[1]) {
                Self::set_pointer_rot_trans_transform(transform, position, self.direction, world);
            }
            self.position = position;
        }
    }

    fn wallize(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
    ) {
        self.world
            .entry(Self::to_world_key(position))
            .or_insert_with(|| HexData {
                state: HexState::Wall,
                entity: Self::create_wall(data, world, position),
            });
    }
}

impl SimpleState for HexFlatBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer_entities.extend(&Self::create_pointer(
            &mut data,
            &world,
            self.position,
            self.direction,
        ));
        self.world.insert(
            Self::to_world_key(self.position),
            HexData {
                state: HexState::Open,
                entity: Self::create_ground(&mut data, &world, self.position),
            },
        );
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        for pointer in &self.pointer_entities {
            data.world.delete_entity(*pointer).expect("delete entity");
        }
        self.pointer_entities.clear();
        for (_, hex) in &self.world {
            data.world.delete_entity(hex.entity).expect("delete entity");
        }
        self.world.clear();
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            let mut trans = Trans::None;
            let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
            match get_key(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.set_direction((self.direction + 1) % 6, &mut data, &world);
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.set_direction((self.direction + 5) % 6, &mut data, &world);
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    let next = self.position.neighbor(self.direction);
                    let mut new = false;
                    let next_state =
                        self.world
                            .entry(Self::to_world_key(next))
                            .or_insert_with(|| {
                                new = true;
                                HexData {
                                    state: HexState::Open,
                                    entity: Self::create_ground(&mut data, &world, next),
                                }
                            });
                    match next_state.state {
                        HexState::Open => {
                            if new {
                                // Left
                                self.wallize(
                                    &mut data,
                                    &world,
                                    self.position.neighbor((self.direction + 1) % 6),
                                );
                                // Right
                                self.wallize(
                                    &mut data,
                                    &world,
                                    self.position.neighbor((self.direction + 5) % 6),
                                );
                                // Ahead
                                let ahead_left = next.neighbor((self.direction + 1) % 6);
                                let ahead = next.neighbor(self.direction);
                                let ahead_right = next.neighbor((self.direction + 5) % 6);
                                match (
                                    self.world
                                        .get(&Self::to_world_key(ahead_left))
                                        .map(|h| h.state),
                                    self.world.get(&Self::to_world_key(ahead)).map(|h| h.state),
                                    self.world
                                        .get(&Self::to_world_key(ahead_right))
                                        .map(|h| h.state),
                                ) {
                                    (Some(HexState::Open), _, _) | (_, _, Some(HexState::Open)) => {
                                        self.wallize(&mut data, &world, ahead);
                                    }
                                    (_, Some(HexState::Open), _) => {
                                        self.wallize(&mut data, &world, ahead_left);
                                        self.wallize(&mut data, &world, ahead_right);
                                    }
                                    _ => {}
                                }
                            }
                            self.set_position(next, &mut data, &world);
                        }
                        HexState::Wall => {}
                    }
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }
}
