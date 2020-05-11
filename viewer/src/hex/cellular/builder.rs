use crate::hex::cellular::world::World;
use amethyst::{
    core::timing::Time,
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};

#[derive(Debug, PartialEq, Eq)]
enum CellularState {
    Moving,
    Expanded,
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
}

impl SimpleState for HexCellularBuilder {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        self.world
            .reset_world(self.world_radius, self.cell_radius, 0.5, &mut data);
        self.state = CellularState::Moving;
        self.remaining_millis = 0;
    }

    fn on_stop(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        self.world.clear(&mut data);
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
                    self.world
                        .reset_world(self.world_radius, self.cell_radius, 0.5, &mut data);
                    self.state = CellularState::Moving;
                    self.remaining_millis = 0;
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.state == CellularState::Expanded {
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
                    break;
                }
            }
        }
        Trans::None
    }
}
