use crate::{
    dispose::Dispose,
    hex::{
        pointer::HexPointer,
        render::{
            renderer::HexRenderer,
            tile::{HexScale, TileRenderer},
        },
    },
    world::RhombusViewerWorld,
};
use amethyst::{
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::{
    coordinates::{axial::AxialVector, direction::HexagonalDirection},
    storage::hash::RectHashStorage,
};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum HexState {
    Open,
    Wall,
}

struct HexData {
    state: HexState,
}

impl Dispose for HexData {
    fn dispose(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}

pub struct HexFlatBuilderDemo {
    world: RectHashStorage<(HexData, <TileRenderer as HexRenderer>::Hex)>,
    renderer: TileRenderer,
    pointer: HexPointer,
}

impl HexFlatBuilderDemo {
    pub fn new() -> Self {
        let world = RectHashStorage::new();
        let renderer = TileRenderer::new(
            HexScale {
                horizontal: 0.8,
                vertical: 0.1,
            },
            HexScale {
                horizontal: 0.8,
                vertical: 0.3,
            },
            0,
        );
        let pointer = HexPointer::new_with_level_height(1.0);
        Self {
            world,
            renderer,
            pointer,
        }
    }

    fn raise_wall(&mut self, position: AxialVector, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.world
            .insert(
                position,
                (
                    HexData {
                        state: HexState::Wall,
                    },
                    self.renderer.new_hex(true, true),
                ),
            )
            .map(|mut hex| hex.dispose(data));
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.renderer.update_hex(
            position,
            &mut self.world.get_mut(position).unwrap().1,
            data,
            &world,
        );
    }
}

impl SimpleState for HexFlatBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.create_entities(&mut data, &world);
        self.world
            .insert(
                self.pointer.position(),
                (
                    HexData {
                        state: HexState::Open,
                    },
                    self.renderer.new_hex(false, true),
                ),
            )
            .map(|mut hex| hex.dispose(&mut data));
        self.renderer.update_hex(
            self.pointer.position(),
            &mut self.world.get_mut(self.pointer.position()).unwrap().1,
            &mut data,
            &world,
        );
    }

    fn on_stop(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.delete_entities(&mut data, &world);
        self.renderer.clear(&mut data);
        self.world.dispose(&mut data);
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
                    let new_hex = (
                        HexData {
                            state: HexState::Open,
                        },
                        self.renderer.new_hex(false, true),
                    );
                    let next_state = self
                        .world
                        .entry(next)
                        .or_insert_with(|| {
                            new = true;
                            new_hex
                        })
                        .0
                        .state;
                    if new {
                        self.renderer.update_hex(
                            next,
                            &mut self.world.get_mut(next).unwrap().1,
                            &mut data,
                            &world,
                        );
                    }
                    match next_state {
                        HexState::Open => {
                            if new {
                                // Left
                                self.raise_wall(
                                    self.pointer
                                        .position()
                                        .neighbor((self.pointer.direction() + 1) % 6),
                                    &mut data,
                                );
                                // Right
                                self.raise_wall(
                                    self.pointer
                                        .position()
                                        .neighbor((self.pointer.direction() + 5) % 6),
                                    &mut data,
                                );
                                // Ahead
                                let ahead_left = next.neighbor((self.pointer.direction() + 1) % 6);
                                let ahead = next.neighbor(self.pointer.direction());
                                let ahead_right = next.neighbor((self.pointer.direction() + 5) % 6);
                                match (
                                    self.world.get(ahead_left).map(|h| h.0.state),
                                    self.world.get(ahead).map(|h| h.0.state),
                                    self.world.get(ahead_right).map(|h| h.0.state),
                                ) {
                                    (Some(HexState::Open), _, _) | (_, _, Some(HexState::Open)) => {
                                        self.raise_wall(ahead, &mut data);
                                    }
                                    (_, Some(HexState::Open), _) => {
                                        self.raise_wall(ahead_left, &mut data);
                                        self.raise_wall(ahead_right, &mut data);
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
