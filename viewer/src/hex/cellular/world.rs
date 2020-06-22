use crate::{
    dispose::Dispose,
    hex::{
        pointer::HexPointer,
        render::{
            area::AreaRenderer,
            edge::EdgeRenderer,
            renderer::HexRenderer,
            tile::{HexScale, TileRenderer},
        },
        shape::cubic_range::CubicRangeShape,
    },
    world::RhombusViewerWorld,
};
use amethyst::{
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rand::{thread_rng, RngCore};
use rhombus_core::hex::{
    coordinates::{axial::AxialVector, cubic::CubicVector, direction::HexagonalDirection},
    field_of_view::FieldOfView,
    storage::hash::RectHashStorage,
};
use std::{collections::HashSet, sync::Arc};

const HEX_SCALE_HORIZONTAL: f32 = 0.8;
const GROUND_HEX_SCALE_VERTICAL: f32 = 0.1;
const WALL_HEX_SCALE_VERTICAL: f32 = 1.0;

pub fn new_tile_renderer() -> TileRenderer {
    TileRenderer::new(
        HexScale {
            horizontal: HEX_SCALE_HORIZONTAL,
            vertical: GROUND_HEX_SCALE_VERTICAL,
        },
        HexScale {
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

pub struct World<R: HexRenderer> {
    shape: CubicRangeShape,
    cell_radius: usize,
    limits_entity: Option<Entity>,
    hexes: RectHashStorage<(HexData, R::Hex)>,
    renderer: R,
    renderer_dirty: bool,
    pointer: Option<(HexPointer, FovState)>,
}

impl<R: HexRenderer> World<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            shape: CubicRangeShape::default(),
            cell_radius: 1,
            limits_entity: None,
            hexes: RectHashStorage::new(),
            renderer,
            renderer_dirty: false,
            pointer: None,
        }
    }

    pub fn set_shape_and_reset_world(
        &mut self,
        shape: CubicRangeShape,
        cell_radius_ratio_den: usize,
        wall_ratio: f32,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        self.shape = shape;
        self.reset_world(cell_radius_ratio_den, wall_ratio, data);
    }

    pub fn reset_world(
        &mut self,
        cell_radius_ratio_den: usize,
        wall_ratio: f32,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.clear(data, &world);

        if let Some(entity) = self.limits_entity {
            let mut debug_lines_storage = data.world.write_storage::<DebugLinesComponent>();
            let debug_lines = debug_lines_storage.get_mut(entity).expect("Debug lines");
            debug_lines.clear();
            self.add_limit_lines(debug_lines, &world);
        } else {
            let mut debug_lines = DebugLinesComponent::with_capacity(6);
            self.add_limit_lines(&mut debug_lines, &world);
            self.limits_entity = Some(data.world.create_entity().with(debug_lines).build());
        }

        self.cell_radius = Self::compute_cell_radius(&self.shape, cell_radius_ratio_den);
        self.renderer.set_cell_radius(self.cell_radius);
        let mut rng = thread_rng();
        let internal_ranges = [
            self.shape.range_x().start() + 1 + self.cell_radius as isize
                ..=self.shape.range_x().end() - 1 - self.cell_radius as isize,
            self.shape.range_y().start() + 1 + self.cell_radius as isize
                ..=self.shape.range_y().end() - 1 - self.cell_radius as isize,
            self.shape.range_z().start() + 1 + self.cell_radius as isize
                ..=self.shape.range_z().end() - 1 - self.cell_radius as isize,
        ];
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().big_ring_iter(self.cell_radius, r) {
                if !pos
                    .ring_iter(self.cell_radius)
                    .any(|v| self.shape.contains(v))
                {
                    continue;
                }
                end = false;
                let cubic = CubicVector::from(pos);
                let state = if internal_ranges[0].contains(&cubic.x())
                    && internal_ranges[1].contains(&cubic.y())
                    && internal_ranges[2].contains(&cubic.z())
                {
                    if ((rng.next_u32() & 0xffff) as f32 / 0x1_0000 as f32) < wall_ratio {
                        HexState::Wall
                    } else {
                        HexState::Open
                    }
                } else {
                    HexState::HardWall
                };
                self.hexes.insert(
                    pos,
                    (
                        HexData {
                            state,
                            automaton_count: 0,
                        },
                        self.renderer.new_hex(state != HexState::Open, true),
                    ),
                );
            }
            if end {
                break;
            }
            r += 1;
        }
    }

    pub fn try_resize_shape(
        &mut self,
        resize: fn(&mut CubicRangeShape) -> bool,
        cell_radius_ratio_den: usize,
        wall_ratio: f32,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) -> bool {
        if resize(&mut self.shape) {
            self.reset_world(cell_radius_ratio_den, wall_ratio, data);
            true
        } else {
            false
        }
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
        self.renderer.clear(data);
        self.hexes.dispose(data);
        if let Some(entity) = self.limits_entity.take() {
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

    fn add_limit_lines(&self, debug_lines: &mut DebugLinesComponent, world: &RhombusViewerWorld) {
        let translations = self
            .shape
            .vertices()
            .iter()
            .map(|v| world.axial_translation((*v, 2.0).into()))
            .collect::<Vec<[f32; 3]>>();
        let color = Srgba::new(0.2, 0.2, 0.2, 1.0);
        debug_lines.add_line(translations[0].into(), translations[1].into(), color);
        debug_lines.add_line(translations[1].into(), translations[2].into(), color);
        debug_lines.add_line(translations[2].into(), translations[3].into(), color);
        debug_lines.add_line(translations[3].into(), translations[4].into(), color);
        debug_lines.add_line(translations[4].into(), translations[5].into(), color);
        debug_lines.add_line(translations[5].into(), translations[0].into(), color);
    }

    pub fn cellular_automaton_phase1_step1(&mut self) {
        for (hex_data, _) in self.hexes.hexes_mut() {
            hex_data.automaton_count = 0;
        }
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().big_ring_iter(self.cell_radius, r) {
                if let Some((
                    HexData {
                        state: hex_state, ..
                    },
                    _,
                )) = self.hexes.get(pos)
                {
                    let is_wall = match hex_state {
                        HexState::Wall | HexState::HardWall => true,
                        HexState::Open => false,
                    };
                    if is_wall {
                        for neighbor in pos.big_ring_iter(self.cell_radius, 1) {
                            if let Some((hex_data, _)) = self.hexes.get_mut(neighbor) {
                                hex_data.automaton_count += 1;
                            }
                        }
                    }
                    end = false;
                }
            }
            if end {
                break;
            }
            r += 1;
        }
    }

    pub fn cellular_automaton_phase2_step1(&mut self) {
        for (hex_data, _) in self.hexes.hexes_mut() {
            hex_data.automaton_count = 0;
        }
        let positions = self.hexes.positions().collect::<Vec<_>>();
        for pos in positions {
            let hex_state = self.hexes.get(pos).unwrap().0.state;
            let is_wall = match hex_state {
                HexState::Wall | HexState::HardWall => true,
                HexState::Open => false,
            };
            if is_wall {
                for neighbor in pos.ring_iter(1) {
                    if let Some((hex_data, _)) = self.hexes.get_mut(neighbor) {
                        hex_data.automaton_count += 1;
                    }
                }
            }
        }
    }

    pub fn cellular_automaton_step2<RaiseF, RemainF>(
        &mut self,
        raise_wall_test: RaiseF,
        remain_wall_test: RemainF,
    ) -> bool
    where
        RaiseF: Fn(u8) -> bool,
        RemainF: Fn(u8) -> bool,
    {
        let mut frozen = true;
        for (hex_data, _) in self.hexes.hexes_mut() {
            match hex_data.state {
                HexState::Wall => {
                    if !remain_wall_test(hex_data.automaton_count) {
                        hex_data.state = HexState::Open;
                        frozen = false;
                    }
                }
                HexState::Open => {
                    if raise_wall_test(hex_data.automaton_count) {
                        hex_data.state = HexState::Wall;
                        frozen = false;
                    }
                }
                HexState::HardWall => {}
            }
        }
        if !frozen {
            self.renderer_dirty = true;
        }
        frozen
    }

    pub fn expand(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if self.cell_radius <= 0 {
            return;
        }
        self.renderer.set_cell_radius(0);
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().big_ring_iter(self.cell_radius, r) {
                if let Some((
                    HexData {
                        state: hex_state, ..
                    },
                    _,
                )) = self.hexes.get(pos)
                {
                    let hex_state = *hex_state;
                    let wall = match hex_state {
                        HexState::Wall | HexState::HardWall => true,
                        HexState::Open => false,
                    };
                    if !self.shape.contains(pos) {
                        self.hexes.remove(pos).map(|mut hex| hex.dispose(data));
                    }
                    for s in 1..=self.cell_radius {
                        for sub_pos in pos.ring_iter(s) {
                            if self.shape.contains(sub_pos) {
                                self.hexes.insert(
                                    sub_pos,
                                    (
                                        HexData {
                                            state: hex_state,
                                            automaton_count: 0,
                                        },
                                        self.renderer.new_hex(wall, true),
                                    ),
                                );
                            }
                        }
                    }
                    end = false;
                }
            }
            if end {
                break;
            }
            r += 1;
        }
        self.renderer_dirty = true;
    }

    fn find_open_hex(&self) -> Option<AxialVector> {
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().ring_iter(r) {
                let hex_data = self.hexes.get(pos).map(|hex| &hex.0);
                match hex_data {
                    Some(HexData {
                        state: HexState::Open,
                        ..
                    }) => return Some(pos),
                    Some(..) => end = false,
                    None => (),
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
                state: HexState::Open,
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
            force,
            data,
            &world,
        );

        self.renderer_dirty = false;
    }
}
