use crate::color::{DARK_RED, GREEN, GREY, RED, WHITE};
use crate::demo::{Demo, DemoGraphics};
use piston_window::{Button, ButtonArgs, ButtonState, Key};
use rhombus_core::hex::coordinates::axial::AxialVector;
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
enum HexState {
    Open,
    Wall,
}

pub struct HexFlatBuilderDemo {
    position: CubicVector,
    world: BTreeMap<(isize, isize), HexState>,
    direction: usize,
}

impl HexFlatBuilderDemo {
    pub fn new(position: CubicVector) -> Self {
        let mut world = BTreeMap::new();
        world.insert(Self::to_world_key(position), HexState::Open);
        Self {
            position,
            world,
            direction: 0,
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn wallize(&mut self, pos: CubicVector) {
        self.world
            .entry(Self::to_world_key(pos))
            .or_insert_with(|| HexState::Wall);
    }
}

impl Demo for HexFlatBuilderDemo {
    fn draw(&self, graphics: &dyn DemoGraphics) {
        for (pos, state) in &self.world {
            match state {
                HexState::Open => {
                    graphics.draw_hex(AxialVector::new(pos.0, pos.1).into(), 1.0, GREY);
                }
                HexState::Wall => {
                    graphics.draw_hex(AxialVector::new(pos.0, pos.1).into(), 1.0, DARK_RED);
                }
            }
        }
        let ahead = self
            .world
            .get(&Self::to_world_key(self.position.neighbor(self.direction)));
        let color = match ahead {
            Some(HexState::Wall) => RED,
            Some(HexState::Open) => WHITE,
            None => GREEN,
        };
        graphics.draw_hex_arrow(self.position, 60.0 * self.direction as f32, color)
    }

    fn handle_button_args(&mut self, args: &ButtonArgs) {
        if let Button::Keyboard(key) = args.button {
            match args.state {
                ButtonState::Press => match key {
                    Key::Left => self.direction = (self.direction + 1) % 6,
                    Key::Right => self.direction = (self.direction + 5) % 6,
                    Key::Up => {
                        let next = self.position.neighbor(self.direction);
                        let mut new = false;
                        let next_state =
                            self.world
                                .entry(Self::to_world_key(next))
                                .or_insert_with(|| {
                                    new = true;
                                    HexState::Open
                                });
                        match next_state {
                            HexState::Open => {
                                if new {
                                    // Left
                                    self.wallize(self.position.neighbor((self.direction + 1) % 6));
                                    // Right
                                    self.wallize(self.position.neighbor((self.direction + 5) % 6));
                                    // Ahead
                                    let ahead_left = next.neighbor((self.direction + 1) % 6);
                                    let ahead = next.neighbor(self.direction);
                                    let ahead_right = next.neighbor((self.direction + 5) % 6);
                                    match (
                                        self.world.get(&Self::to_world_key(ahead_left)),
                                        self.world.get(&Self::to_world_key(ahead)),
                                        self.world.get(&Self::to_world_key(ahead_right)),
                                    ) {
                                        (Some(HexState::Open), _, _)
                                        | (_, _, Some(HexState::Open)) => {
                                            self.wallize(ahead);
                                        }
                                        (_, Some(HexState::Open), _) => {
                                            self.wallize(ahead_left);
                                            self.wallize(ahead_right);
                                        }
                                        _ => {}
                                    }
                                }
                                self.position = next;
                            }
                            HexState::Wall => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
