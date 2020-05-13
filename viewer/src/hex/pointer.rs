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
    prelude::*,
    renderer::{types::Texture, Material},
};
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VerticalDirection {
    Horizontal,
    Up,
    Down,
}

impl Default for VerticalDirection {
    fn default() -> Self {
        VerticalDirection::Horizontal
    }
}

#[derive(Default)]
pub struct HexPointer {
    /* Logical position */
    position: CubicVector,
    height: isize,
    /* Logical directions */
    direction: usize,
    vertical_direction: VerticalDirection,
    /* Display data */
    level_height: f32,
    entities: Option<HexPointerEntities>,
}

struct HexPointerEntities {
    pointer: Entity,
    pointer_rot_trans: Entity,
}

impl HexPointer {
    pub fn new_with_level_height(level_height: f32) -> Self {
        Self {
            level_height,
            ..Default::default()
        }
    }

    /* Position */

    pub fn position(&self) -> CubicVector {
        self.position
    }

    pub fn height(&self) -> isize {
        self.height
    }

    pub fn set_position(
        &mut self,
        position: CubicVector,
        height: isize,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        let update_rot_trans = self.position != position || self.height != height;

        self.position = position;
        self.height = height;

        if let Some(entities) = &self.entities {
            if update_rot_trans {
                let mut transform_storage = data.world.write_storage::<Transform>();
                if let Some(transform) = transform_storage.get_mut(entities.pointer_rot_trans) {
                    self.set_pointer_rot_trans_transform(transform, world);
                }
            }
        }
    }

    /* Directions */

    pub fn direction(&self) -> usize {
        self.direction
    }

    pub fn vertical_direction(&self) -> VerticalDirection {
        self.vertical_direction
    }

    pub fn increment_direction(
        &mut self,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        self.set_direction(
            (self.direction + 1) % 6,
            self.vertical_direction,
            data,
            world,
        );
    }

    pub fn decrement_direction(
        &mut self,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        self.set_direction(
            (self.direction + 5) % 6,
            self.vertical_direction,
            data,
            world,
        );
    }

    pub fn increment_vertical_direction(
        &mut self,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        self.set_direction(
            self.direction,
            match self.vertical_direction {
                VerticalDirection::Horizontal | VerticalDirection::Up => VerticalDirection::Up,
                VerticalDirection::Down => VerticalDirection::Horizontal,
            },
            data,
            world,
        );
    }

    pub fn decrement_vertical_direction(
        &mut self,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        self.set_direction(
            self.direction,
            match self.vertical_direction {
                VerticalDirection::Horizontal | VerticalDirection::Down => VerticalDirection::Down,
                VerticalDirection::Up => VerticalDirection::Horizontal,
            },
            data,
            world,
        );
    }

    pub fn set_direction(
        &mut self,
        direction: usize,
        vertical_direction: VerticalDirection,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        let update_rot_trans =
            self.direction != direction || self.vertical_direction != vertical_direction;
        let update_texture = self.vertical_direction != vertical_direction;

        self.direction = direction;
        self.vertical_direction = vertical_direction;

        if let Some(entities) = &self.entities {
            if update_rot_trans {
                let mut transform_storage = data.world.write_storage::<Transform>();
                if let Some(transform) = transform_storage.get_mut(entities.pointer_rot_trans) {
                    self.set_pointer_rot_trans_transform(transform, world);
                }
            }

            if update_texture {
                let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
                let mut material_storage = data.world.write_storage::<Handle<Material>>();
                if let (Some(texture), Some(material)) = (
                    texture_storage.get_mut(entities.pointer),
                    material_storage.get_mut(entities.pointer),
                ) {
                    let color_data = Self::get_pointer_texture_and_material(
                        self.vertical_direction,
                        &world.assets,
                    );
                    *texture = color_data.texture;
                    *material = color_data.material;
                }
            }
        }
    }

    /* Display */

    pub fn create_entities(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) {
        if self.entities.is_none() {
            self.entities = Some(self.create_pointer(data, world));
        }
    }

    pub fn delete_entities(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some(entities) = self.entities.take() {
            data.world
                .delete_entity(entities.pointer)
                .expect("delete entity");
            data.world
                .delete_entity(entities.pointer_rot_trans)
                .expect("delete entity");
        }
    }

    fn create_pointer(
        &self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &Arc<RhombusViewerWorld>,
    ) -> HexPointerEntities {
        let mut transform = Transform::default();
        self.set_pointer_rot_trans_transform(&mut transform, world);
        let pointer_rot_trans = data.world.create_entity().with(transform).build();

        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.3, 0.1, 0.3));
        transform.set_translation_x(0.7);
        let color_data =
            Self::get_pointer_texture_and_material(self.vertical_direction, &world.assets);
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

        HexPointerEntities {
            pointer,
            pointer_rot_trans,
        }
    }

    fn set_pointer_rot_trans_transform(
        &self,
        transform: &mut Transform,
        world: &Arc<RhombusViewerWorld>,
    ) {
        let pos = (self.position, 0.7 + self.height as f32 * self.level_height).into();
        world.transform_cubic(pos, transform);
        transform.set_rotation_y_axis(-(self.direction as f32) * std::f32::consts::PI / 3.0);
        match self.vertical_direction {
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
        vertical_direction: VerticalDirection,
        assets: &RhombusViewerAssets,
    ) -> ColorData {
        let color = match vertical_direction {
            VerticalDirection::Horizontal => Color::Cyan,
            VerticalDirection::Up => Color::Green,
            VerticalDirection::Down => Color::Red,
        };
        assets.color_data[&color].clone()
    }
}
