use crate::color::{DARK_RED, GREY, WHITE};
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
        let axial = AxialVector::from(position);
        world.insert((axial.q(), axial.r()), HexState::Open);
        Self {
            position,
            world,
            direction: 0,
        }
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
        graphics.draw_hex_arrow(self.position, 60.0 * self.direction as f32, WHITE)
    }

    fn handle_button_args(&mut self, args: &ButtonArgs) {
        if let Button::Keyboard(key) = args.button {
            match args.state {
                ButtonState::Press => match key {
                    Key::Left => self.direction = (self.direction + 1) % 6,
                    Key::Right => self.direction = (self.direction + 5) % 6,
                    Key::Up => {
                        let next = self.position.neighbor(self.direction);
                        let next_axial = AxialVector::from(next);
                        let mut new = false;
                        let next_state = self
                            .world
                            .entry((next_axial.q(), next_axial.r()))
                            .or_insert_with(|| {
                                new = true;
                                HexState::Open
                            });
                        match next_state {
                            HexState::Open => {
                                if new {
                                    // Left
                                    let side = AxialVector::from(
                                        self.position.neighbor((self.direction + 1) % 6),
                                    );
                                    self.world
                                        .entry((side.q(), side.r()))
                                        .or_insert_with(|| HexState::Wall);
                                    // Right
                                    let side = AxialVector::from(
                                        self.position.neighbor((self.direction + 5) % 6),
                                    );
                                    self.world
                                        .entry((side.q(), side.r()))
                                        .or_insert_with(|| HexState::Wall);
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
