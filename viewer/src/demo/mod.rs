use crate::color::Color;
use piston_window::ButtonArgs;
use rhombus_core::dodec::coordinates::quadric::QuadricVector;
use rhombus_core::hex::coordinates::cubic::CubicVector;

pub mod dodec;
pub mod hex;

pub trait Demo {
    fn advance(&mut self, _millis: u64) {}

    fn draw(&self, graphics: &dyn DemoGraphics);

    fn handle_button_args(&mut self, _args: &ButtonArgs) {}
}

pub trait DemoGraphics {
    fn draw_hex(&self, position: CubicVector, radius_ratio: f32, color: Color) {
        self.draw_hex_translate(position, (0.0, 0.0, 0.0), radius_ratio, color)
    }

    fn draw_hex_translate(
        &self,
        position: CubicVector,
        translation: (f32, f32, f32),
        radius_ratio: f32,
        color: Color,
    );

    fn draw_hex_arrow(
        &self,
        from: CubicVector,
        rotation_z: f32,
        translation: (f32, f32, f32),
        rotation: (f32, f32, f32, f32),
        color: Color,
    );

    fn draw_dodec(&self, position: QuadricVector, radius_ratio: f32, color: Color);
}

struct Snake<V, I> {
    radius: usize,
    state: Vec<V>,
    iter: I,
}
