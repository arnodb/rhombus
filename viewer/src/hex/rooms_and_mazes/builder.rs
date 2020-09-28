use crate::{
    hex::{
        cellular::world::FovState, rooms_and_mazes::world::World,
        shape::cubic_range::CubicRangeShape,
    },
    input::get_key_and_modifiers,
    world::RhombusViewerWorld,
};
use amethyst::{
    core::timing::Time, ecs::prelude::*, input::ElementState, prelude::*, winit::VirtualKeyCode,
};
use std::sync::Arc;

const ROOM_ROUNDS: usize = 100;

#[derive(Debug, PartialEq, Eq)]
enum BuilderState {
    Rooms(usize),
    Grown,
    FieldOfView(bool),
}

pub struct HexRoomsAndMazesBuilder {
    world: World,
    remaining_millis: u64,
    state: BuilderState,
}

impl HexRoomsAndMazesBuilder {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            remaining_millis: 0,
            state: BuilderState::Grown,
        }
    }

    fn reset(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let world_radius = 42;
        self.world.set_shape_and_reset_world(
            CubicRangeShape::new(
                (-world_radius, world_radius),
                (-world_radius, world_radius),
                (-world_radius, world_radius),
            ),
            data,
        );
        self.state = BuilderState::Rooms(ROOM_ROUNDS);
        self.remaining_millis = 0;
    }
}

impl SimpleState for HexRoomsAndMazesBuilder {
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
                    self.world.reset_world(&mut data);
                    self.state = BuilderState::Rooms(ROOM_ROUNDS);
                    self.remaining_millis = 0;
                }
                Some((VirtualKeyCode::C, ElementState::Pressed, _)) => {
                    let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
                    world.toggle_follow(&data);
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let BuilderState::FieldOfView(..) = self.state {
            self.world.update_renderer_world(data);
            self.remaining_millis = 0;
            return Trans::None;
        }
        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 5;
        self.remaining_millis = delta_millis % 5;
        for _ in 0..num {
            match self.state {
                BuilderState::Rooms(countdown) => {
                    self.world.room_phase_step(data);
                    self.state = if countdown > 1 {
                        BuilderState::Rooms(countdown - 1)
                    } else {
                        BuilderState::Grown
                    };
                }
                BuilderState::Grown => {
                    self.world.create_pointer(FovState::Partial, data);
                    self.state = BuilderState::FieldOfView(false);
                }
                BuilderState::FieldOfView(..) => {
                    break;
                }
            }
        }
        self.world.update_renderer_world(data);
        Trans::None
    }
}