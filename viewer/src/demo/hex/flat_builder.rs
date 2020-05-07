use crate::{
    demo::{Color, RhombusViewerAssets},
    system::cubic::{CubicPosition, CubicPositionSystem},
};
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
use std::{collections::BTreeMap, ops::Deref, sync::Arc};

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
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            world: BTreeMap::new(),
            direction: 0,
            pointer_entities: Vec::new(),
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn create_pointer(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
        direction: usize,
    ) -> [Entity; 2] {
        let mut transform = Transform::default();
        let pos = (position, 0.5).into();
        CubicPositionSystem::transform(pos, &mut transform);
        transform.set_rotation_z_axis(direction as f32 * std::f32::consts::PI / 3.0);
        let pointer_rot_trans = data.world.create_entity().with(transform).with(pos).build();

        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.3, 0.3, 0.3));
        transform.set_translation_x(0.7);
        let color_data = assets.color_data[&Color::Cyan].clone();
        let pointer = data
            .world
            .create_entity()
            .with(Parent {
                entity: pointer_rot_trans,
            })
            .with(assets.pointer_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .build();

        [pointer, pointer_rot_trans]
    }

    fn create_ground(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
    ) -> Entity {
        let pos = (position, 0.1).into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 1.0));
        CubicPositionSystem::transform(pos, &mut transform);
        let color_data = assets.color_data[&Color::White].clone();
        data.world
            .create_entity()
            .with(assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .with(pos)
            .build()
    }

    fn create_wall(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
    ) -> Entity {
        let pos = (position, 0.3).into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 3.0));
        CubicPositionSystem::transform(pos, &mut transform);
        let color_data = assets.color_data[&Color::Red].clone();
        data.world
            .create_entity()
            .with(assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .with(pos)
            .build()
    }

    fn set_direction(&mut self, direction: usize, data: &mut StateData<'_, GameData<'_, '_>>) {
        if self.direction != direction {
            let mut transform_storage = data.world.write_storage::<Transform>();
            if let Some(transform) = transform_storage.get_mut(self.pointer_entities[1]) {
                transform.set_rotation_z_axis(direction as f32 * std::f32::consts::PI / 3.0);
            }
            self.direction = direction;
        }
    }

    fn set_position(&mut self, position: CubicVector, data: &mut StateData<'_, GameData<'_, '_>>) {
        if self.position != position {
            let mut pos_storage = data.world.write_storage::<CubicPosition>();
            if let Some(pos) = pos_storage.get_mut(self.pointer_entities[1]) {
                *pos = (position, 0.5).into();
            }
            self.position = position;
        }
    }

    fn wallize(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
    ) {
        self.world
            .entry(Self::to_world_key(position))
            .or_insert_with(|| HexData {
                state: HexState::Wall,
                entity: Self::create_wall(data, assets, position),
            });
    }
}

impl SimpleState for HexFlatBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();
        self.pointer_entities.extend(&Self::create_pointer(
            &mut data,
            &assets,
            self.position,
            self.direction,
        ));
        self.world.insert(
            Self::to_world_key(self.position),
            HexData {
                state: HexState::Open,
                entity: Self::create_ground(&mut data, &assets, self.position),
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
            let assets = data
                .world
                .read_resource::<Arc<RhombusViewerAssets>>()
                .deref()
                .clone();
            match get_key(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.set_direction((self.direction + 1) % 6, &mut data);
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.set_direction((self.direction + 5) % 6, &mut data);
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
                                    entity: Self::create_ground(&mut data, &assets, next),
                                }
                            });
                    match next_state.state {
                        HexState::Open => {
                            if new {
                                // Left
                                self.wallize(
                                    &mut data,
                                    &assets,
                                    self.position.neighbor((self.direction + 1) % 6),
                                );
                                // Right
                                self.wallize(
                                    &mut data,
                                    &assets,
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
                                        self.wallize(&mut data, &assets, ahead);
                                    }
                                    (_, Some(HexState::Open), _) => {
                                        self.wallize(&mut data, &assets, ahead_left);
                                        self.wallize(&mut data, &assets, ahead_right);
                                    }
                                    _ => {}
                                }
                            }
                            self.set_position(next, &mut data);
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
