use crate::color::{
    Color, BLUE, CYAN, DARK_BLUE, DARK_CYAN, DARK_GREEN, DARK_MAGENTA, DARK_RED, DARK_YELLOW,
    GREEN, MAGENTA, RED, YELLOW,
};
use crate::demo::{Demo, DemoGraphics};
use rhombus_core::dodec::coordinates::quadric::QuadricVector;

pub struct DodecDirectionsDemo {
    position: QuadricVector,
}

impl DodecDirectionsDemo {
    pub fn new(position: QuadricVector) -> Self {
        Self { position }
    }

    fn draw_direction(
        graphics: &dyn DemoGraphics,
        mut origin: QuadricVector,
        direction: usize,
        length: usize,
        color: Color,
    ) {
        for _ in 0..length {
            origin = origin.neighbor(direction);
            graphics.draw_dodec(origin, 0.3, color);
        }
    }
}

impl Demo for DodecDirectionsDemo {
    fn draw(&self, graphics: &dyn DemoGraphics) {
        Self::draw_direction(graphics, self.position, 0, 3, DARK_RED);
        Self::draw_direction(graphics, self.position, 6, 2, RED);

        Self::draw_direction(graphics, self.position, 1, 3, DARK_GREEN);
        Self::draw_direction(graphics, self.position, 7, 2, GREEN);

        Self::draw_direction(graphics, self.position, 2, 3, DARK_BLUE);
        Self::draw_direction(graphics, self.position, 8, 2, BLUE);

        Self::draw_direction(graphics, self.position, 3, 3, DARK_YELLOW);
        Self::draw_direction(graphics, self.position, 9, 2, YELLOW);

        Self::draw_direction(graphics, self.position, 4, 3, DARK_MAGENTA);
        Self::draw_direction(graphics, self.position, 10, 2, MAGENTA);

        Self::draw_direction(graphics, self.position, 5, 3, DARK_CYAN);
        Self::draw_direction(graphics, self.position, 11, 2, CYAN);
    }
}
