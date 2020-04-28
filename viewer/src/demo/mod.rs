use crate::color::Color;
use rhombus_core::dodec::coordinates::quadric::QuadricVector;
use rhombus_core::hex::coordinates::cubic::CubicVector;

pub mod dodec;
pub mod hex;

pub trait Demo {
    fn advance(&mut self, _millis: u64) {}

    fn draw(&self, graphics: &dyn DemoGraphics);
}

pub trait DemoGraphics {
    fn draw_hex(&self, position: CubicVector, radius_ratio: f32, color: Color);

    fn draw_dodec(&self, position: QuadricVector, radius_ratio: f32, color: Color);
}

struct Snake<V, I> {
    radius: usize,
    state: Vec<V>,
    iter: I,
}
