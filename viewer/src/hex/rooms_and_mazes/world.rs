use crate::{
    dispose::Dispose,
    hex::{
        cellular::world::FovState, pointer::HexPointer, render::renderer::HexRenderer,
        shape::cubic_range::CubicRangeShape,
    },
    world::RhombusViewerWorld,
};
use amethyst::{
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
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
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HexState {
    Open,
    Wall,
}

pub struct HexData {
    state: HexState,
}

impl Dispose for HexData {
    fn dispose(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}

const CELL_RADIUS_RATIO_DEN: usize = 42;

pub struct World<R: HexRenderer> {
    shape: CubicRangeShape,
    shape_positions: Vec<AxialVector>,
    hexes: RectHashStorage<(HexData, R::Hex)>,
    renderer: R,
    renderer_dirty: bool,
    limits_entity: Option<Entity>,
    rooms: Vec<(CubicRangeShape, Entity)>,
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
            limits_entity: None,
            rooms: Vec::new(),
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

        if let Some(entity) = self.limits_entity {
            let mut debug_lines_storage = data.world.write_storage::<DebugLinesComponent>();
            let debug_lines = debug_lines_storage.get_mut(entity).expect("Debug lines");
            debug_lines.clear();
            Self::add_limit_lines(
                &self.shape,
                Srgba::new(0.2, 0.2, 0.2, 1.0),
                debug_lines,
                &world,
            );
        } else {
            let mut debug_lines = DebugLinesComponent::with_capacity(6);
            Self::add_limit_lines(
                &self.shape,
                Srgba::new(0.2, 0.2, 0.2, 1.0),
                &mut debug_lines,
                &world,
            );
            self.limits_entity = Some(data.world.create_entity().with(debug_lines).build());
        }

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
        for (_, entity) in self.rooms.iter() {
            data.world.delete_entity(*entity).expect("delete entity");
        }
        self.rooms.clear();
        if let Some(entity) = self.limits_entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
        self.renderer.clear(data);
        self.hexes.dispose(data);
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

    fn add_limit_lines(
        shape: &CubicRangeShape,
        color: Srgba,
        debug_lines: &mut DebugLinesComponent,
        world: &RhombusViewerWorld,
    ) {
        let translations = shape
            .vertices()
            .iter()
            .map(|v| world.axial_translation((*v, 2.0).into()))
            .collect::<Vec<[f32; 3]>>();
        debug_lines.add_line(translations[0].into(), translations[1].into(), color);
        debug_lines.add_line(translations[1].into(), translations[2].into(), color);
        debug_lines.add_line(translations[2].into(), translations[3].into(), color);
        debug_lines.add_line(translations[3].into(), translations[4].into(), color);
        debug_lines.add_line(translations[4].into(), translations[5].into(), color);
        debug_lines.add_line(translations[5].into(), translations[0].into(), color);
    }

    pub fn add_room(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
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

        let random_pos: CubicVector =
            (*&self.shape_positions[rng.gen_range(0, self.shape_positions.len())]).into();

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

        if is_inside_shape
            && !self
                .rooms
                .iter()
                .any(|(room, _)| room.intersects(&new_room))
        {
            let mut debug_lines = DebugLinesComponent::with_capacity(6);
            let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
            Self::add_limit_lines(
                &new_room,
                Srgba::new(0.5, 0.5, 0.5, 1.0),
                &mut debug_lines,
                &world,
            );
            let new_entity = data.world.create_entity().with(debug_lines).build();

            let mut r = 0;
            loop {
                let mut end = true;
                for pos in new_room.center().ring_iter(r) {
                    if new_room.contains_position(pos) {
                        self.hexes.get_mut(pos).expect("new room cell").0.state = HexState::Open;
                        end = false;
                    }
                }
                if end {
                    break;
                }
                r += 1;
            }

            self.rooms.push((new_room, new_entity));

            self.renderer_dirty = true;
        }
    }

    pub fn start_maze(&self) -> MazeState {
        MazeState {
            next_pos: 0,
            cells: Vec::new(),
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
                    if let Some(via) = via {
                        self.hexes.get_mut(via).expect("via cell").0.state = HexState::Open;
                    }
                    self.hexes.get_mut(cell).expect("carve cell").0.state = HexState::Open;
                    self.renderer_dirty = true;
                    let mut directions = Vec::new();
                    for dir in 0..NUM_DIRECTIONS {
                        let neighbour = cell + AxialVector::direction(dir) * 2;
                        if self.can_carve(neighbour) {
                            directions.push(dir);
                        }
                    }
                    if !directions.is_empty() {
                        let d = rng.gen_range(0, directions.len());
                        let dir = directions[d];
                        let via = cell + AxialVector::direction(dir);
                        let neighbour = cell + AxialVector::direction(dir) * 2;
                        state.cells.push((neighbour, Some(via)));
                        for (i, dir) in directions.into_iter().enumerate() {
                            if i != d {
                                let via = cell + AxialVector::direction(dir);
                                let neighbour = cell + AxialVector::direction(dir) * 2;
                                state.cells.push((neighbour, Some(via)));
                            }
                        }
                    }
                    return false;
                }
            } else {
                break;
            }
        }
        return true;
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

    pub fn create_pointer(
        &mut self,
        _fov_state: FovState,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.delete_pointer(data, &world);

        // TODO
    }

    pub fn update_renderer_world(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
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
                        state: HexState::Open,
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
            |_, hex| hex.0.state != HexState::Open,
            |pos, _| {
                visible_positions
                    .as_ref()
                    .map_or(true, |vp| vp.contains(&pos))
            },
            |hex| &mut hex.1,
            visible_only,
            false,
            data,
            &world,
        );

        self.renderer_dirty = false;
    }
}

#[derive(Debug)]
pub struct MazeState {
    next_pos: usize,
    cells: Vec<(AxialVector, Option<AxialVector>)>,
}
