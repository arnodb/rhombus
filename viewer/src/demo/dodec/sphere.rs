use crate::color::GREY;
use crate::demo::{Demo, DemoGraphics};
use rhombus_core::dodec::coordinates::quadric::QuadricVector;

pub struct DodecSphereDemo {
    position: QuadricVector,
    spheres: Vec<usize>,
}

impl DodecSphereDemo {
    pub fn new(position: QuadricVector) -> Self {
        Self {
            position,
            spheres: vec![1],
        }
    }
}

impl Demo for DodecSphereDemo {
    fn draw(&self, graphics: &dyn DemoGraphics) {
        for radius in &self.spheres {
            for dodec in self.position.sphere_iter(*radius) {
                graphics.draw_dodec(dodec, 1.0, GREY);
            }
        }
    }
}
