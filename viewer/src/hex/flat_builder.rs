use crate::{
    hex::{
        pointer::HexPointer,
        render::tile::{CellScale, TileRenderer},
    },
    world::RhombusViewerWorld,
};
use amethyst::{
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{axial::AxialVector, direction::HexagonalDirection};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum HexState {
    Open,
    Wall,
}

struct HexData {
    state: HexState,
}

pub struct HexFlatBuilderDemo {
    world: BTreeMap<AxialVector, HexData>,
    tile_renderer: TileRenderer,
    pointer: HexPointer,
}

impl HexFlatBuilderDemo {
    pub fn new() -> Self {
        let world = BTreeMap::new();
        let tile_renderer = TileRenderer::new(
            CellScale {
                horizontal: 0.8,
                vertical: 0.1,
            },
            CellScale {
                horizontal: 0.8,
                vertical: 0.3,
            },
        );
        let pointer = HexPointer::new_with_level_height(1.0);
        Self {
            world,
            tile_renderer,
            pointer,
        }
    }

    fn raise_wall(
        &mut self,
        position: AxialVector,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.world.insert(
            position,
            HexData {
                state: HexState::Wall,
            },
        );
        self.tile_renderer
            .insert_cell(position, true, true, data, world);
    }
}

impl SimpleState for HexFlatBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.create_entities(&mut data, &world);
        self.world.insert(
            self.pointer.position(),
            HexData {
                state: HexState::Open,
            },
        );
        self.tile_renderer
            .insert_cell(self.pointer.position(), false, true, &mut data, &world);
    }

    fn on_stop(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.delete_entities(&mut data, &world);
        self.tile_renderer.clear(&mut data);
        self.world.clear();
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            let mut trans = Trans::None;
            let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
            match get_key(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.pointer.increment_direction(&data, &world);
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.pointer.decrement_direction(&data, &world);
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    let next = self.pointer.position().neighbor(self.pointer.direction());
                    let mut new = false;
                    let next_state = self.world.entry(next).or_insert_with(|| {
                        new = true;
                        HexData {
                            state: HexState::Open,
                        }
                    });
                    if new {
                        self.tile_renderer
                            .insert_cell(next, false, true, &mut data, &world);
                    }
                    match next_state.state {
                        HexState::Open => {
                            if new {
                                // Left
                                self.raise_wall(
                                    self.pointer
                                        .position()
                                        .neighbor((self.pointer.direction() + 1) % 6),
                                    &mut data,
                                    &world,
                                );
                                // Right
                                self.raise_wall(
                                    self.pointer
                                        .position()
                                        .neighbor((self.pointer.direction() + 5) % 6),
                                    &mut data,
                                    &world,
                                );
                                // Ahead
                                let ahead_left = next.neighbor((self.pointer.direction() + 1) % 6);
                                let ahead = next.neighbor(self.pointer.direction());
                                let ahead_right = next.neighbor((self.pointer.direction() + 5) % 6);
                                match (
                                    self.world.get(&ahead_left).map(|h| h.state),
                                    self.world.get(&ahead).map(|h| h.state),
                                    self.world.get(&ahead_right).map(|h| h.state),
                                ) {
                                    (Some(HexState::Open), _, _) | (_, _, Some(HexState::Open)) => {
                                        self.raise_wall(ahead, &mut data, &world);
                                    }
                                    (_, Some(HexState::Open), _) => {
                                        self.raise_wall(ahead_left, &mut data, &world);
                                        self.raise_wall(ahead_right, &mut data, &world);
                                    }
                                    _ => {}
                                }
                            }
                            self.pointer.set_position(next, 0, &data, &world);
                        }
                        HexState::Wall => {}
                    }
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }
}
