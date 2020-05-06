use crate::{
    demo::{Color, RhombusViewerAssets},
    system::cubic::CubicPositionSystem,
};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
    sync::Arc,
};

const BLOCK_HEIGHT: isize = 2;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VerticalBlock {
    floor: isize,
    ceiling: isize,
    floor_entity: Entity,
    ceiling_entity: Entity,
}

#[derive(Debug, PartialEq, Eq)]
enum VerticalDirection {
    Horizontal,
    Up,
    Down,
}

pub struct HexBumpyBuilderDemo {
    position: CubicVector,
    height: isize,
    world: BTreeMap<(isize, isize), BTreeSet<VerticalBlock>>,
    direction: (usize, VerticalDirection),
}

impl HexBumpyBuilderDemo {
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            height: 0,
            world: BTreeMap::new(),
            direction: (0, VerticalDirection::Horizontal),
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn create_floor(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
        floor: isize,
    ) -> Entity {
        let pos = (position, floor as f32).into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 2.0));
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

    fn create_ceiling(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
        ceiling: isize,
    ) -> Entity {
        let pos = (position, ceiling as f32).into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.9, 0.9, 1.0));
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
}

impl SimpleState for HexBumpyBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();
        let vblock = self
            .world
            .entry(Self::to_world_key(self.position))
            .or_insert_with(BTreeSet::new);
        vblock.insert(VerticalBlock {
            floor: 0,
            ceiling: BLOCK_HEIGHT,
            floor_entity: Self::create_floor(&mut data, &assets, self.position, 0),
            ceiling_entity: Self::create_ceiling(&mut data, &assets, self.position, BLOCK_HEIGHT),
        });
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        for block in self.world.iter().flat_map(|(_, vblock)| vblock.iter()) {
            data.world
                .delete_entity(block.floor_entity)
                .expect("delete entity");
            data.world
                .delete_entity(block.ceiling_entity)
                .expect("delete entity");
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
                    self.direction.0 = (self.direction.0 + 1) % 6;
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.direction.0 = (self.direction.0 + 5) % 6;
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    self.direction.1 = match self.direction.1 {
                        VerticalDirection::Horizontal | VerticalDirection::Up => {
                            VerticalDirection::Up
                        }
                        VerticalDirection::Down => VerticalDirection::Horizontal,
                    }
                }
                Some((VirtualKeyCode::Down, ElementState::Pressed)) => {
                    self.direction.1 = match self.direction.1 {
                        VerticalDirection::Horizontal | VerticalDirection::Down => {
                            VerticalDirection::Down
                        }
                        VerticalDirection::Up => VerticalDirection::Horizontal,
                    }
                }
                Some((VirtualKeyCode::Space, ElementState::Pressed)) => {
                    let next_pos = self.position.neighbor(self.direction.0);
                    let next_floor = match self.direction.1 {
                        VerticalDirection::Horizontal => self.height,
                        VerticalDirection::Down => self.height - 1,
                        VerticalDirection::Up => self.height + 1,
                    };
                    let next_ceiling = next_floor + BLOCK_HEIGHT;
                    let vblock = self
                        .world
                        .entry(Self::to_world_key(next_pos))
                        .or_insert_with(BTreeSet::new);
                    // Really need an interval tree for that
                    enum Movement {
                        Void,
                        Go { height: isize },
                        Blocked,
                    }
                    let mut movement = Movement::Void;
                    for block in vblock.iter() {
                        if (block.floor - self.height).abs() <= 1 {
                            // Just go regardless of the vertical direction
                            movement = Movement::Go {
                                height: block.floor,
                            };
                            break;
                        }
                        if block.ceiling >= next_floor {
                            if block.floor < next_ceiling {
                                movement = Movement::Blocked;
                            }
                            break;
                        }
                    }
                    match movement {
                        Movement::Void => {
                            vblock.insert(VerticalBlock {
                                floor: next_floor,
                                ceiling: next_ceiling,
                                floor_entity: Self::create_floor(
                                    &mut data, &assets, next_pos, next_floor,
                                ),
                                ceiling_entity: Self::create_ceiling(
                                    &mut data,
                                    &assets,
                                    next_pos,
                                    next_ceiling,
                                ),
                            });
                            self.position = next_pos;
                            self.height = next_floor;
                        }
                        Movement::Go { height } => {
                            self.position = next_pos;
                            self.height = height;
                        }
                        Movement::Blocked => {}
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
