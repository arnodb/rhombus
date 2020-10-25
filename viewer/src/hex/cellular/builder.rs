use crate::{
    hex::{
        cellular::world::{FovState, MoveMode, World},
        render::renderer::HexRenderer,
        shape::cubic_range::CubicRangeShape,
    },
    input::get_key_and_modifiers,
    world::RhombusViewerWorld,
};
use amethyst::{
    core::timing::Time, ecs::prelude::*, input::ElementState, prelude::*, winit::VirtualKeyCode,
};
use std::sync::Arc;

const CELL_RADIUS_RATIO_DEN: usize = 42;
const WALL_RATIO: f32 = 0.5;

#[derive(Debug, PartialEq, Eq)]
enum CellularState {
    GrowingPhase1,
    GrowingPhase2(usize),
    Grown,
    FieldOfView(bool),
}

pub struct HexCellularBuilder<R: HexRenderer> {
    world: World<R>,
    remaining_millis: u64,
    state: CellularState,
}

impl<R: HexRenderer> HexCellularBuilder<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            world: World::new(renderer),
            remaining_millis: 0,
            state: CellularState::Grown,
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
            CELL_RADIUS_RATIO_DEN,
            WALL_RATIO,
            data,
        );
        self.state = CellularState::GrowingPhase1;
        self.remaining_millis = 0;
    }
}

impl<R: HexRenderer> SimpleState for HexCellularBuilder<R> {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        world.set_camera_distance(&data, 300.0);
        self.reset(&mut data);
        self.world.update_renderer_world(true, &mut data);
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
                    self.world
                        .reset_world(CELL_RADIUS_RATIO_DEN, WALL_RATIO, &mut data);
                    self.state = CellularState::GrowingPhase1;
                    self.remaining_millis = 0;
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
                Some((VirtualKeyCode::F, ElementState::Pressed, modifiers)) => {
                    if self.world.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_x_start
                        } else {
                            CubicRangeShape::stretch_x_start
                        },
                        CELL_RADIUS_RATIO_DEN,
                        WALL_RATIO,
                        &mut data,
                    ) {
                        self.state = CellularState::GrowingPhase1;
                        self.remaining_millis = 0;
                    }
                }
                Some((VirtualKeyCode::G, ElementState::Pressed, modifiers)) => {
                    if self.world.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_x_end
                        } else {
                            CubicRangeShape::stretch_x_end
                        },
                        CELL_RADIUS_RATIO_DEN,
                        WALL_RATIO,
                        &mut data,
                    ) {
                        self.state = CellularState::GrowingPhase1;
                        self.remaining_millis = 0;
                    }
                }
                Some((VirtualKeyCode::H, ElementState::Pressed, modifiers)) => {
                    if self.world.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_y_start
                        } else {
                            CubicRangeShape::stretch_y_start
                        },
                        CELL_RADIUS_RATIO_DEN,
                        WALL_RATIO,
                        &mut data,
                    ) {
                        self.state = CellularState::GrowingPhase1;
                        self.remaining_millis = 0;
                    }
                }
                Some((VirtualKeyCode::J, ElementState::Pressed, modifiers)) => {
                    if self.world.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_y_end
                        } else {
                            CubicRangeShape::stretch_y_end
                        },
                        CELL_RADIUS_RATIO_DEN,
                        WALL_RATIO,
                        &mut data,
                    ) {
                        self.state = CellularState::GrowingPhase1;
                        self.remaining_millis = 0;
                    }
                }
                Some((VirtualKeyCode::K, ElementState::Pressed, modifiers)) => {
                    if self.world.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_z_start
                        } else {
                            CubicRangeShape::stretch_z_start
                        },
                        CELL_RADIUS_RATIO_DEN,
                        WALL_RATIO,
                        &mut data,
                    ) {
                        self.state = CellularState::GrowingPhase1;
                        self.remaining_millis = 0;
                    }
                }
                Some((VirtualKeyCode::L, ElementState::Pressed, modifiers)) => {
                    if self.world.try_resize_shape(
                        if modifiers.shift {
                            CubicRangeShape::shrink_z_end
                        } else {
                            CubicRangeShape::stretch_z_end
                        },
                        CELL_RADIUS_RATIO_DEN,
                        WALL_RATIO,
                        &mut data,
                    ) {
                        self.state = CellularState::GrowingPhase1;
                        self.remaining_millis = 0;
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
            self.world.update_renderer_world(false, data);
            self.remaining_millis = 0;
            return Trans::None;
        }
        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 500;
        self.remaining_millis = delta_millis % 500;
        let mut force_update = false;
        for _ in 0..num {
            match self.state {
                CellularState::GrowingPhase1 => {
                    self.world.cellular_automaton_phase1_step1();
                    let frozen = self.world.cellular_automaton_phase1_step2(
                        |count| count >= 5 && count <= 6,
                        |count| count >= 3 && count <= 6,
                    );
                    if frozen {
                        self.world.expand(data);
                        force_update = true;
                        self.state = CellularState::GrowingPhase2(2);
                    }
                }
                CellularState::GrowingPhase2(countdown) => {
                    self.world.cellular_automaton_phase2_step1();
                    self.world.cellular_automaton_phase2_step2(
                        |count| count >= 3 && count <= 6,
                        |count| count >= 3 && count <= 6,
                    );
                    if countdown > 1 {
                        self.state = CellularState::GrowingPhase2(countdown - 1)
                    } else {
                        self.state = CellularState::Grown;
                    }
                }
                CellularState::Grown => {
                    self.world.create_pointer(FovState::Partial, data);
                    self.state = CellularState::FieldOfView(false);
                }
                CellularState::FieldOfView(..) => {
                    break;
                }
            }
        }
        self.world.update_renderer_world(force_update, data);
        Trans::None
    }
}
