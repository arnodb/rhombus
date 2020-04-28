use crate::color::{Color, BLUE, DARK_BLUE, DARK_GREEN, DARK_RED, GREEN, RED};
use crate::demo::{Demo, DemoGraphics};
use rhombus_core::hex::coordinates::cubic::CubicVector;

pub struct HexDirectionsDemo {
    position: CubicVector,
}

impl HexDirectionsDemo {
    pub fn new(position: CubicVector) -> Self {
        Self { position }
    }

    fn draw_direction(
        graphics: &dyn DemoGraphics,
        mut origin: CubicVector,
        direction: usize,
        length: usize,
        color: Color,
    ) {
        for _ in 0..length {
            origin = origin.neighbor(direction);
            graphics.draw_hex(origin, 0.3, color);
        }
    }
}

impl Demo for HexDirectionsDemo {
    fn draw(&self, graphics: &dyn DemoGraphics) {
        Self::draw_direction(graphics, self.position, 0, 3, DARK_RED);
        Self::draw_direction(graphics, self.position, 3, 2, RED);

        Self::draw_direction(graphics, self.position, 1, 3, DARK_GREEN);
        Self::draw_direction(graphics, self.position, 4, 2, GREEN);

        Self::draw_direction(graphics, self.position, 2, 3, DARK_BLUE);
        Self::draw_direction(graphics, self.position, 5, 2, BLUE);
    }
}
