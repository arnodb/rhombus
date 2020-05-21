use crate::{assets::Color, hex::pointer::HexPointer, world::RhombusViewerWorld};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{
    axial::AxialVector, cubic::CubicVector, direction::HexagonalDirection,
};
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
    world: BTreeMap<(isize, isize), HexData>,
    pointer: HexPointer,
}

impl HexFlatBuilderDemo {
    pub fn new() -> Self {
        Self {
            world: BTreeMap::new(),
            pointer: HexPointer::new_with_level_height(1.0),
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn create_ground(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
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
        world: &RhombusViewerWorld,
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

    fn raise_wall(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
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
        self.pointer.create_entities(&mut data, &world);
        self.world.insert(
            Self::to_world_key(self.pointer.position()),
            HexData {
                state: HexState::Open,
                entity: Self::create_ground(&mut data, &world, self.pointer.position()),
            },
        );
    }

    fn on_stop(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.delete_entities(&mut data, &world);
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
                    self.pointer.increment_direction(&data, &world);
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.pointer.decrement_direction(&data, &world);
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    let next = self.pointer.position().neighbor(self.pointer.direction());
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
                                self.raise_wall(
                                    &mut data,
                                    &world,
                                    self.pointer
                                        .position()
                                        .neighbor((self.pointer.direction() + 1) % 6),
                                );
                                // Right
                                self.raise_wall(
                                    &mut data,
                                    &world,
                                    self.pointer
                                        .position()
                                        .neighbor((self.pointer.direction() + 5) % 6),
                                );
                                // Ahead
                                let ahead_left = next.neighbor((self.pointer.direction() + 1) % 6);
                                let ahead = next.neighbor(self.pointer.direction());
                                let ahead_right = next.neighbor((self.pointer.direction() + 5) % 6);
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
                                        self.raise_wall(&mut data, &world, ahead);
                                    }
                                    (_, Some(HexState::Open), _) => {
                                        self.raise_wall(&mut data, &world, ahead_left);
                                        self.raise_wall(&mut data, &world, ahead_right);
                                    }
                                    _ => {}
                                }
                            }
                            self.pointer.set_position(next, 0, &mut data, &world);
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
