use crate::color::{DARK_RED, GREEN, GREY, RED, WHITE};
use crate::demo::{Demo, DemoGraphics};
use piston_window::{Button, ButtonArgs, ButtonState, Key};
use rhombus_core::hex::coordinates::axial::AxialVector;
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::collections::{BTreeMap, BTreeSet};

const BLOCK_HEIGHT: isize = 2;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VerticalBlock {
    floor: isize,
    ceiling: isize,
}

#[derive(Debug, PartialEq, Eq)]
enum VerticalDirection {
    Horizontal,
    Up,
    Down,
}

pub struct HexBumpyBuilderDemo {
    position: CubicVector,
    height: isize,
    world: BTreeMap<(isize, isize), BTreeSet<VerticalBlock>>,
    direction: (usize, VerticalDirection),
}

impl HexBumpyBuilderDemo {
    pub fn new(position: CubicVector) -> Self {
        let mut blocks = BTreeSet::new();
        blocks.insert(VerticalBlock {
            floor: 0,
            ceiling: BLOCK_HEIGHT,
        });
        let mut world = BTreeMap::new();
        world.insert(Self::to_world_key(position), blocks);
        Self {
            position,
            height: 0,
            world,
            direction: (0, VerticalDirection::Horizontal),
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }
}

impl Demo for HexBumpyBuilderDemo {
    fn draw(&self, graphics: &dyn DemoGraphics) {
        for (pos, blocks) in &self.world {
            for block in blocks {
                for i in -2..=0 {
                    graphics.draw_hex_translate(
                        AxialVector::new(pos.0, pos.1).into(),
                        (0.0, 0.0, i as f32 * 0.1 + 0.5 * block.floor as f32),
                        1.0,
                        GREY,
                    );
                }
                graphics.draw_hex_translate(
                    AxialVector::new(pos.0, pos.1).into(),
                    (0.0, 0.0, 0.5 * block.ceiling as f32),
                    1.0,
                    DARK_RED,
                );
            }
        }
        match self.direction.1 {
            VerticalDirection::Horizontal => {
                graphics.draw_hex_arrow(
                    self.position,
                    60.0 * self.direction.0 as f32,
                    (0.0, 0.0, self.height as f32),
                    (0.0, 0.0, 0.0, 0.0),
                    WHITE,
                );
            }
            VerticalDirection::Up => {
                graphics.draw_hex_arrow(
                    self.position,
                    60.0 * self.direction.0 as f32,
                    (0.0, 0.0, self.height as f32),
                    (-10.0, 0.0, 1.0, 0.0),
                    GREEN,
                );
            }
            VerticalDirection::Down => {
                graphics.draw_hex_arrow(
                    self.position,
                    60.0 * self.direction.0 as f32,
                    (0.0, 0.0, self.height as f32),
                    (10.0, 0.0, 1.0, 0.0),
                    RED,
                );
            }
        }
    }

    fn handle_button_args(&mut self, args: &ButtonArgs) {
        if let Button::Keyboard(key) = args.button {
            match args.state {
                ButtonState::Press => match key {
                    Key::Left => self.direction.0 = (self.direction.0 + 1) % 6,
                    Key::Right => self.direction.0 = (self.direction.0 + 5) % 6,
                    Key::Up => {
                        self.direction.1 = match self.direction.1 {
                            VerticalDirection::Horizontal | VerticalDirection::Up => {
                                VerticalDirection::Up
                            }
                            VerticalDirection::Down => VerticalDirection::Horizontal,
                        }
                    }
                    Key::Down => {
                        self.direction.1 = match self.direction.1 {
                            VerticalDirection::Horizontal | VerticalDirection::Down => {
                                VerticalDirection::Down
                            }
                            VerticalDirection::Up => VerticalDirection::Horizontal,
                        }
                    }
                    Key::Space => {
                        let next_pos = self.position.neighbor(self.direction.0);
                        let next_floor = match self.direction.1 {
                            VerticalDirection::Horizontal => self.height,
                            VerticalDirection::Down => self.height - 1,
                            VerticalDirection::Up => self.height + 1,
                        };
                        let next_ceiling = next_floor + BLOCK_HEIGHT;
                        let vblock = self
                            .world
                            .entry(Self::to_world_key(next_pos))
                            .or_insert_with(|| BTreeSet::new());
                        // Really need an interval tree for that
                        let mut iter = vblock.iter();
                        enum Movement {
                            Void,
                            Go { height: isize },
                            Blocked,
                        }
                        let mut movement = Movement::Void;
                        while let Some(block) = iter.next() {
                            if (block.floor - self.height).abs() <= 1 {
                                // Just go regardless of the vertical direction
                                movement = Movement::Go {
                                    height: block.floor,
                                };
                                break;
                            }
                            if block.ceiling >= next_floor {
                                if block.floor < next_ceiling {
                                    movement = Movement::Blocked;
                                }
                                break;
                            }
                        }
                        match movement {
                            Movement::Void => {
                                vblock.insert(VerticalBlock {
                                    floor: next_floor,
                                    ceiling: next_ceiling,
                                });
                                self.position = next_pos;
                                self.height = next_floor;
                            }
                            Movement::Go { height } => {
                                self.position = next_pos;
                                self.height = height;
                            }
                            Movement::Blocked => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
