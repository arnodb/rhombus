use crate::{
    dispose::Dispose,
    hex::{pointer::HexPointer, render::renderer::HexRenderer},
    world::RhombusViewerWorld,
};
use amethyst::{ecs::prelude::*, prelude::*};
use rhombus_core::hex::{
    coordinates::{axial::AxialVector, direction::HexagonalDirection},
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

enum CustomMode {
    Hex(usize),
    Corridor,
}

const MODES: [CustomMode; 3] = [CustomMode::Hex(0), CustomMode::Hex(1), CustomMode::Corridor];

pub struct World<R: HexRenderer> {
    hexes: RectHashStorage<(HexData, R::Hex)>,
    renderer: R,
    renderer_dirty: bool,
    pointer: Option<(HexPointer, FovState)>,
    mode: usize,
}

impl<R: HexRenderer> World<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            hexes: RectHashStorage::new(),
            renderer,
            renderer_dirty: false,
            pointer: None,
            mode: 0,
        }
    }

    pub fn reset_world(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.clear(data, &world);
    }

    pub fn clear(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.delete_pointer(data, world);
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

    pub fn next_mode(&mut self) {
        self.mode = (self.mode + 1) % MODES.len();
    }

    pub fn grow_custom(&mut self) {
        match MODES[self.mode] {
            CustomMode::Hex(radius) => self.grow_hex(radius),
            CustomMode::Corridor => self.grow_corridor(),
        }
    }

    fn grow_hex(&mut self, radius: usize) {
        for r in 0..=radius {
            for pos in AxialVector::default().ring_iter(r) {
                self.hexes.insert(
                    pos,
                    (
                        HexData {
                            state: HexState::Open,
                        },
                        self.renderer.new_hex(false, true),
                    ),
                );
            }
        }
        for pos in AxialVector::default().ring_iter(radius + 1) {
            self.hexes.insert(
                pos,
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

    fn grow_corridor(&mut self) {
        for (q, r) in [(0, 0), (1, 0)].iter() {
            self.hexes.insert(
                AxialVector::new(*q, *r),
                (
                    HexData {
                        state: HexState::Open,
                    },
                    self.renderer.new_hex(false, true),
                ),
            );
        }
        for (q, r) in [(0, 1), (1, 1), (2, 0), (2, -1), (2, -1), (1, -1)].iter() {
            self.hexes.insert(
                AxialVector::new(*q, *r),
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

    fn find_open_hex(&self) -> Option<AxialVector> {
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in AxialVector::default().ring_iter(r) {
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
