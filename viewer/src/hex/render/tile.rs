use crate::{
    assets::{Color, TextureAndMaterial},
    world::RhombusViewerWorld,
};
use amethyst::{
    assets::Handle,
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
    renderer::{types::Texture, Material},
};
use itertools::{EitherOrBoth, Itertools};
use rhombus_core::hex::coordinates::axial::AxialVector;
use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CellScale {
    pub horizontal: f32,
    pub vertical: f32,
}

#[derive(Debug)]
struct Cell {
    entity: Entity,
    wall: bool,
    visible: bool,
}

impl Cell {
    fn delete_entity(self, data: &mut StateData<'_, GameData<'_, '_>>) {
        data.world
            .delete_entity(self.entity)
            .expect("delete entity");
    }
}

pub struct TileRenderer {
    ground_scale: CellScale,
    wall_scale: CellScale,
    world: RefCell<BTreeMap<AxialVector, Cell>>,
}

impl TileRenderer {
    pub fn new(ground_scale: CellScale, wall_scale: CellScale) -> Self {
        Self {
            ground_scale,
            wall_scale,
            world: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn insert_cell(
        &mut self,
        position: AxialVector,
        wall: bool,
        visible: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        match self.world.borrow_mut().entry(position) {
            Entry::Vacant(entry) => {
                entry.insert(Cell {
                    entity: Self::create_hex(
                        position,
                        self.get_scale(wall),
                        self.get_texture_and_material(wall, visible, world),
                        data,
                        world,
                    ),
                    wall,
                    visible,
                });
            }
            Entry::Occupied(mut entry) => {
                let mut transform_storage = data.world.write_storage::<Transform>();
                let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
                let mut material_storage = data.world.write_storage::<Handle<Material>>();
                self.update_cell_internal(
                    entry.get_mut(),
                    wall,
                    visible,
                    world,
                    &mut transform_storage,
                    &mut texture_storage,
                    &mut material_storage,
                );
            }
        }
    }

    pub fn update_cell(
        &mut self,
        position: AxialVector,
        wall: bool,
        visible: bool,
        data: &StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.world.borrow_mut().entry(position).and_modify(|cell| {
            let mut transform_storage = data.world.write_storage::<Transform>();
            let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
            let mut material_storage = data.world.write_storage::<Handle<Material>>();
            self.update_cell_internal(
                cell,
                wall,
                visible,
                world,
                &mut transform_storage,
                &mut texture_storage,
                &mut material_storage,
            )
        });
    }

    pub fn set_scales(
        &mut self,
        ground_scale: CellScale,
        wall_scale: CellScale,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let mut tile_world = self.world.borrow_mut();
        if tile_world.is_empty() {
            self.ground_scale = ground_scale;
            self.wall_scale = wall_scale;
            return;
        }
        let update_grounds = self.ground_scale != ground_scale;
        let update_walls = self.wall_scale != wall_scale;
        if update_walls || update_walls {
            let mut transform_storage = data.world.write_storage::<Transform>();
            for cell in tile_world.values_mut() {
                if !cell.wall && update_grounds {
                    Self::update_cell_transform(cell, ground_scale, &mut transform_storage);
                }
                if cell.wall && update_walls {
                    Self::update_cell_transform(cell, wall_scale, &mut transform_storage);
                }
            }
            self.ground_scale = ground_scale;
            self.wall_scale = wall_scale;
        }
    }

    pub fn update_world<'a, C, I, Wall, Visible>(
        &mut self,
        cells: I,
        is_wall_cell: Wall,
        is_visible_cell: Visible,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        C: 'a,
        I: Iterator<Item = (&'a AxialVector, &'a C)>,
        Wall: Fn(AxialVector, &C) -> bool,
        Visible: Fn(AxialVector, &C) -> bool,
    {
        let mut to_remove = Vec::new();
        let mut to_insert = Vec::new();
        let mut tile_world = self.world.borrow_mut();
        {
            let mut transform_storage = data.world.write_storage::<Transform>();
            let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
            let mut material_storage = data.world.write_storage::<Handle<Material>>();
            for joint in tile_world
                .iter_mut()
                .merge_join_by(cells, |left, right| left.0.cmp(right.0))
            {
                match joint {
                    EitherOrBoth::Both(left, right) => self.update_cell_internal(
                        left.1,
                        is_wall_cell(*right.0, right.1),
                        is_visible_cell(*right.0, right.1),
                        world,
                        &mut transform_storage,
                        &mut texture_storage,
                        &mut material_storage,
                    ),
                    EitherOrBoth::Left(left) => to_remove.push(*left.0),
                    EitherOrBoth::Right(right) => {
                        let wall = is_wall_cell(*right.0, right.1);
                        let visible = is_visible_cell(*right.0, right.1);
                        to_insert.push((*right.0, wall, visible));
                    }
                }
            }
        }
        for position in to_remove {
            if let Some(removed) = tile_world.remove(&position) {
                removed.delete_entity(data);
            }
        }
        for (position, wall, visible) in to_insert {
            let vacant = tile_world.insert(
                position,
                Cell {
                    entity: Self::create_hex(
                        position,
                        self.get_scale(wall),
                        self.get_texture_and_material(wall, visible, world),
                        data,
                        world,
                    ),
                    wall,
                    visible,
                },
            );
            debug_assert!(vacant.is_none());
        }
    }

    pub fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let mut tile_world = BTreeMap::new();
        std::mem::swap(&mut *self.world.borrow_mut(), &mut tile_world);
        for (_, cell) in tile_world {
            cell.delete_entity(data);
        }
    }

    fn get_scale(&self, wall: bool) -> CellScale {
        if wall {
            self.wall_scale
        } else {
            self.ground_scale
        }
    }

    fn get_texture_and_material(
        &self,
        wall: bool,
        visible: bool,
        world: &RhombusViewerWorld,
    ) -> TextureAndMaterial {
        let color = if wall { Color::Red } else { Color::White };
        if visible {
            world.assets.color_data[&color].light.clone()
        } else {
            world.assets.color_data[&color].dark.clone()
        }
    }

    fn create_hex(
        position: AxialVector,
        scale: CellScale,
        tex_mat: TextureAndMaterial,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(
            scale.horizontal,
            scale.vertical,
            scale.horizontal,
        ));
        let pos = (position, scale.vertical).into();
        world.transform_axial(pos, &mut transform);
        data.world
            .create_entity()
            .with(world.assets.hex_handle.clone())
            .with(tex_mat.texture)
            .with(tex_mat.material)
            .with(transform)
            .build()
    }

