use crate::{
    assets::Color, dispose::Dispose, hex::render::renderer::HexRenderer, world::RhombusViewerWorld,
};
use amethyst::{
    assets::Handle,
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
    renderer::Material,
};
use rhombus_core::hex::{coordinates::axial::AxialVector, storage::hash::RectHashStorage};

#[derive(Clone, Copy, Debug)]
pub struct HexScale {
    pub horizontal: f32,
    pub vertical: f32,
}

#[derive(Debug)]
pub struct Hex {
    entity: Option<Entity>,
    wall: bool,
    visible: bool,
}

impl Dispose for Hex {
    fn dispose(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some(entity) = self.entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
    }
}

pub struct TileRenderer {
    ground_scale: HexScale,
    wall_scale: HexScale,
}

impl TileRenderer {
    pub fn new(ground_scale: HexScale, wall_scale: HexScale) -> Self {
        Self {
            ground_scale,
            wall_scale,
        }
    }

    fn get_scale(&self, wall: bool) -> HexScale {
        if wall {
            self.wall_scale
        } else {
            self.ground_scale
        }
    }

    fn get_material(
        &self,
        wall: bool,
        visible: bool,
        world: &RhombusViewerWorld,
    ) -> Handle<Material> {
        let color = if wall { Color::Red } else { Color::White };
        if visible {
            world.assets.color_data[&color].light.clone()
        } else {
            world.assets.color_data[&color].dark.clone()
        }
    }

    fn create_hex(
        position: AxialVector,
        scale: HexScale,
        material: Handle<Material>,
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
            .with(material)
            .with(transform)
            .build()
    }

    pub fn update_hex(
        &self,
        position: AxialVector,
        hex: &mut Hex,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        let scale = self.get_scale(hex.wall);
        let material = self.get_material(hex.wall, hex.visible, world);
        if let Some(entity) = hex.entity {
            Self::update_hex_transform(entity, scale, &mut data.world.write_storage::<Transform>());
            Self::update_hex_color(
                entity,
                material,
                &mut data.world.write_storage::<Handle<Material>>(),
            );
        } else {
            hex.entity = Some(Self::create_hex(position, scale, material, data, world));
        }
    }

    fn update_hex_internal(
        &self,
        hex: &mut Hex,
        wall: bool,
        visible: bool,
        scale: HexScale,
        force: bool,
        world: &RhombusViewerWorld,
        transform_storage: &mut WriteStorage<Transform>,
        material_storage: &mut WriteStorage<Handle<Material>>,
    ) {
        if let Some(entity) = hex.entity {
            if force || hex.wall != wall {
                Self::update_hex_transform(entity, scale, transform_storage);
            }
            if force || hex.wall != wall || hex.visible != visible {
                Self::update_hex_color(
                    entity,
                    self.get_material(wall, visible, world),
                    material_storage,
                );
            }
        } else {
            unreachable!();
        }
        hex.wall = wall;
        hex.visible = visible;
    }

    fn update_hex_transform(
        entity: Entity,
        scale: HexScale,
        transform_storage: &mut WriteStorage<Transform>,
    ) {
        let transform = transform_storage
            .get_mut(entity)
            .expect("An hex always has a Transform");
        transform.set_scale(Vector3::new(
            scale.horizontal,
            scale.vertical,
            scale.horizontal,
        ));
        transform.translation_mut()[1] = scale.vertical;
    }

    fn update_hex_color(
        entity: Entity,
        material: Handle<Material>,
        material_storage: &mut WriteStorage<Handle<Material>>,
    ) {
        *material_storage
            .get_mut(entity)
            .expect("An hex always has a Material") = material;
    }
}

impl HexRenderer for TileRenderer {
    type Hex = Hex;

    fn new_hex(&mut self, wall: bool, visible: bool) -> Self::Hex {
        Hex {
            entity: None,
            wall,
            visible,
        }
    }

    fn update_world<'a, StorageHex, MapHex, Wall, Visible>(
        &mut self,
        hexes: &mut RectHashStorage<StorageHex>,
        is_wall_hex: Wall,
        is_visible_hex: Visible,
        get_renderer_hex: MapHex,
        visible_only: bool,
        force: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        StorageHex: 'a + Dispose,
        MapHex: Fn(&mut StorageHex) -> &mut Self::Hex,
        Wall: Fn(AxialVector, &StorageHex) -> bool,
        Visible: Fn(AxialVector, &StorageHex) -> bool,
    {
        let ground_scale = self.get_scale(false);
        let wall_scale = self.get_scale(true);
        {
            let mut transform_storage = data.world.write_storage::<Transform>();
            let mut material_storage = data.world.write_storage::<Handle<Material>>();
            for (pos, hex) in hexes.iter_mut() {
                let wall = is_wall_hex(pos, hex);
                let visible = is_visible_hex(pos, hex);
                let renderer_hex = get_renderer_hex(hex);
                if !visible_only || visible {
                    if renderer_hex.entity.is_some() {
                        self.update_hex_internal(
                            renderer_hex,
                            wall,
                            visible,
                            if wall { wall_scale } else { ground_scale },
                            force,
                            world,
                            &mut transform_storage,
                            &mut material_storage,
                        );
                    }
                }
            }
        }
        {
            for (pos, hex) in hexes.iter_mut() {
                let wall = is_wall_hex(pos, hex);
                let visible = is_visible_hex(pos, hex);
                let renderer_hex = get_renderer_hex(hex);
                if !visible_only || visible {
                    if renderer_hex.entity.is_none() {
                        renderer_hex.entity = Some(Self::create_hex(
                            pos,
                            if wall { wall_scale } else { ground_scale },
                            self.get_material(wall, visible, world),
                            data,
                            world,
                        ));
                        renderer_hex.wall = wall;
                        renderer_hex.visible = visible;
                    }
                } else {
                    if let Some(entity) = renderer_hex.entity.take() {
                        data.world.delete_entity(entity).expect("delete entity");
                    }
                }
            }
        }
    }

    fn clear(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}
