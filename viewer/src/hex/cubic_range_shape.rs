use crate::{
    dispose::Dispose,
    hex::{
        pointer::{HexPointer, VerticalDirection},
        render::{
            renderer::HexRenderer,
            tile::{HexScale, TileRenderer},
        },
        shape::cubic_range::CubicRangeShape,
    },
    input::get_key_and_modifiers,
    world::RhombusViewerWorld,
};
use amethyst::{ecs::prelude::*, input::ElementState, prelude::*, winit::VirtualKeyCode};
use rhombus_core::hex::{
    coordinates::direction::HexagonalDirection, storage::hash::RectHashStorage,
};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveMode {
    StraightAhead,
    StrafeLeftAhead,
    StrafeLeftBack,
    StrafeRightAhead,
    StrafeRightBack,
    StraightBack,
}

pub struct HexCubicRangeShapeDemo {
    shape: CubicRangeShape,
    world: RectHashStorage<<TileRenderer as HexRenderer>::Hex>,
    renderer: TileRenderer,
    pointer: HexPointer,
}

impl HexCubicRangeShapeDemo {
    pub fn new() -> Self {
        let shape = CubicRangeShape::new(-2..=2, -2..=2, -2..=2);
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
            shape,
            world,
            renderer,
            pointer,
        }
    }

    fn reset_shape(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        self.renderer.clear(data);
        self.world.dispose(data);
        let position = self.shape.center();
        self.pointer.set_position(position, 0, data, world);
        if self.shape.contains(position) {
            self.pointer.set_direction(
                self.pointer.direction(),
                VerticalDirection::Horizontal,
                data,
                world,
            );
        } else {
            self.pointer.set_direction(
                self.pointer.direction(),
                VerticalDirection::Down,
                data,
                world,
            );
        }
        for v in self.shape.vertices().iter() {
            self.world
                .insert(*v, self.renderer.new_hex(true, true))
                .map(|mut hex| hex.dispose(data));
            self.renderer
                .update_hex(*v, &mut self.world.get_mut(*v).unwrap(), data, world);
        }
        let center = self.shape.center();
        self.world
            .insert(center, self.renderer.new_hex(false, true))
            .map(|mut hex| hex.dispose(data));
        self.renderer.update_hex(
            center,
            &mut self.world.get_mut(center).unwrap(),
            data,
            world,
        );
    }

    fn try_resize_shape(
        &mut self,
        resize: fn(&mut CubicRangeShape) -> bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        resize(&mut self.shape);
        self.reset_shape(data, world);
    }

    fn next_position(&mut self, mode: MoveMode, data: &mut StateData<'_, GameData<'_, '_>>) {
        let direction = match mode {
            MoveMode::StraightAhead => self.pointer.direction(),
            MoveMode::StrafeLeftAhead => (self.pointer.direction() + 5) % 6,
            MoveMode::StrafeLeftBack => (self.pointer.direction() + 4) % 6,
            MoveMode::StrafeRightAhead => (self.pointer.direction() + 1) % 6,
            MoveMode::StrafeRightBack => (self.pointer.direction() + 2) % 6,
            MoveMode::StraightBack => (self.pointer.direction() + 3) % 6,
        };
        let next = self.pointer.position().neighbor(direction);
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.set_position(next, 0, data, &world);
        if self.shape.contains(next) {
            self.pointer.set_direction(
                self.pointer.direction(),
                VerticalDirection::Horizontal,
                data,
                &world,
            );
        } else {
            self.pointer.set_direction(
                self.pointer.direction(),
                VerticalDirection::Down,
                data,
                &world,
            );
        }
    }
}

impl SimpleState for HexCubicRangeShapeDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.pointer.create_entities(&mut data, &world);
        self.reset_shape(&mut data, &world);
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
            match get_key_and_modifiers(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed, _)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed, _)) => {
                    self.pointer.increment_direction(&data, &world);
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed, _)) => {
                    self.pointer.decrement_direction(&data, &world);
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed, _)) => {
                    self.next_position(MoveMode::StraightAhead, &mut data);
                }
                Some((VirtualKeyCode::Down, ElementState::Pressed, _)) => {
                    self.next_position(MoveMode::StraightBack, &mut data);
                }
                Some((VirtualKeyCode::F, ElementState::Pressed, modifiers)) => {
                    self.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_x_start
                        } else {
                            CubicRangeShape::stretch_x_start
                        },
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::G, ElementState::Pressed, modifiers)) => {
                    self.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_x_end
                        } else {
                            CubicRangeShape::stretch_x_end
                        },
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::H, ElementState::Pressed, modifiers)) => {
                    self.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_y_start
                        } else {
                            CubicRangeShape::stretch_y_start
                        },
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::J, ElementState::Pressed, modifiers)) => {
                    self.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_y_end
                        } else {
                            CubicRangeShape::stretch_y_end
                        },
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::K, ElementState::Pressed, modifiers)) => {
                    self.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_z_start
                        } else {
                            CubicRangeShape::stretch_z_start
                        },
                        &mut data,
                        &world,
                    );
                }
                Some((VirtualKeyCode::L, ElementState::Pressed, modifiers)) => {
                    self.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_z_end
                        } else {
                            CubicRangeShape::stretch_z_end
                        },
                        &mut data,
                        &world,
                    );
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }
}
