use crate::color::COLORS;
use crate::demo::{Demo, DemoGraphics, Snake};
use rhombus_core::dodec::coordinates::quadric::{QuadricVector, SphereIter};

pub struct DodecSnakeDemo {
    position: QuadricVector,
    snakes: Vec<Snake<QuadricVector, SphereIter>>,
    last_millis: u64,
}

impl DodecSnakeDemo {
    pub fn new(position: QuadricVector) -> Self {
        Self {
            position,
            snakes: vec![Self::new_snake(position, 2)],
            last_millis: 0,
        }
    }

    fn new_snake(position: QuadricVector, radius: usize) -> Snake<QuadricVector, SphereIter> {
        let mut iter = Self::snake_center(position).sphere_iter(radius);
        Snake {
            radius,
            state: vec![iter.next().expect("first")],
            iter,
        }
    }

    fn snake_center(position: QuadricVector) -> QuadricVector {
        position
    }

    fn snake_tail_size(radius: usize) -> usize {
        12 * radius
    }
}

impl Demo for DodecSnakeDemo {
    fn advance(&mut self, millis: u64) {
        let num = (millis + self.last_millis % 100) / 100;
        self.last_millis += millis;
        for snake in &mut self.snakes {
            for _ in 0..num {
                if let Some(dodec) = snake.iter.next() {
                    snake.state.push(dodec);
                } else {
                    snake.iter = Self::snake_center(self.position).sphere_iter(snake.radius);
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
            for (i, dodec) in snake
                .state
                .iter()
                .rev()
                .take(Self::snake_tail_size(snake.radius))
                .enumerate()
            {
                graphics.draw_dodec(*dodec, 0.8, COLORS[i % 6]);
            }
        }
    }
}
