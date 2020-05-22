use crate::{assets::Color, hex::pointer::HexPointer, world::RhombusViewerWorld};
use amethyst::{
    assets::Handle,
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
    renderer::{types::Texture, Material},
};
use rand::{thread_rng, RngCore};
use rhombus_core::hex::{
    coordinates::{axial::AxialVector, cubic::CubicVector, direction::HexagonalDirection},
    field_of_view::FieldOfView,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

const HEX_SCALE_HORIZONTAL: f32 = 0.8;
const GROUND_HEX_SCALE_VERTICAL: f32 = 0.1;
const WALL_HEX_SCALE_VERTICAL: f32 = 1.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HexState {
    Open,
    Wall,
    HardWall,
}

pub struct HexData {
    state: HexState,
    entity: Option<(Entity, bool)>,
    automaton_count: u8,
}

impl HexData {
    fn delete_entity(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some((entity, _)) = self.entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FovState {
    Partial,
    Full,
}

#[derive(Default)]
pub struct World {
    world: BTreeMap<(isize, isize), HexData>,
    pointer: Option<(HexPointer, FovState)>,
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
        world: &RhombusViewerWorld,
        position: CubicVector,
        scale: f32,
        visible: bool,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(
            scale * HEX_SCALE_HORIZONTAL,
            GROUND_HEX_SCALE_VERTICAL,
            scale * HEX_SCALE_HORIZONTAL,
        ));
        let pos = (position, GROUND_HEX_SCALE_VERTICAL).into();
        world.transform_cubic(pos, &mut transform);
        let color_data = if visible {
            &world.assets.color_data[&Color::White].light
        } else {
            &world.assets.color_data[&Color::White].dark
        }
        .clone();
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
        scale: f32,
        visible: bool,
    ) -> Entity {
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(
            scale * HEX_SCALE_HORIZONTAL,
            WALL_HEX_SCALE_VERTICAL,
            scale * HEX_SCALE_HORIZONTAL,
        ));
        let pos = (position, WALL_HEX_SCALE_VERTICAL).into();
        world.transform_cubic(pos, &mut transform);
        let color_data = if visible {
            &world.assets.color_data[&Color::Red].light
        } else {
            &world.assets.color_data[&Color::Red].dark
        }
        .clone();
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
        self.clear(data, &world);
        let mut rng = thread_rng();
        for r in 0..radius {
            for cell in CubicVector::default().big_ring_iter(cell_radius, r) {
                self.world
                    .entry(Self::to_world_key(cell))
                    .or_insert_with(|| {
                        let is_wall =
                            ((rng.next_u32() & 0xffff) as f32 / 0x1_0000 as f32) < wall_ratio;
                        if is_wall {
                            HexData {
                                state: HexState::Wall,
                                entity: Some((
                                    Self::create_wall(
                                        data,
                                        &world,
                                        cell,
                                        (2.0 * cell_radius as f32).max(1.0),
                                        true,
                                    ),
                                    true,
                                )),
                                automaton_count: 0,
                            }
                        } else {
                            HexData {
                                state: HexState::Open,
                                entity: Some((
                                    Self::create_ground(
                                        data,
                                        &world,
                                        cell,
                                        (2.0 * cell_radius as f32).max(1.0),
                                        true,
                                    ),
                                    true,
                                )),
                                automaton_count: 0,
                            }
                        }
                    });
            }
        }
        for cell in CubicVector::default().big_ring_iter(cell_radius, radius) {
            self.world
                .entry(Self::to_world_key(cell))
                .or_insert_with(|| HexData {
                    state: HexState::HardWall,
                    entity: Some((
                        Self::create_wall(
                            data,
                            &world,
                            cell,
                            (2.0 * cell_radius as f32).max(1.0),
                            true,
                        ),
                        true,
                    )),
                    automaton_count: 0,
                });
        }
    }

    pub fn clear(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.delete_pointer(data, world);
        for (_, hex) in &mut self.world {
            hex.delete_entity(data);
        }
        self.world.clear();
    }

    fn delete_pointer(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        if let Some((mut pointer, _)) = self.pointer.take() {
            pointer.delete_entities(data, world);
        }
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
        for (_, hex_data) in &mut self.world {
            hex_data.automaton_count = 0;
        }
        for r in 0..=radius {
            for cell in CubicVector::default().big_ring_iter(cell_radius, r) {
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
                        hex_data.delete_entity(data);
                        hex_data.entity = Some((
                            Self::create_ground(
                                data,
                                &world,
                                Self::to_position(*key),
                                (2.0 * cell_radius as f32).max(1.0),
                                true,
                            ),
                            true,
                        ));
                        hex_data.state = HexState::Open;
                        frozen = false;
                    }
                }
                HexState::Open => {
                    if raise_wall_test(hex_data.automaton_count) {
                        hex_data.delete_entity(data);
                        hex_data.entity = Some((
                            Self::create_wall(
                                data,
                                &world,
                                Self::to_position(*key),
                                (2.0 * cell_radius as f32).max(1.0),
                                true,
                            ),
                            true,
                        ));
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
            for cell in CubicVector::default().big_ring_iter(cell_radius, r) {
                let HexData {
                    state: hex_state,
                    entity: hex_entity,
                    ..
                } = *self.world.get(&Self::to_world_key(cell)).unwrap();
                let is_wall = match hex_state {
                    HexState::Wall | HexState::HardWall => true,
                    HexState::Open => false,
                };
                if let Some((hex_entity, _)) = hex_entity {
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
                }
                for s in 1..=cell_radius {
                    for sub_cell in cell.ring_iter(s) {
                        self.world
                            .entry(Self::to_world_key(sub_cell))
                            .or_insert_with(|| HexData {
                                state: hex_state,
                                entity: Some((
                                    if is_wall {
                                        Self::create_wall(data, &world, sub_cell, 1.0, true)
                                    } else {
                                        Self::create_ground(data, &world, sub_cell, 1.0, true)
                                    },
                                    true,
                                )),
                                automaton_count: 0,
                            });
                    }
                }
            }
        }
    }

    fn find_open_cell(&self) -> Option<CubicVector> {
        let mut r = 0;
        loop {
            for cell in CubicVector::default().ring_iter(r) {
                let cell_data = self.world.get(&Self::to_world_key(cell));
                match cell_data {
                    Some(HexData {
                        state: HexState::Open,
                        ..
                    }) => return Some(cell),
                    Some(..) => (),
                    None => return None,
                }
            }
            r += 1;
        }
    }

    pub fn create_pointer(
        &mut self,
        fov_state: FovState,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.delete_pointer(data, &world);

        if let Some(cell) = self.find_open_cell() {
            let mut pointer = HexPointer::new_with_level_height(1.0);
            pointer.set_position(cell, 0, data, &world);
            pointer.create_entities(data, &world);
            self.pointer = Some((pointer, fov_state));
            self.update_entities(data);
        }
    }

    pub fn increment_direction(&mut self, data: &StateData<'_, GameData<'_, '_>>) {
        if let Some((pointer, _)) = &mut self.pointer {
            let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
            pointer.increment_direction(data, &world);
        }
    }

    pub fn decrement_direction(&mut self, data: &StateData<'_, GameData<'_, '_>>) {
        if let Some((pointer, _)) = &mut self.pointer {
            let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
            pointer.decrement_direction(data, &world);
        }
    }

    pub fn next_position(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some((pointer, _)) = &mut self.pointer {
            let next = pointer.position().neighbor(pointer.direction());
            if let Some(HexData {
                state: HexState::Open,
                ..
            }) = self.world.get(&Self::to_world_key(next))
            {
                let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
                pointer.set_position(next, 0, data, &world);
                self.update_entities(data);
            }
        }
    }

    pub fn change_field_of_view(
        &mut self,
        fov_state: FovState,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        if let Some((_, pointer_fov_state)) = &mut self.pointer {
            *pointer_fov_state = fov_state;
            self.update_entities(data);
        }
    }

    fn update_entities(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let (pointer, fov_state) = if let Some((pointer, fov_state)) = &self.pointer {
            (pointer, *fov_state)
        } else {
            return;
        };
        let mut visible_keys = BTreeSet::new();
        visible_keys.insert(Self::to_world_key(pointer.position()));
        let mut fov = FieldOfView::default();
        fov.start(pointer.position());
        let is_obstacle = |pos| {
            let hex_data = self.world.get(&Self::to_world_key(pos));
            match hex_data {
                Some(HexData {
                    state: HexState::Open,
                    ..
                }) => false,
                Some(HexData {
                    state: HexState::Wall,
                    ..
                })
                | Some(HexData {
                    state: HexState::HardWall,
                    ..
                }) => true,
                None => false,
            }
        };
        loop {
            let prev_len = visible_keys.len();
            for pos in fov.iter() {
                let key = Self::to_world_key(pointer.position() + pos);
                if self.world.contains_key(&key) {
                    let inserted = visible_keys.insert(key);
                    debug_assert!(inserted);
                }
            }
            if visible_keys.len() == prev_len {
                break;
            }
            fov.next_radius(&is_obstacle);
        }
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Action {
            None,
            CreateVisible,
            CreateInvisible,
            UpdateVisible,
            UpdateInvisible,
            Delete,
        }
        for (key, hex_data) in &mut self.world {
            // The two matches could probably be merged into one
            let action = if visible_keys.contains(key) {
                match hex_data.entity {
                    None => Action::CreateVisible,
                    Some((_, false)) => Action::UpdateVisible,
                    Some((_, true)) => Action::None,
                }
            } else {
                match (hex_data.entity, fov_state) {
                    (None, FovState::Partial) => Action::CreateInvisible,
                    (None, FovState::Full) => Action::None,
                    (Some((_, false)), FovState::Partial) => Action::None,
                    (Some((_, true)), FovState::Partial) => Action::UpdateInvisible,
                    (Some((_, _)), FovState::Full) => Action::Delete,
                }
            };
            match action {
                Action::CreateVisible => match hex_data.state {
                    HexState::Open => {
                        hex_data.entity = Some((
                            Self::create_ground(data, &world, Self::to_position(*key), 1.0, true),
                            true,
                        ));
                    }
                    HexState::Wall | HexState::HardWall => {
                        hex_data.entity = Some((
                            Self::create_wall(data, &world, Self::to_position(*key), 1.0, true),
                            true,
                        ));
                    }
                },
                Action::CreateInvisible => match hex_data.state {
                    HexState::Open => {
                        hex_data.entity = Some((
                            Self::create_ground(data, &world, Self::to_position(*key), 1.0, false),
                            false,
                        ));
                    }
                    HexState::Wall | HexState::HardWall => {
                        hex_data.entity = Some((
                            Self::create_wall(data, &world, Self::to_position(*key), 1.0, false),
                            false,
                        ));
                    }
                },
                Action::UpdateVisible => {
                    let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
                    let mut material_storage = data.world.write_storage::<Handle<Material>>();
                    let color_data = match hex_data.state {
                        HexState::Open => world.assets.color_data[&Color::White].light.clone(),
                        HexState::Wall | HexState::HardWall => {
                            world.assets.color_data[&Color::Red].light.clone()
                        }
                    };
                    texture_storage
                        .insert(hex_data.entity.unwrap().0, color_data.texture)
                        .expect("insert texture");
                    material_storage
                        .insert(hex_data.entity.unwrap().0, color_data.material)
                        .expect("insert material");
                    hex_data.entity.as_mut().unwrap().1 = true;
                }
                Action::UpdateInvisible => {
                    let mut texture_storage = data.world.write_storage::<Handle<Texture>>();
                    let mut material_storage = data.world.write_storage::<Handle<Material>>();
                    let color_data = match hex_data.state {
                        HexState::Open => world.assets.color_data[&Color::White].dark.clone(),
                        HexState::Wall | HexState::HardWall => {
                            world.assets.color_data[&Color::Red].dark.clone()
                        }
                    };
                    texture_storage
                        .insert(hex_data.entity.unwrap().0, color_data.texture)
                        .expect("insert texture");
                    material_storage
                        .insert(hex_data.entity.unwrap().0, color_data.material)
                        .expect("insert material");
                    hex_data.entity.as_mut().unwrap().1 = false;
                }
                Action::Delete => hex_data.delete_entity(data),
                Action::None => {}
            }
        }
    }
}
