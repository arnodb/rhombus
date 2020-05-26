use crate::{
    hex::{
        pointer::HexPointer,
        render::{
            area::AreaRenderer,
            edge::EdgeRenderer,
            renderer::HexRenderer,
            tile::{CellScale, TileRenderer},
        },
    },
    world::RhombusViewerWorld,
};
use amethyst::{ecs::prelude::*, prelude::*};
use rand::{thread_rng, RngCore};
use rhombus_core::hex::{
    coordinates::{axial::AxialVector, direction::HexagonalDirection},
    field_of_view::FieldOfView,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

const HEX_SCALE_HORIZONTAL: f32 = 0.8;
const GROUND_HEX_SCALE_VERTICAL: f32 = 0.1;
const WALL_HEX_SCALE_VERTICAL: f32 = 1.0;

pub fn new_tile_renderer() -> TileRenderer {
    TileRenderer::new(
        CellScale {
            horizontal: HEX_SCALE_HORIZONTAL,
            vertical: GROUND_HEX_SCALE_VERTICAL,
        },
        CellScale {
            horizontal: HEX_SCALE_HORIZONTAL,
            vertical: WALL_HEX_SCALE_VERTICAL,
        },
        0,
    )
}

pub fn new_edge_renderer() -> EdgeRenderer {
    EdgeRenderer::new()
}

pub fn new_area_renderer() -> AreaRenderer {
    AreaRenderer::new()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HexState {
    Open,
    Wall,
    HardWall,
}

pub struct HexData {
    state: HexState,
    automaton_count: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FovState {
    Partial,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveMode {
    StraightAhead,
    StrafeLeft,
    StrafeRight,
}

pub struct World<R: HexRenderer> {
    world: BTreeMap<AxialVector, HexData>,
    renderer: R,
    renderer_dirty: bool,
    pointer: Option<(HexPointer, FovState)>,
}

impl<R: HexRenderer> World<R> {
    pub fn new(renderer: R) -> Self {
        let world = BTreeMap::new();
        Self {
            world,
            renderer,
            renderer_dirty: false,
            pointer: None,
        }
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
        self.renderer.set_cell_radius(cell_radius, data);
        let mut rng = thread_rng();
        for r in 0..radius {
            for cell in AxialVector::default().big_ring_iter(cell_radius, r) {
                let wall = ((rng.next_u32() & 0xffff) as f32 / 0x1_0000 as f32) < wall_ratio;
                self.world.insert(
                    cell,
                    if wall {
                        HexData {
                            state: HexState::Wall,
                            automaton_count: 0,
                        }
                    } else {
                        HexData {
                            state: HexState::Open,
                            automaton_count: 0,
                        }
                    },
                );
                self.renderer.insert_cell(cell, wall, true, data, &world);
            }
        }
        for cell in AxialVector::default().big_ring_iter(cell_radius, radius) {
            self.world.insert(
                cell,
                HexData {
                    state: HexState::HardWall,
                    automaton_count: 0,
                },
            );
            self.renderer.insert_cell(cell, true, true, data, &world);
        }
    }

    pub fn clear(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.delete_pointer(data, world);
        self.renderer.clear(data);
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
                        self.renderer.update_cell(*pos, false, true, data, &world);
                        hex_data.state = HexState::Open;
                        frozen = false;
                    }
                }
                HexState::Open => {
                    if raise_wall_test(hex_data.automaton_count) {
                        self.renderer.update_cell(*pos, true, true, data, &world);
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
        self.renderer.set_cell_radius(0, data);
        for r in 0..=radius {
            for cell in AxialVector::default().big_ring_iter(cell_radius, r) {
                let HexData {
                    state: hex_state, ..
                } = *self.world.get(&cell).unwrap();
                let wall = match hex_state {
                    HexState::Wall | HexState::HardWall => true,
                    HexState::Open => false,
                };
                for s in 1..=cell_radius {
                    for sub_pos in cell.ring_iter(s) {
                        self.world.insert(
                            sub_pos,
                            HexData {
                                state: hex_state,
                                automaton_count: 0,
                            },
                        );
                        self.renderer.insert_cell(sub_pos, wall, true, data, &world);
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
            self.renderer_dirty = true;
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

    pub fn next_position(&mut self, mode: MoveMode, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some((pointer, _)) = &mut self.pointer {
            let direction = match mode {
                MoveMode::StraightAhead => pointer.direction(),
                MoveMode::StrafeLeft => (pointer.direction() + 5) % 6,
                MoveMode::StrafeRight => (pointer.direction() + 1) % 6,
            };
            let next = pointer.position().neighbor(direction);
            if let Some(HexData {
                state: HexState::Open,
                ..
            }) = self.world.get(&next)
            {
                let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
                pointer.set_position(next, 0, data, &world);
                self.renderer_dirty = true;
            }
        }
    }

    pub fn change_field_of_view(&mut self, fov_state: FovState) {
        if let Some((_, pointer_fov_state)) = &mut self.pointer {
            *pointer_fov_state = fov_state;
            self.renderer_dirty = true;
        }
    }

    pub fn update_renderer_world(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if !self.renderer_dirty {
            return;
        }

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

        match fov_state {
            FovState::Partial => {
                self.renderer.update_world(
                    self.world.iter(),
                    |_, hex_data| hex_data.state != HexState::Open,
                    |pos, _| visible_positions.contains(&pos),
                    data,
                    &world,
                );
            }
            FovState::Full => {
                self.renderer.update_world(
                    self.world
                        .iter()
                        .filter(|(pos, _)| visible_positions.contains(pos)),
                    |_, hex_data| hex_data.state != HexState::Open,
                    |_, _| true,
                    data,
                    &world,
                );
            }
        }

        self.renderer_dirty = false;
    }

    pub fn update_renderer(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.renderer.update(data, &world);
    }
}
