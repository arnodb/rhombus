use crate::{
    assets::{Color, ColorData, RhombusViewerAssets},
    world::RhombusViewerWorld,
};
use amethyst::{
    assets::Handle,
    core::{
        math::Vector3,
        transform::{Parent, Transform},
    },
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    renderer::{types::Texture, Material},
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    pointer_entities: Vec<Entity>,
}

impl HexBumpyBuilderDemo {
    pub fn new() -> Self {
        Self {
            position: CubicVector::new(0, 0, 0),
            height: 0,
            world: BTreeMap::new(),
            direction: (0, VerticalDirection::Horizontal),
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
        height: isize,
        direction: (usize, VerticalDirection),
        world: &Arc<RhombusViewerWorld>,
    ) {
        let pos = (position, 0.7 + height as f32 * LEVEL_HEIGHT).into();
        world.transform_cubic(pos, transform);
        transform.set_rotation_y_axis(-(direction.0 as f32) * std::f32::consts::PI / 3.0);
        match direction.1 {
            VerticalDirection::Horizontal => {}
            VerticalDirection::Up => {
                transform.append_rotation_z_axis(-std::f32::consts::PI / 10.0);
            }
            VerticalDirection::Down => {
                transform.append_rotation_z_axis(std::f32::consts::PI / 10.0);
            }
        }
    }

    fn get_pointer_texture_and_material(
        direction: (usize, VerticalDirection),
        assets: &RhombusViewerAssets,
    ) -> ColorData {
        let color = match direction.1 {
            VerticalDirection::Horizontal => Color::Cyan,
            VerticalDirection::Up => Color::Green,
            VerticalDirection::Down => Color::Red,
        };
        assets.color_data[&color].clone()
    }

    fn create_pointer(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
        height: isize,
        direction: (usize, VerticalDirection),
    ) -> [Entity; 2] {
        let mut transform = Transform::default();
        Self::set_pointer_rot_trans_transform(&mut transform, position, height, direction, world);
        let pointer_rot_trans = data.world.create_entity().with(transform).build();

        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.3, 0.1, 0.3));
        transform.set_translation_x(0.7);
        let color_data = Self::get_pointer_texture_and_material(direction, &world.assets);
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

    fn create_floor(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
        floor: isize,
    ) -> Entity {
        let mut transform = Transform::default();
        // Height = 0.4
        transform.set_scale(Vector3::new(0.8, 0.2, 0.8));
        // Floor is solid from 0.0 to height.
        let pos = (position, floor as f32 * LEVEL_HEIGHT + 0.2).into();
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

    fn create_ceiling(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
        position: CubicVector,
        ceiling: isize,
    ) -> Entity {
        let mut transform = Transform::default();
        // Height = 0.1
        transform.set_scale(Vector3::new(0.8, 0.05, 0.8));
        let pos = (position, (ceiling as f32 + 0.7) * LEVEL_HEIGHT).into();
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
        direction: (usize, VerticalDirection),
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        if self.direction.0 != direction.0 || self.direction.1 != direction.1 {
            let mut transform_storage = data.world.write_storage::<Transform>();
            if let Some(transform) = transform_storage.get_mut(self.pointer_entities[1]) {
                Self::set_pointer_rot_trans_transform(
                    transform,
                    self.position,
                    self.height,
                    direction,
                    world,
                );
            }
        }

        if self.direction.1 != direction.1 {
            let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
            let mut material_storage = data.world.write_storage::<Handle<Material>>();
            if let (Some(texture), Some(material)) = (
                texture_storage.get_mut(self.pointer_entities[0]),
                material_storage.get_mut(self.pointer_entities[0]),
            ) {
                let color_data = Self::get_pointer_texture_and_material(direction, &world.assets);
                *texture = color_data.texture;
                *material = color_data.material;
            }
        }

        self.direction = direction;
    }

    fn set_position(
        &mut self,
        position: CubicVector,
        height: isize,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        if self.position != position || self.height != height {
            let mut transform_storage = data.world.write_storage::<Transform>();
            if let Some(transform) = transform_storage.get_mut(self.pointer_entities[1]) {
                Self::set_pointer_rot_trans_transform(
                    transform,
                    position,
                    height,
                    self.direction,
                    world,
                );
            }
            self.position = position;
            self.height = height;
        }
    }
}

impl SimpleState for HexBumpyBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer_entities.extend(&Self::create_pointer(
            &mut data,
            &world,
            self.position,
            self.height,
            self.direction,
        ));
        let vblock = self
            .world
            .entry(Self::to_world_key(self.position))
            .or_insert_with(BTreeSet::new);
        vblock.insert(VerticalBlock {
            floor: 0,
            ceiling: BLOCK_HEIGHT,
            floor_entity: Self::create_floor(&mut data, &world, self.position, 0),
            ceiling_entity: Self::create_ceiling(&mut data, &world, self.position, BLOCK_HEIGHT),
        });
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        for pointer in &self.pointer_entities {
            data.world.delete_entity(*pointer).expect("delete entity");
        }
        self.pointer_entities.clear();
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
                    self.set_direction(
                        ((self.direction.0 + 1) % 6, self.direction.1),
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.set_direction(
                        ((self.direction.0 + 5) % 6, self.direction.1),
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    self.set_direction(
                        (
                            self.direction.0,
                            match self.direction.1 {
                                VerticalDirection::Horizontal | VerticalDirection::Up => {
                                    VerticalDirection::Up
                                }
                                VerticalDirection::Down => VerticalDirection::Horizontal,
                            },
                        ),
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::Down, ElementState::Pressed)) => {
                    self.set_direction(
                        (
                            self.direction.0,
                            match self.direction.1 {
                                VerticalDirection::Horizontal | VerticalDirection::Down => {
                                    VerticalDirection::Down
                                }
                                VerticalDirection::Up => VerticalDirection::Horizontal,
                            },
                        ),
                        &mut data,
                        &world,
                    );
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
                            self.set_position(next_pos, next_floor, &mut data, &world);
                        }
                        Movement::Go { height } => {
                            self.set_position(next_pos, height, &mut data, &world);
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
