use crate::color::WHITE;
use crate::demo::{Demo, DemoGraphics};
use rhombus_core::hex::coordinates::cubic::CubicVector;

pub struct HexRingDemo {
    position: CubicVector,
    rings: Vec<usize>,
}

impl HexRingDemo {
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            rings: vec![2],
        }
    }
}

impl Demo for HexRingDemo {
    fn draw(&self, graphics: &dyn DemoGraphics) {
        for radius in &self.rings {
            for hex in self.position.ring_iter(*radius) {
                graphics.draw_hex(hex, 1.0, WHITE);
            }
        }
    }
}
