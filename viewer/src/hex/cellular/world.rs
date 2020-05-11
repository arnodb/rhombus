use crate::{assets::Color, world::RhombusViewerWorld};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
};
use rand::{thread_rng, RngCore};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::{collections::BTreeMap, sync::Arc};

const HEX_SCALE_HORIZONTAL: f32 = 0.8;
const GROUND_HEX_SCALE_VERTICAL: f32 = 0.1;
const WALL_HEX_SCALE_VERTICAL: f32 = 1.0;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HexState {
    Open,
    Wall,
    HardWall,
}

pub struct HexData {
    state: HexState,
    entity: Entity,
    automaton_count: u8,
}

#[derive(Default)]
pub struct World {
    world: BTreeMap<(isize, isize), HexData>,
}

impl World {
    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn to_position(world_key: (isize, isize)) -> CubicVector {
        let axial = AxialVector::new(world_key.0, world_key.1);
        axial.into()
    }

    fn create_ground(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
        scale: f32,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(
            scale * HEX_SCALE_HORIZONTAL,
            GROUND_HEX_SCALE_VERTICAL,
            scale * HEX_SCALE_HORIZONTAL,
        ));
        let pos = (position, GROUND_HEX_SCALE_VERTICAL).into();
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
        scale: f32,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(
            scale * HEX_SCALE_HORIZONTAL,
            WALL_HEX_SCALE_VERTICAL,
            scale * HEX_SCALE_HORIZONTAL,
        ));
        let pos = (position, WALL_HEX_SCALE_VERTICAL).into();
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

    pub fn reset_world(
        &mut self,
        radius: usize,
        cell_radius: usize,
        wall_ratio: f32,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.clear(data);
        let mut rng = thread_rng();
        for r in 0..radius {
            for cell in CubicVector::new(0, 0, 0).big_ring_iter(cell_radius, r) {
                self.world
                    .entry(Self::to_world_key(cell))
                    .or_insert_with(|| {
                        let is_wall =
                            ((rng.next_u32() & 0xffff) as f32 / 0x1_0000 as f32) < wall_ratio;
                        if is_wall {
                            HexData {
                                state: HexState::Wall,
                                entity: Self::create_wall(
                                    data,
                                    &world,
                                    cell,
                                    (2.0 * cell_radius as f32).max(1.0),
                                ),
                                automaton_count: 0,
                            }
                        } else {
                            HexData {
                                state: HexState::Open,
                                entity: Self::create_ground(
                                    data,
                                    &world,
                                    cell,
                                    (2.0 * cell_radius as f32).max(1.0),
                                ),
                                automaton_count: 0,
                            }
                        }
                    });
            }
        }
        for cell in CubicVector::new(0, 0, 0).big_ring_iter(cell_radius, radius) {
            self.world
                .entry(Self::to_world_key(cell))
                .or_insert_with(|| HexData {
                    state: HexState::HardWall,
                    entity: Self::create_wall(
                        data,
                        &world,
                        cell,
                        (2.0 * cell_radius as f32).max(1.0),
                    ),
                    automaton_count: 0,
                });
        }
    }

    pub fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        for hex in self.world.values() {
            data.world.delete_entity(hex.entity).expect("delete entity");
        }
        self.world.clear();
    }

    pub fn apply_cellular_automaton<RaiseF, RemainF>(
        &mut self,
        radius: usize,
        cell_radius: usize,
        raise_wall_test: RaiseF,
        remain_wall_test: RemainF,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) -> bool
    where
        RaiseF: Fn(u8) -> bool,
        RemainF: Fn(u8) -> bool,
    {
        for hex_data in &mut self.world {
            hex_data.1.automaton_count = 0;
        }
        for r in 0..=radius {
            for cell in CubicVector::new(0, 0, 0).big_ring_iter(cell_radius, r) {
                let hex_state = self.world.get(&Self::to_world_key(cell)).unwrap().state;
                let is_wall = match hex_state {
                    HexState::Wall | HexState::HardWall => true,
                    HexState::Open => false,
                };
                if is_wall {
                    for neighbor in cell.big_ring_iter(cell_radius, 1) {
                        self.world
                            .get_mut(&Self::to_world_key(neighbor))
                            .map(|hex_data| {
                                hex_data.automaton_count += 1;
                            });
                    }
                }
            }
        }
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        let mut frozen = true;
        for (key, hex_data) in &mut self.world {
            match hex_data.state {
                HexState::Wall => {
                    if !remain_wall_test(hex_data.automaton_count) {
                        data.world
                            .delete_entity(hex_data.entity)
                            .expect("delete entity");
                        hex_data.entity = Self::create_ground(
                            data,
                            &world,
                            Self::to_position(*key),
                            (2.0 * cell_radius as f32).max(1.0),
                        );
                        hex_data.state = HexState::Open;
                        frozen = false;
                    }
                }
                HexState::Open => {
                    if raise_wall_test(hex_data.automaton_count) {
                        data.world
                            .delete_entity(hex_data.entity)
                            .expect("delete entity");
                        hex_data.entity = Self::create_wall(
                            data,
                            &world,
                            Self::to_position(*key),
                            (2.0 * cell_radius as f32).max(1.0),
                        );
                        hex_data.state = HexState::Wall;
                        frozen = false;
                    }
                }
                HexState::HardWall => {}
            }
        }
        frozen
    }

    pub fn expand(
        &mut self,
        radius: usize,
        cell_radius: usize,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        if cell_radius <= 0 {
            return;
        }
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        for r in 0..=radius {
            for cell in CubicVector::new(0, 0, 0).big_ring_iter(cell_radius, r) {
                let HexData {
                    state: hex_state,
                    entity: hex_entity,
                    ..
                } = *self.world.get(&Self::to_world_key(cell)).unwrap();
                let is_wall = match hex_state {
                    HexState::Wall | HexState::HardWall => true,
                    HexState::Open => false,
                };
                {
                    let mut transform_storage = data.world.write_storage::<Transform>();
                    transform_storage.get_mut(hex_entity).map(|transform| {
                        transform.set_scale(Vector3::new(
                            HEX_SCALE_HORIZONTAL,
                            if is_wall {
                                WALL_HEX_SCALE_VERTICAL
                            } else {
                                GROUND_HEX_SCALE_VERTICAL
                            },
                            HEX_SCALE_HORIZONTAL,
                        ))
                    });
                }
                for s in 1..=cell_radius {
                    for sub_cell in cell.ring_iter(s) {
                        self.world
                            .entry(Self::to_world_key(sub_cell))
                            .or_insert_with(|| HexData {
                                state: hex_state,
                                entity: if is_wall {
                                    Self::create_wall(data, &world, sub_cell, 1.0)
                                } else {
                                    Self::create_ground(data, &world, sub_cell, 1.0)
                                },
                                automaton_count: 0,
                            });
                    }
                }
            }
        }
    }
}
