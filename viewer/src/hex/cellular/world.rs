use crate::{assets::Color, world::RhombusViewerWorld};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
};
use rand::{thread_rng, RngCore};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::{collections::BTreeMap, sync::Arc};

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

    pub fn reset_world(
        &mut self,
        radius: usize,
        // TODO
        _cell_radius: usize,
        wall_ratio: f32,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.clear(data);
        let mut rng = thread_rng();
        for r in 0..radius {
            for cell in CubicVector::new(0, 0, 0).ring_iter(r) {
                self.world
                    .entry(Self::to_world_key(cell))
                    .or_insert_with(|| {
                        let is_wall =
                            ((rng.next_u32() & 0xffff) as f32 / 0x1_0000 as f32) < wall_ratio;
                        if is_wall {
                            HexData {
                                state: HexState::Wall,
                                entity: Self::create_wall(data, &world, cell),
                                automaton_count: 0,
                            }
                        } else {
                            HexData {
                                state: HexState::Open,
                                entity: Self::create_ground(data, &world, cell),
                                automaton_count: 0,
                            }
                        }
                    });
            }
        }
        for cell in CubicVector::new(0, 0, 0).ring_iter(radius) {
            self.world
                .entry(Self::to_world_key(cell))
                .or_insert_with(|| HexData {
                    state: HexState::HardWall,
                    entity: Self::create_wall(data, &world, cell),
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
        // TODO
        _cell_radius: usize,
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
            for cell in CubicVector::new(0, 0, 0).ring_iter(r) {
                let hex_data = self.world.get(&Self::to_world_key(cell));
                let is_wall = match hex_data {
                    Some(HexData {
                        state: HexState::Wall,
                        ..
                    })
                    | Some(HexData {
                        state: HexState::HardWall,
                        ..
                    }) => true,
                    Some(HexData {
                        state: HexState::Open,
                        ..
                    }) => false,
                    None => unreachable!(""),
                };
                if is_wall {
                    for dir in 0..6 {
                        let neighbor = cell + CubicVector::direction(dir);
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
                        hex_data.entity =
                            Self::create_ground(data, &world, Self::to_position(*key));
                        hex_data.state = HexState::Open;
                        frozen = false;
                    }
                }
                HexState::Open => {
                    if raise_wall_test(hex_data.automaton_count) {
                        data.world
                            .delete_entity(hex_data.entity)
                            .expect("delete entity");
                        hex_data.entity = Self::create_wall(data, &world, Self::to_position(*key));
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
        _radius: usize,
        cell_radius: usize,
        _data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        if cell_radius <= 0 {
            return;
        }
        todo!();
    }
}