    fn update_cell_internal(
        &self,
        cell: &mut Cell,
        wall: bool,
        visible: bool,
        world: &RhombusViewerWorld,
        transform_storage: &mut WriteStorage<Transform>,
        texture_storage: &mut WriteStorage<Handle<Texture>>,
        material_storage: &mut WriteStorage<Handle<Material>>,
    ) {
        if cell.wall != wall {
            Self::update_cell_transform(cell, self.get_scale(wall), transform_storage);
        }
        if cell.wall != wall || cell.visible != visible {
            Self::update_cell_color(
                cell,
                self.get_texture_and_material(wall, visible, world),
                texture_storage,
                material_storage,
            );
        }
        cell.wall = wall;
        cell.visible = visible;
    }

    fn update_cell_transform(
        cell: &mut Cell,
        scale: CellScale,
        transform_storage: &mut WriteStorage<Transform>,
    ) {
        let transform = transform_storage
            .get_mut(cell.entity)
            .expect("A cell always has a Transform");
        transform.set_scale(Vector3::new(
            scale.horizontal,
            scale.vertical,
            scale.horizontal,
        ));
        transform.translation_mut()[1] = scale.vertical;
    }

    fn update_cell_color(
        cell: &mut Cell,
        tex_mat: TextureAndMaterial,
        texture_storage: &mut WriteStorage<Handle<Texture>>,
        material_storage: &mut WriteStorage<Handle<Material>>,
    ) {
        *texture_storage
            .get_mut(cell.entity)
            .expect("A cell always has a Texture") = tex_mat.texture;
        *material_storage
            .get_mut(cell.entity)
            .expect("A cell always has a Material") = tex_mat.material;
    }
}
