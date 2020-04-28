use crate::color::COLORS;
use crate::demo::{Demo, DemoGraphics, Snake};
use rhombus_core::hex::coordinates::cubic::{CubicVector, RingIter};

pub struct HexSnakeDemo {
    position: CubicVector,
    snakes: Vec<Snake<CubicVector, RingIter>>,
    last_millis: u64,
}

impl HexSnakeDemo {
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            snakes: vec![Self::new_snake(position, 1), Self::new_snake(position, 3)],
            last_millis: 0,
        }
    }

    fn new_snake(position: CubicVector, radius: usize) -> Snake<CubicVector, RingIter> {
        let mut iter = Self::snake_center(position).ring_iter(radius);
        Snake {
            radius,
            state: vec![iter.next().expect("first")],
            iter,
        }
    }

    fn snake_center(position: CubicVector) -> CubicVector {
        position
    }

    fn snake_tail_size(radius: usize) -> usize {
        3 * radius
    }
}

impl Demo for HexSnakeDemo {
    fn advance(&mut self, millis: u64) {
        let num = (millis + self.last_millis % 100) / 100;
        self.last_millis += millis;
        for snake in &mut self.snakes {
            for _ in 0..num {
                if let Some(hex) = snake.iter.next() {
                    snake.state.push(hex);
                } else {
                    snake.iter = Self::snake_center(self.position).ring_iter(snake.radius);
                    let slice = snake.state.as_mut_slice();
                    let len = slice.len().min(Self::snake_tail_size(snake.radius));
                    slice.copy_within(slice.len() - len..slice.len(), 0);
                    snake.state.truncate(len);
                    snake.state.push(snake.iter.next().expect("first"));
                }
            }
        }
    }

    fn draw(&self, graphics: &dyn DemoGraphics) {
        for snake in &self.snakes {
            for (i, hex) in snake
                .state
                .iter()
                .rev()
                .take(Self::snake_tail_size(snake.radius))
                .enumerate()
            {
                graphics.draw_hex(*hex, 0.8, COLORS[i % 6]);
            }
        }
    }
}
