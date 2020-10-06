use crate::{
    hex::{
        new_area_renderer, new_edge_renderer, new_square_renderer, new_tile_renderer,
        render::{
            area::AreaRenderer, edge::EdgeRenderer, renderer::HexRenderer, square::SquareRenderer,
            tile::TileRenderer,
        },
        rooms_and_mazes::world::{
            ConnectState, FovState, MazeState, MoveMode, RemoveDeadEndsState, World,
        },
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

#[derive(Debug)]
enum BuilderState {
    Rooms(usize),
    Maze(MazeState),
    Connect(ConnectState),
    RemoveDeadEnds(RemoveDeadEndsState),
    Grown,
    FieldOfView(bool),
}

pub struct HexRoomsAndMazesBuilder<R: HexRenderer> {
    world: World<R>,
    remaining_millis: u64,
    state: BuilderState,
}

impl<R: HexRenderer> HexRoomsAndMazesBuilder<R> {
    fn new(renderer: R) -> Self {
        Self {
            world: World::new(renderer),
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

impl<R: HexRenderer> SimpleState for HexRoomsAndMazesBuilder<R> {
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
                    if let BuilderState::FieldOfView(mut fov_enabled) = self.state {
                        fov_enabled = !fov_enabled;
                        self.world.change_field_of_view(if fov_enabled {
                            FovState::Full
                        } else {
                            FovState::Partial
                        });
                        self.state = BuilderState::FieldOfView(fov_enabled);
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
        if let BuilderState::FieldOfView(..) = self.state {
            self.world.update_renderer_world(false, data);
            self.remaining_millis = 0;
            return Trans::None;
        }
        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 5;
        self.remaining_millis = delta_millis % 5;
        let mut force_update = false;
        for _ in 0..num {
            match &mut self.state {
                BuilderState::Rooms(countdown) => {
                    self.world.add_room();
                    self.state = if *countdown > 1 {
                        BuilderState::Rooms(*countdown - 1)
                    } else {
                        BuilderState::Maze(self.world.start_maze())
                    };
                }
                BuilderState::Maze(state) => {
                    if self.world.grow_maze(state) {
                        self.state = BuilderState::Connect(self.world.start_connect());
                    }
                }
                BuilderState::Connect(state) => {
                    if self.world.connect(state) {
                        self.state =
                            BuilderState::RemoveDeadEnds(self.world.start_remove_dead_ends());
                    }
                }
                BuilderState::RemoveDeadEnds(state) => {
                    if self.world.remove_dead_ends(state) {
                        self.world.clean_walls(data);
                        force_update = true;
                        self.state = BuilderState::Grown;
                    }
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
        self.world.update_renderer_world(force_update, data);
        Trans::None
    }
}

impl HexRoomsAndMazesBuilder<TileRenderer> {
    pub fn new_tile() -> Self {
        Self::new(new_tile_renderer())
    }
}

impl HexRoomsAndMazesBuilder<SquareRenderer> {
    pub fn new_square() -> Self {
        Self::new(new_square_renderer())
    }
}

impl HexRoomsAndMazesBuilder<EdgeRenderer> {
    pub fn new_edge() -> Self {
        Self::new(new_edge_renderer())
    }
}

impl HexRoomsAndMazesBuilder<AreaRenderer> {
    pub fn new_area() -> Self {
        Self::new(new_area_renderer())
    }
}
