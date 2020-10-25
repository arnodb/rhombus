use crate::{
    hex::{
        custom::world::{FovState, MoveMode, World},
        render::renderer::HexRenderer,
    },
    input::get_key_and_modifiers,
    world::RhombusViewerWorld,
};
use amethyst::{ecs::prelude::*, input::ElementState, prelude::*, winit::VirtualKeyCode};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
enum CustomState {
    Growing,
    Grown,
    FieldOfView(bool),
}

pub struct HexCustomBuilder<R: HexRenderer> {
    world: World<R>,
    state: CustomState,
}

impl<R: HexRenderer> HexCustomBuilder<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            world: World::new(renderer),
            state: CustomState::Grown,
        }
    }

    fn reset(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.world.reset_world(data);
        self.state = CustomState::Growing;
    }
}

impl<R: HexRenderer> SimpleState for HexCustomBuilder<R> {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        world.set_camera_distance(&data, 300.0);
        self.reset(&mut data);
    }

    fn on_stop(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.world.clear(&mut data, &world);
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            let mut trans = Trans::None;
            match get_key_and_modifiers(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed, _)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::N, ElementState::Pressed, _)) => {
                    self.world.next_mode();
                    self.world.reset_world(&mut data);
                    self.state = CustomState::Growing;
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed, modifiers)) => {
                    if modifiers.shift {
                        self.world
                            .next_position(MoveMode::StrafeRightAhead, &mut data);
                    } else if modifiers.ctrl {
                        self.world
                            .next_position(MoveMode::StrafeRightBack, &mut data);
                    } else {
                        self.world.increment_direction(&data);
                    }
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed, modifiers)) => {
                    if modifiers.shift {
                        self.world
                            .next_position(MoveMode::StrafeLeftAhead, &mut data);
                    } else if modifiers.ctrl {
                        self.world
                            .next_position(MoveMode::StrafeLeftBack, &mut data);
                    } else {
                        self.world.decrement_direction(&data);
                    }
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed, _)) => {
                    self.world.next_position(MoveMode::StraightAhead, &mut data);
                }
                Some((VirtualKeyCode::Down, ElementState::Pressed, _)) => {
                    self.world.next_position(MoveMode::StraightBack, &mut data);
                }
                Some((VirtualKeyCode::C, ElementState::Pressed, _)) => {
                    let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
                    world.toggle_follow(&data);
                }
                Some((VirtualKeyCode::V, ElementState::Pressed, _)) => {
                    if let CustomState::FieldOfView(mut fov_enabled) = self.state {
                        fov_enabled = !fov_enabled;
                        self.world.change_field_of_view(if fov_enabled {
                            FovState::Full
                        } else {
                            FovState::Partial
                        });
                        self.state = CustomState::FieldOfView(fov_enabled);
                    }
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let CustomState::FieldOfView(..) = self.state {
            self.world.update_renderer_world(false, data);
            return Trans::None;
        }
        let mut force_update = false;
        match self.state {
            CustomState::Growing => {
                self.world.grow_custom();
                force_update = true;
                self.state = CustomState::Grown;
            }
            CustomState::Grown => {
                self.world.create_pointer(FovState::Partial, data);
                self.state = CustomState::FieldOfView(false);
            }
            CustomState::FieldOfView(..) => {}
        }
        self.world.update_renderer_world(force_update, data);
        Trans::None
    }
}
