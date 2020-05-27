use crate::{
    hex::{
        cellular::world::{
            new_area_renderer, new_edge_renderer, new_tile_renderer, FovState, MoveMode, World,
        },
        render::{
            area::AreaRenderer, edge::EdgeRenderer, renderer::HexRenderer, tile::TileRenderer,
        },
    },
    input::get_key_and_modifiers,
    world::RhombusViewerWorld,
};
use amethyst::{
    core::timing::Time, ecs::prelude::*, input::ElementState, prelude::*, winit::VirtualKeyCode,
};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
enum CellularState {
    GrowingPhase1,
    GrowingPhase2(usize),
    Grown,
    FieldOfView(bool),
}

pub struct HexCellularBuilder<R: HexRenderer> {
    world: World<R>,
    world_radius: usize,
    cell_radius: usize,
    remaining_millis: u64,
    state: CellularState,
}

impl<R: HexRenderer> HexCellularBuilder<R> {
    fn new(renderer: R) -> Self {
        Self {
            world: World::new(renderer),
            world_radius: 12,
            cell_radius: 2,
            remaining_millis: 0,
            state: CellularState::Grown,
        }
    }

    fn reset(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.world
            .reset_world(self.world_radius, self.cell_radius, 0.5, data);
        self.state = CellularState::GrowingPhase1;
        self.remaining_millis = 0;
    }
}

impl<R: HexRenderer> SimpleState for HexCellularBuilder<R> {
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
                    self.reset(&mut data);
                }
                Some((VirtualKeyCode::Key8, ElementState::Pressed, _)) => {
                    if self.cell_radius < 12 {
                        self.cell_radius += 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Key7, ElementState::Pressed, _)) => {
                    if self.cell_radius > 0 {
                        self.cell_radius -= 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Key0, ElementState::Pressed, _)) => {
                    if self.world_radius < 42 {
                        self.world_radius += 1;
                        self.reset(&mut data);
                    }
                }
                Some((VirtualKeyCode::Key9, ElementState::Pressed, _)) => {
                    if self.world_radius > 0 {
                        self.world_radius -= 1;
                        self.reset(&mut data);
                    }
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
                Some((VirtualKeyCode::F, ElementState::Pressed, _)) => {
                    if let CellularState::FieldOfView(mut fov_enabled) = self.state {
                        fov_enabled = !fov_enabled;
                        self.world.change_field_of_view(if fov_enabled {
                            FovState::Full
                        } else {
                            FovState::Partial
                        });
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
            self.world.update_renderer_world(data);
            self.world.update_renderer(data);
            self.remaining_millis = 0;
            return Trans::None;
        }
        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 500;
        self.remaining_millis = delta_millis % 500;
        let mut update_renderer_world = false;
        let mut update_renderer = false;
        for _ in 0..num {
            match self.state {
                CellularState::GrowingPhase1 => {
                    self.world
                        .cellular_automaton_phase1_step1(self.world_radius, self.cell_radius);
                    let frozen = self.world.cellular_automaton_step2(
                        |count| count >= 5 && count <= 6,
                        |count| count >= 3 && count <= 6,
                        data,
                    );
                    if frozen {
                        self.world.expand(self.world_radius, self.cell_radius, data);
                        self.state = CellularState::GrowingPhase2(2);
                    }
                    update_renderer = true;
                }
                CellularState::GrowingPhase2(countdown) => {
                    self.world.cellular_automaton_phase2_step1();
                    self.world.cellular_automaton_step2(
                        |count| count >= 3 && count <= 6,
                        |count| count >= 3 && count <= 6,
                        data,
                    );
                    if countdown > 1 {
                        self.state = CellularState::GrowingPhase2(countdown - 1)
                    } else {
                        self.state = CellularState::Grown;
                    }
                    update_renderer = true;
                }
                CellularState::Grown => {
                    self.world.create_pointer(FovState::Full, data);
                    self.state = CellularState::FieldOfView(true);
                }
                CellularState::FieldOfView(..) => {
                    update_renderer_world = true;
                    break;
                }
            }
        }
        if update_renderer_world {
            self.world.update_renderer_world(data);
            update_renderer = true;
        }
        if update_renderer {
            self.world.update_renderer(data);
        }
        Trans::None
    }
}

impl HexCellularBuilder<TileRenderer> {
    pub fn new_tile() -> Self {
        Self::new(new_tile_renderer())
    }
}

impl HexCellularBuilder<EdgeRenderer> {
    pub fn new_edge() -> Self {
        Self::new(new_edge_renderer())
    }
}

impl HexCellularBuilder<AreaRenderer> {
    pub fn new_area() -> Self {
        Self::new(new_area_renderer())
    }
}
