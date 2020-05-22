use crate::{
    hex::cellular::world::{FovState, World},
    world::RhombusViewerWorld,
};
use amethyst::{
    core::timing::Time,
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
enum CellularState {
    Moving,
    Expanded,
    FieldOfView(bool),
}

pub struct HexCellularBuilder {
    world: World,
    world_radius: usize,
    cell_radius: usize,
    remaining_millis: u64,
    state: CellularState,
}

impl HexCellularBuilder {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            world_radius: 12,
            cell_radius: 2,
            remaining_millis: 0,
            state: CellularState::Expanded,
        }
    }

    fn reset(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.world
            .reset_world(self.world_radius, self.cell_radius, 0.5, data);
        self.state = CellularState::Moving;
        self.remaining_millis = 0;
    }
}

impl SimpleState for HexCellularBuilder {
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
            match get_key(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::N, ElementState::Pressed)) => {
                    self.reset(&mut data);
                }
                Some((VirtualKeyCode::Key8, ElementState::Pressed)) => {
                    if self.cell_radius < 12 {
                        self.cell_radius += 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Key7, ElementState::Pressed)) => {
                    if self.cell_radius > 0 {
                        self.cell_radius -= 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Key0, ElementState::Pressed)) => {
                    if self.world_radius < 42 {
                        self.world_radius += 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Key9, ElementState::Pressed)) => {
                    if self.world_radius > 0 {
                        self.world_radius -= 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.world.increment_direction(&mut data);
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.world.decrement_direction(&mut data);
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    self.world.next_position(&mut data);
                }
                Some((VirtualKeyCode::C, ElementState::Pressed)) => {
                    let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
                    world.toggle_follow(&data);
                }
                Some((VirtualKeyCode::F, ElementState::Pressed)) => {
                    if let CellularState::FieldOfView(mut fov_enabled) = self.state {
                        fov_enabled = !fov_enabled;
                        self.world.change_field_of_view(
                            if fov_enabled {
                                FovState::Full
                            } else {
                                FovState::Partial
                            },
                            &mut data,
                        );
                        self.state = CellularState::FieldOfView(fov_enabled);
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
        if let CellularState::FieldOfView(..) = self.state {
            self.remaining_millis = 0;
            return Trans::None;
        }
        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 500;
        self.remaining_millis = delta_millis % 500;
        for _ in 0..num {
            match self.state {
                CellularState::Moving => {
                    let frozen = self.world.apply_cellular_automaton(
                        self.world_radius,
                        self.cell_radius,
                        |count| count >= 5 && count <= 6,
                        |count| count >= 3 && count <= 6,
                        data,
                    );
                    if frozen {
                        self.world.expand(self.world_radius, self.cell_radius, data);
                        self.state = CellularState::Expanded;
                    }
                }
                CellularState::Expanded => {
                    self.world.create_pointer(FovState::Full, data);
                    self.state = CellularState::FieldOfView(true);
                }
                CellularState::FieldOfView(..) => {
                    break;
                }
            }
        }
        Trans::None
    }
}
