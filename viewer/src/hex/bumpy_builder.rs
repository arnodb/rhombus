use crate::{
    assets::Color,
    hex::pointer::{HexPointer, VerticalDirection},
    world::RhombusViewerWorld,
};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{axial::AxialVector, direction::HexagonalDirection};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

const LEVEL_HEIGHT: f32 = 0.5;
// So that turning direction at each step leads to a nice stairway
const BLOCK_HEIGHT: isize = 5;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VerticalBlock {
    floor: isize,
    ceiling: isize,
    floor_entity: Entity,
    ceiling_entity: Entity,
}

pub struct HexBumpyBuilderDemo {
    world: BTreeMap<AxialVector, BTreeSet<VerticalBlock>>,
    pointer: HexPointer,
}

impl HexBumpyBuilderDemo {
    pub fn new() -> Self {
        Self {
            world: BTreeMap::new(),
            pointer: HexPointer::new_with_level_height(LEVEL_HEIGHT),
        }
    }

    fn create_floor(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
        position: AxialVector,
        floor: isize,
    ) -> Entity {
        let mut transform = Transform::default();
        // Height = 0.4
        transform.set_scale(Vector3::new(0.8, 0.2, 0.8));
        // Floor is solid from 0.0 to height.
        let pos = (position, floor as f32 * LEVEL_HEIGHT + 0.2).into();
        world.transform_axial(pos, &mut transform);
        let color_data = world.assets.color_data[&Color::White].light.clone();
        data.world
            .create_entity()
            .with(world.assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .build()
    }

    fn create_ceiling(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
        position: AxialVector,
        ceiling: isize,
    ) -> Entity {
        let mut transform = Transform::default();
        // Height = 0.1
        transform.set_scale(Vector3::new(0.8, 0.05, 0.8));
        let pos = (position, (ceiling as f32 + 0.7) * LEVEL_HEIGHT).into();
        world.transform_axial(pos, &mut transform);
        let color_data = world.assets.color_data[&Color::Red].light.clone();
        data.world
            .create_entity()
            .with(world.assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .build()
    }
}

impl SimpleState for HexBumpyBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.create_entities(&mut data, &world);
        let vblock = self
            .world
            .entry(self.pointer.position())
            .or_insert_with(BTreeSet::new);
        vblock.insert(VerticalBlock {
            floor: 0,
            ceiling: BLOCK_HEIGHT,
            floor_entity: Self::create_floor(&mut data, &world, self.pointer.position(), 0),
            ceiling_entity: Self::create_ceiling(
                &mut data,
                &world,
                self.pointer.position(),
                BLOCK_HEIGHT,
            ),
        });
    }

    fn on_stop(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.delete_entities(&mut data, &world);
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
                    self.pointer.increment_vertical_direction(&data, &world);
                }
                Some((VirtualKeyCode::Down, ElementState::Pressed)) => {
                    self.pointer.decrement_vertical_direction(&data, &world);
                }
                Some((VirtualKeyCode::Space, ElementState::Pressed)) => {
                    let next_pos = self.pointer.position().neighbor(self.pointer.direction());
                    let next_floor = match self.pointer.vertical_direction() {
                        VerticalDirection::Horizontal => self.pointer.height(),
                        VerticalDirection::Down => self.pointer.height() - 1,
                        VerticalDirection::Up => self.pointer.height() + 1,
                    };
                    let next_ceiling = next_floor + BLOCK_HEIGHT;
                    let vblock = self.world.entry(next_pos).or_insert_with(BTreeSet::new);
                    // Really need an interval tree for that
                    enum Movement {
                        Void,
                        Go { height: isize },
                        Blocked,
                    }
                    let mut movement = Movement::Void;
                    for block in vblock.iter() {
                        if (block.floor - self.pointer.height()).abs() <= 1 {
                            // Just go regardless of the vertical direction
                            movement = Movement::Go {
                                height: block.floor,
                            };
                            break;
                        }
                        if block.ceiling >= next_floor {
                            if block.floor <= next_ceiling {
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
                                    &mut data, &world, next_pos, next_floor,
                                ),
                                ceiling_entity: Self::create_ceiling(
                                    &mut data,
                                    &world,
                                    next_pos,
                                    next_ceiling,
                                ),
                            });
                            self.pointer
                                .set_position(next_pos, next_floor, &data, &world);
                        }
                        Movement::Go { height } => {
                            self.pointer.set_position(next_pos, height, &data, &world);
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
