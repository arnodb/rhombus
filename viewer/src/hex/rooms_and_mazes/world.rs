use crate::{
    dispose::Dispose,
    hex::{
        pointer::HexPointer, render::renderer::HexRenderer, shape::cubic_range::CubicRangeShape,
    },
    world::RhombusViewerWorld,
};
use amethyst::{ecs::prelude::*, prelude::*};
use rand::{thread_rng, Rng};
use rhombus_core::hex::{
    coordinates::{
        axial::AxialVector,
        cubic::CubicVector,
        direction::{HexagonalDirection, NUM_DIRECTIONS},
    },
    field_of_view::FieldOfView,
    storage::hash::RectHashStorage,
};
use smallvec::SmallVec;
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HexState {
    Open(usize),
    Wall,
}

pub struct HexData {
    state: HexState,
}

impl Dispose for HexData {
    fn dispose(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FovState {
    Partial,
    Full,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveMode {
    StraightAhead,
    StrafeLeftAhead,
    StrafeLeftBack,
    StrafeRightAhead,
    StrafeRightBack,
    StraightBack,
}

const CELL_RADIUS_RATIO_DEN: usize = 42;

pub struct World<R: HexRenderer> {
    shape: CubicRangeShape,
    shape_positions: Vec<AxialVector>,
    hexes: RectHashStorage<(HexData, R::Hex)>,
    renderer: R,
    renderer_dirty: bool,
    rooms: Vec<CubicRangeShape>,
    next_region: usize,
    pointer: Option<(HexPointer, FovState)>,
}

impl<R: HexRenderer> World<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            shape: CubicRangeShape::default(),
            shape_positions: Vec::new(),
            hexes: RectHashStorage::new(),
            renderer,
            renderer_dirty: false,
            rooms: Vec::new(),
            next_region: 0,
            pointer: None,
        }
    }

