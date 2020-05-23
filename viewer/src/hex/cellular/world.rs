use crate::{assets::Color, hex::pointer::HexPointer, world::RhombusViewerWorld};
use amethyst::{
    assets::Handle,
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba, types::Texture, Material},
};
use rand::{thread_rng, RngCore};
use rhombus_core::hex::{
    coordinates::{axial::AxialVector, direction::HexagonalDirection},
    field_of_view::FieldOfView,
    largest_area::LargestAreaIterator,
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
    world: BTreeMap<AxialVector, HexData>,
    open_areas: Option<Entity>,
    wall_areas: Option<Entity>,
    pointer: Option<(HexPointer, FovState)>,
}

impl World {
    fn create_ground(
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
        position: AxialVector,
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
        world.transform_axial(pos, &mut transform);
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
        position: AxialVector,
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
        world.transform_axial(pos, &mut transform);
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
            for cell in AxialVector::default().big_ring_iter(cell_radius, r) {
                self.world.entry(cell).or_insert_with(|| {
                    let is_wall = ((rng.next_u32() & 0xffff) as f32 / 0x1_0000 as f32) < wall_ratio;
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
        for cell in AxialVector::default().big_ring_iter(cell_radius, radius) {
            self.world.entry(cell).or_insert_with(|| HexData {
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
        self.delete_areas(data);
        for hex in self.world.values_mut() {
            hex.delete_entity(data);
        }
        self.world.clear();
    }

    fn delete_areas(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some(entity) = self.open_areas.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
        if let Some(entity) = self.wall_areas.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
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
        for hex_data in self.world.values_mut() {
            hex_data.automaton_count = 0;
        }
        for r in 0..=radius {
            for cell in AxialVector::default().big_ring_iter(cell_radius, r) {
                let hex_state = self.world.get(&cell).unwrap().state;
                let is_wall = match hex_state {
                    HexState::Wall | HexState::HardWall => true,
                    HexState::Open => false,
                };
                if is_wall {
                    for neighbor in cell.big_ring_iter(cell_radius, 1) {
                        if let Some(hex_data) = self.world.get_mut(&neighbor) {
                            hex_data.automaton_count += 1;
                        }
                    }
                }
            }
        }
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        let mut frozen = true;
        for (pos, hex_data) in &mut self.world {
            match hex_data.state {
                HexState::Wall => {
                    if !remain_wall_test(hex_data.automaton_count) {
                        hex_data.delete_entity(data);
                        hex_data.entity = Some((
                            Self::create_ground(
                                data,
                                &world,
                                *pos,
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
                                *pos,
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
            for cell in AxialVector::default().big_ring_iter(cell_radius, r) {
                let HexData {
                    state: hex_state,
                    entity: hex_entity,
                    ..
                } = *self.world.get(&cell).unwrap();
                let is_wall = match hex_state {
                    HexState::Wall | HexState::HardWall => true,
                    HexState::Open => false,
                };
                if let Some((hex_entity, _)) = hex_entity {
                    {
                        let mut transform_storage = data.world.write_storage::<Transform>();
                        if let Some(transform) = transform_storage.get_mut(hex_entity) {
                            transform.set_scale(Vector3::new(
                                HEX_SCALE_HORIZONTAL,
                                if is_wall {
                                    WALL_HEX_SCALE_VERTICAL
                                } else {
                                    GROUND_HEX_SCALE_VERTICAL
                                },
                                HEX_SCALE_HORIZONTAL,
                            ))
                        }
                    }
                }
                for s in 1..=cell_radius {
                    for sub_cell in cell.ring_iter(s) {
                        self.world.entry(sub_cell).or_insert_with(|| HexData {
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

    fn find_open_cell(&self) -> Option<AxialVector> {
        let mut r = 0;
        loop {
            for cell in AxialVector::default().ring_iter(r) {
                let cell_data = self.world.get(&cell);
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
            }) = self.world.get(&next)
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
        let mut visible_positions = BTreeSet::new();
        visible_positions.insert(pointer.position());
        let mut fov = FieldOfView::default();
        fov.start(pointer.position());
        let is_obstacle = |pos| {
            let hex_data = self.world.get(&pos);
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
            let prev_len = visible_positions.len();
            for pos in fov.iter() {
                let key = pointer.position() + pos;
                if self.world.contains_key(&key) {
                    let inserted = visible_positions.insert(key);
                    debug_assert!(inserted);
                }
            }
            if visible_positions.len() == prev_len {
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
        for (pos, hex_data) in &mut self.world {
            // The two matches could probably be merged into one
            let action = if visible_positions.contains(pos) {
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
                        hex_data.entity =
                            Some((Self::create_ground(data, &world, *pos, 1.0, true), true));
                    }
                    HexState::Wall | HexState::HardWall => {
                        hex_data.entity =
                            Some((Self::create_wall(data, &world, *pos, 1.0, true), true));
                    }
                },
                Action::CreateInvisible => match hex_data.state {
                    HexState::Open => {
                        hex_data.entity =
                            Some((Self::create_ground(data, &world, *pos, 1.0, false), false));
                    }
                    HexState::Wall | HexState::HardWall => {
                        hex_data.entity =
                            Some((Self::create_wall(data, &world, *pos, 1.0, false), false));
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

        match fov_state {
            FovState::Partial => self.update_areas(data, &world, &|_| true),
            FovState::Full => {
                self.update_areas(data, &world, &|pos| visible_positions.contains(&pos))
            }
        }
    }

    fn update_areas<F>(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
        filter: &F,
    ) where
        F: Fn(AxialVector) -> bool,
    {
        self.delete_areas(data);
        self.open_areas = Some(self.create_debug_area(data, world, false, &filter));
        self.wall_areas = Some(self.create_debug_area(data, world, true, &filter));
    }

    fn create_debug_area<F>(
        &self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
        wall: bool,
        filter: &F,
    ) -> Entity
    where
        F: Fn(AxialVector) -> bool,
    {
        let mut debug_lines_component = DebugLinesComponent::with_capacity(100);
        let color = if wall {
            Srgba::new(1.0, 0.0, 0.0, 1.0)
        } else {
            Srgba::new(1.0, 1.0, 1.0, 1.0)
        };
        let mut largest_area_iterator = LargestAreaIterator::default();
        if wall {
            largest_area_iterator.initialize(self.world.iter().filter_map(|(pos, cell_data)| {
                if filter(*pos) && cell_data.state != HexState::Open {
                    Some(*pos)
                } else {
                    None
                }
            }));
        } else {
            largest_area_iterator.initialize(self.world.iter().filter_map(|(pos, cell_data)| {
                if filter(*pos) && cell_data.state == HexState::Open {
                    Some(*pos)
                } else {
                    None
                }
            }));
        }
        loop {
            let area = largest_area_iterator.next_largest_area();
            if area.1.is_none() {
                break;
            }
            if let Some((range_q, range_r)) = area.1 {
                let mut p1 = world.axial_translation(
                    (AxialVector::new(*range_q.start(), *range_r.start()), 2.0).into(),
                );
                p1[0] -= 3.0_f32.sqrt() / 2.0;
                p1[2] += 0.5;
                let mut p2 = world.axial_translation(
                    (AxialVector::new(*range_q.start(), *range_r.end()), 2.0).into(),
                );
                p2[0] -= 1.0 / (3.0_f32.sqrt() * 2.0);
                p2[2] -= 0.5;
                let mut p3 = world.axial_translation(
                    (AxialVector::new(*range_q.end(), *range_r.end()), 2.0).into(),
                );
                p3[0] += 3.0_f32.sqrt() / 2.0;
                p3[2] -= 0.5;
                let mut p4 = world.axial_translation(
                    (AxialVector::new(*range_q.end(), *range_r.start()), 2.0).into(),
                );
                p4[0] += 1.0 / (3.0_f32.sqrt() * 2.0);
                p4[2] += 0.5;
                debug_lines_component.add_line(p1.into(), p2.into(), color);
                debug_lines_component.add_line(p2.into(), p3.into(), color);
                debug_lines_component.add_line(p3.into(), p4.into(), color);
                debug_lines_component.add_line(p4.into(), p1.into(), color);
            }
        }
        data.world
            .create_entity()
            .with(debug_lines_component)
            .build()
    }
}