    pub fn set_shape_and_reset_world(
        &mut self,
        shape: CubicRangeShape,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        self.shape = shape;
        let cell_radius = Self::compute_cell_radius(&self.shape, CELL_RADIUS_RATIO_DEN);
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().big_ring_iter(cell_radius, r) {
                let mut one_inside = false;
                for v in pos.ring_iter(cell_radius) {
                    if self.shape.contains_position(v) {
                        self.shape_positions.push(v);
                        one_inside = true;
                    }
                }
                if !one_inside {
                    continue;
                }
                end = false;
                for s in 0..cell_radius {
                    for v in pos.ring_iter(s) {
                        if self.shape.contains_position(v) {
                            self.shape_positions.push(v);
                        }
                    }
                }
            }
            if end {
                break;
            }
            r += 1;
        }
        self.reset_world(data);
    }

    pub fn reset_world(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.clear(data, &world);

        for v in &self.shape_positions {
            self.hexes.insert(
                *v,
                (
                    HexData {
                        state: HexState::Wall,
                    },
                    self.renderer.new_hex(true, true),
                ),
            );
        }

        self.renderer_dirty = true;
    }

    fn compute_cell_radius(shape: &CubicRangeShape, cell_radius_ratio_den: usize) -> usize {
        let mut deltas = [
            shape.range_x().end() - shape.range_x().start(),
            shape.range_y().end() - shape.range_y().start(),
            shape.range_z().end() - shape.range_z().start(),
        ];
        deltas.sort();
        deltas[1] as usize / cell_radius_ratio_den
    }

    pub fn clear(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.delete_pointer(data, world);
        self.rooms.clear();
        self.renderer.clear(data);
        self.hexes.dispose(data);
        self.next_region = 0;
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

    pub fn add_room(&mut self) {
        let mut deltas = [
            self.shape.range_x().end() - self.shape.range_x().start(),
            self.shape.range_y().end() - self.shape.range_y().start(),
            self.shape.range_z().end() - self.shape.range_z().start(),
        ];
        deltas.sort();
        let radius = deltas[1] / 10;

        let mut rng = thread_rng();

        let mut new_room =
            CubicRangeShape::new((-radius, radius), (-radius, radius), (-radius, radius));
        let funcs: [(
            fn(&mut CubicRangeShape, usize) -> bool,
            fn(&mut CubicRangeShape, usize) -> bool,
        ); 6] = [
            (
                CubicRangeShape::shrink_x_start,
                CubicRangeShape::stretch_x_start,
            ),
            (
                CubicRangeShape::shrink_x_end,
                CubicRangeShape::stretch_x_end,
            ),
            (
                CubicRangeShape::shrink_y_start,
                CubicRangeShape::stretch_y_start,
            ),
            (
                CubicRangeShape::shrink_y_end,
                CubicRangeShape::stretch_y_end,
            ),
            (
                CubicRangeShape::shrink_z_start,
                CubicRangeShape::stretch_z_start,
            ),
            (
                CubicRangeShape::shrink_z_end,
                CubicRangeShape::stretch_z_end,
            ),
        ];
        for (st, sh) in funcs.iter() {
            let d = rng.gen_range(-radius / 3, radius / 3 + 1);
            for _ in 0..d.abs() {
                if d > 0 {
                    st(&mut new_room, 2);
                } else if d < 0 {
                    sh(&mut new_room, 2);
                }
            }
        }

        let random_pos =
            CubicVector::from(self.shape_positions[rng.gen_range(0, self.shape_positions.len())]);

        let mut start_x = new_room.range_x().start() + random_pos.x();
        let delta_x = (start_x - self.shape.range_x().start() + 1) % 2;
        start_x += delta_x;
        let end_x = new_room.range_x().end() + random_pos.x() + delta_x;

        let mut start_z = new_room.range_z().start() + random_pos.z();
        let delta_z = (start_z - self.shape.range_z().start() + 1) % 2;
        start_z += delta_z;
        let end_z = new_room.range_z().end() + random_pos.z() + delta_z;

        let start_y = new_room.range_y().start() + random_pos.y() - delta_x - delta_z;
        let end_y = new_room.range_y().end() + random_pos.y() - delta_x - delta_z;

        let is_inside_shape = self.shape.range_x().start() < start_x
            && self.shape.range_x().end() > end_x
            && self.shape.range_y().start() < start_y
            && self.shape.range_y().end() > end_y
            && self.shape.range_z().start() < start_z
            && self.shape.range_z().end() > end_z;
        let new_room = CubicRangeShape::new((start_x, end_x), (start_y, end_y), (start_z, end_z));

        if is_inside_shape && !self.rooms.iter().any(|room| room.intersects(&new_room)) {
            let mut r = 0;
            loop {
                let mut end = true;
                for pos in new_room.center().ring_iter(r) {
                    if new_room.contains_position(pos) {
                        self.hexes.get_mut(pos).expect("new room cell").0.state =
                            HexState::Open(self.next_region);
                        end = false;
                    }
                }
                if end {
                    break;
                }
                r += 1;
            }

            self.rooms.push(new_room);

            self.next_region += 1;

            self.renderer_dirty = true;
        }
    }

    pub fn start_maze(&self) -> MazeState {
        MazeState {
            next_pos: 0,
            cells: Vec::new(),
            region: 0,
        }
    }

    pub fn grow_maze(&mut self, state: &mut MazeState) -> bool {
        loop {
            let mut rng = thread_rng();
            if state.cells.is_empty() {
                let mut pos = state.next_pos;
                loop {
                    if pos < self.shape_positions.len() {
                        let cell = self.shape_positions[pos];
                        if self.can_carve(cell) {
                            state.next_pos = pos + 1;
                            state.cells.push((cell, None));
                            state.region = self.next_region;
                            self.next_region += 1;
                            break;
                        } else {
                            pos += 1;
                        }
                    } else {
                        return true;
                    }
                }
            }
            if let Some((cell, via)) = state.cells.pop() {
                if self.can_carve(cell) {
                    if let Some((via, _)) = via {
                        self.hexes.get_mut(via).expect("via cell").0.state =
                            HexState::Open(state.region);
                    }
                    self.hexes.get_mut(cell).expect("carve cell").0.state =
                        HexState::Open(state.region);
                    self.renderer_dirty = true;
                    let mut directions = Vec::new();
                    let mut wind_d = None;
                    for dir in 0..NUM_DIRECTIONS {
                        let neighbour = cell + AxialVector::direction(dir) * 2;
                        if self.can_carve(neighbour) {
                            if let Some((_, wind_dir)) = via {
                                if wind_dir == dir {
                                    wind_d = Some(directions.len())
                                }
                            }
                            directions.push(dir);
                        }
                    }
                    if !directions.is_empty() && wind_d.is_some() {
                        debug_assert_eq!(directions[wind_d.unwrap()], via.unwrap().1);
                    }
                    if !directions.is_empty() {
                        let d = wind_d
                            .and_then(|d| {
                                let windy = rng.gen_bool(0.6);
                                if windy { Some(d) } else { None }
                            })
                            .unwrap_or_else(|| rng.gen_range(0, directions.len()));
                        let dir = directions[d];
                        for (i, dir) in directions.into_iter().enumerate() {
                            if i != d {
                                let via = cell + AxialVector::direction(dir);
                                let neighbour = cell + AxialVector::direction(dir) * 2;
                                state.cells.push((neighbour, Some((via, dir))));
                            }
                        }
                        let via = cell + AxialVector::direction(dir);
                        let neighbour = cell + AxialVector::direction(dir) * 2;
                        state.cells.push((neighbour, Some((via, dir))));
                    }
                    return false;
                }
            } else {
                break;
            }
        }
        true
    }

    fn can_carve(&self, position: AxialVector) -> bool {
        let cubic = CubicVector::from(position);
        let is_inside_shape = self.shape.range_x().start() < cubic.x()
            && self.shape.range_x().end() > cubic.x()
            && self.shape.range_y().start() < cubic.y()
            && self.shape.range_y().end() > cubic.y()
            && self.shape.range_z().start() < cubic.z()
            && self.shape.range_z().end() > cubic.z();
        is_inside_shape
            && ((cubic.x() - self.shape.range_x().start()) % 2 == 1)
            && ((cubic.z() - self.shape.range_z().start()) % 2 == 1)
            && self
                .hexes
                .get(position)
                .map_or(false, |(data, _)| data.state == HexState::Wall)
    }

    pub fn start_connect(&self) -> ConnectState {
        if self.next_region <= 1 {
            return ConnectState {
                connectors: Vec::new(),
                regions_to_connect: HashSet::new(),
            };
        }
        let connectors = self
            .hexes
            .positions_and_hexes_with_adjacents()
            .filter_map(|(pos, hex_with_adjacents)| {
                if hex_with_adjacents.hex().0.state != HexState::Wall {
                    return None;
                }
                let mut regions: SmallVec<[usize; 3]> = (0..NUM_DIRECTIONS)
                    .filter_map(|dir| {
                        hex_with_adjacents
                            .adjacent(dir)
                            .and_then(|(data, _)| match data.state {
                                HexState::Open(region) => Some(region),
                                HexState::Wall => None,
                            })
                    })
                    .collect();
                regions.sort();
                regions.dedup();
                debug_assert!(regions.len() <= 3);
                if regions.len() > 1 {
                    Some((pos, regions))
                } else {
                    None
                }
            })
            .collect();
        let mut rng = thread_rng();
        let first_region = rng.gen_range(0, self.next_region);
        let regions_to_connect = (0..self.next_region)
            .filter(|region| *region != first_region)
            .collect();
        ConnectState {
            connectors,
            regions_to_connect,
        }
    }

    pub fn connect(&mut self, state: &mut ConnectState) -> bool {
        if state.regions_to_connect.is_empty() {
            return true;
        }
        let indices = state
            .connectors
            .iter()
            .enumerate()
            .filter_map(|(index, (_, connector_regions))| {
                let one_in = connector_regions
                    .iter()
                    .any(|cr| !state.regions_to_connect.contains(cr));
                let one_out = connector_regions
                    .iter()
                    .any(|cr| state.regions_to_connect.contains(cr));
                if one_in && one_out { Some(index) } else { None }
            })
            .collect::<Vec<usize>>();

        let mut rng = thread_rng();

        let (pos, regions) = &state.connectors[indices[rng.gen_range(0, indices.len())]];

        self.hexes.get_mut(*pos).expect("connector cell").0.state = HexState::Open(0);
        for r in regions {
            state.regions_to_connect.remove(r);
        }
        let connected_regions = regions.clone();
        let mut connectors = Vec::new();
        std::mem::swap(&mut state.connectors, &mut connectors);
        let (drained, remaining) = connectors.into_iter().partition(|(_, connector_regions)| {
            connector_regions
                .iter()
                .filter(|r1| connected_regions.iter().any(|r2| *r1 == r2))
                .count()
                >= 2
        });
        state.connectors = remaining;
        for (pos, _) in drained {
            let carve = rng.gen_range(0, 50) == 0;
            if carve {
                self.hexes.get_mut(pos).expect("connector cell").0.state = HexState::Open(0);
            }
        }

        self.renderer_dirty = true;

        false
    }

    pub fn start_remove_dead_ends(&self) -> RemoveDeadEndsState {
        RemoveDeadEndsState {
            tests: self
                .hexes
                .positions()
                .filter(|pos| {
                    let cubic = CubicVector::from(*pos);
                    ((cubic.x() - self.shape.range_x().start()) % 2 == 1)
                        && ((cubic.z() - self.shape.range_z().start()) % 2 == 1)
                })
                .collect(),
            next: 0,
            redo_tests: Vec::new(),
        }
    }

    pub fn remove_dead_ends(&mut self, state: &mut RemoveDeadEndsState) -> bool {
        loop {
            while state.next < state.tests.len() {
                let pos = state.tests[state.next];
                state.next += 1;
                let hex = self.hexes.get(pos);
                if let Some((
                    HexData {
                        state: HexState::Open(..),
                    },
                    _,
                )) = hex
                {
                } else {
                    continue;
                }
                let mut redo = SmallVec::<[usize; NUM_DIRECTIONS]>::new();
                for dir in 0..NUM_DIRECTIONS {
                    let via = self.hexes.get(pos + AxialVector::direction(dir));
                    let adj = self.hexes.get(pos + AxialVector::direction(dir) * 2);
                    if let (
                        Some((
                            HexData {
                                state: HexState::Open(..),
                            },
                            _,
                        )),
                        Some((
                            HexData {
                                state: HexState::Open(..),
                            },
                            _,
                        )),
                    ) = (via, adj)
                    {
                        redo.push(dir);
                    }
                }
                if redo.len() <= 1 {
                    state.redo_tests.extend(
                        redo.into_iter()
                            .map(|dir| pos + AxialVector::direction(dir) * 2),
                    );
                    let mut haa = self.hexes.hex_with_adjacents_mut(pos);
                    haa.hex().as_mut().expect("dead end cell").0.state = HexState::Wall;
                    for dir in 0..NUM_DIRECTIONS {
                        if let Some(adj) = haa.adjacent(dir) {
                            if let HexState::Open(..) = adj.0.state {
                                adj.0.state = HexState::Wall;
                            }
                        };
                    }
                    self.renderer_dirty = true;
                    return false;
                }
            }
            if !state.redo_tests.is_empty() {
                std::mem::swap(&mut state.tests, &mut state.redo_tests);
                state.redo_tests.clear();
                state.next = 0;
            } else {
                break;
            }
        }
        true
    }

    pub fn start_remove_angles(&self) -> RemoveAnglesState {
        RemoveAnglesState {
            tests: self
                .hexes
                .positions()
                .filter(|pos| {
                    let cubic = CubicVector::from(*pos);
                    ((cubic.x() - self.shape.range_x().start()) % 2 == 1)
                        && ((cubic.z() - self.shape.range_z().start()) % 2 == 1)
                })
                .collect(),
            next: 0,
            redo_tests: Vec::new(),
        }
    }

    pub fn remove_angles(&mut self, state: &mut RemoveAnglesState) -> bool {
        loop {
            while state.next < state.tests.len() {
                let pos = state.tests[state.next];
                state.next += 1;
                let hex = self.hexes.get(pos);
                if let Some((
                    HexData {
                        state: HexState::Open(..),
                    },
                    _,
                )) = hex
                {
                } else {
                    continue;
                }
                let mut redo = SmallVec::<[usize; NUM_DIRECTIONS]>::new();
                for dir in 0..NUM_DIRECTIONS {
                    let adj = self.hexes.get(pos + AxialVector::direction(dir));
                    if let Some((
                        HexData {
                            state: HexState::Open(..),
                        },
                        _,
                    )) = adj
                    {
                        redo.push(dir);
                    }
                }
                if redo.len() == 2 && (redo[0] + 1 == redo[1] || redo[0] == 0 && redo[1] == 5) {
                    let hex = self.hexes.get_mut(pos);
                    hex.expect("angle cell").0.state = HexState::Wall;
                }
            }
            if !state.redo_tests.is_empty() {
                std::mem::swap(&mut state.tests, &mut state.redo_tests);
                state.redo_tests.clear();
                state.next = 0;
            } else {
                break;
            }
        }
        true
    }

    pub fn clean_walls(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let mut remove = Vec::new();
        for (pos, haa) in self.hexes.positions_and_hexes_with_adjacents() {
            let mut keep = false;
            for dir in 0..NUM_DIRECTIONS {
                if let Some((
                    HexData {
                        state: HexState::Open(..),
                    },
                    _,
                )) = haa.adjacent(dir)
                {
                    keep = true;
                    break;
                }
            }
            if !keep {
                remove.push(pos);
            }
        }
        if !remove.is_empty() {
            for pos in remove {
                self.hexes.remove(pos).map(|mut hex| hex.dispose(data));
            }
            self.renderer_dirty = true;
        }
    }

    fn find_open_hex(&self) -> Option<AxialVector> {
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().ring_iter(r) {
                let hex_data = self.hexes.get(pos).map(|hex| &hex.0);
                match hex_data {
                    Some(HexData {
                        state: HexState::Open(..),
                        ..
                    }) => return Some(pos),
                    Some(..) => end = false,
                    None => {
                        if self.shape.contains_position(pos) {
                            end = false
                        }
                    }
                }
            }
            if end {
                return None;
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

        if let Some(hex) = self.find_open_hex() {
            let mut pointer = HexPointer::new_with_level_height(1.0);
            pointer.set_position(hex, 0, data, &world);
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
                MoveMode::StrafeLeftAhead => (pointer.direction() + 5) % 6,
                MoveMode::StrafeLeftBack => (pointer.direction() + 4) % 6,
                MoveMode::StrafeRightAhead => (pointer.direction() + 1) % 6,
                MoveMode::StrafeRightBack => (pointer.direction() + 2) % 6,
                MoveMode::StraightBack => (pointer.direction() + 3) % 6,
            };
            let next = pointer.position().neighbor(direction);
            if let Some(HexData {
                state: HexState::Open(..),
                ..
            }) = self.hexes.get(next).map(|hex| &hex.0)
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

    pub fn update_renderer_world(
        &mut self,
        force: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        if !self.renderer_dirty {
            return;
        }

        let (visible_positions, visible_only) = if let Some((pointer, fov_state)) = &self.pointer {
            let mut visible_positions = HashSet::new();
            visible_positions.insert(pointer.position());
            let mut fov = FieldOfView::default();
            fov.start(pointer.position());
            let is_obstacle = |pos| {
                let hex_data = self.hexes.get(pos).map(|hex| &hex.0);
                match hex_data {
                    Some(HexData {
                        state: HexState::Open(..),
                        ..
                    }) => false,
                    Some(HexData {
                        state: HexState::Wall,
                        ..
                    }) => true,
                    None => false,
                }
            };
            loop {
                let prev_len = visible_positions.len();
                for pos in fov.iter() {
                    let key = pointer.position() + pos;
                    if self.hexes.contains_position(key) {
                        let inserted = visible_positions.insert(key);
                        debug_assert!(inserted);
                    }
                }
                if visible_positions.len() == prev_len {
                    break;
                }
                fov.next_radius(&is_obstacle);
            }
            (
                Some(visible_positions),
                match fov_state {
                    FovState::Partial => false,
                    FovState::Full => true,
                },
            )
        } else {
            (None, false)
        };

        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();

        self.renderer.update_world(
            &mut self.hexes,
            |_, hex| !matches!(hex.0.state, HexState::Open(..)),
            |pos, _| {
                visible_positions
                    .as_ref()
                    .map_or(true, |vp| vp.contains(&pos))
            },
            |hex| &mut hex.1,
            visible_only,
            force,
            data,
            &world,
        );

        self.renderer_dirty = false;
    }
}

#[derive(Debug)]
pub struct MazeState {
    next_pos: usize,
    cells: Vec<(AxialVector, Option<(AxialVector, usize)>)>,
    region: usize,
}

#[derive(Debug)]
pub struct ConnectState {
    connectors: Vec<(AxialVector, SmallVec<[usize; 3]>)>,
    regions_to_connect: HashSet<usize>,
}

#[derive(Debug)]
pub struct RemoveDeadEndsState {
    tests: Vec<AxialVector>,
    next: usize,
    redo_tests: Vec<AxialVector>,
}

#[derive(Debug)]
pub struct RemoveAnglesState {
    tests: Vec<AxialVector>,
    next: usize,
    redo_tests: Vec<AxialVector>,
}
