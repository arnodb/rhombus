use glutin_window::GlutinWindow;
use piston_window::*;
use rhombus_hex::coordinates::cubic::{CubicVector, RingIter};
use std::time::Instant;

mod gl;
mod glu;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const HEX_RADIUS: f32 = 1.0;
const HEX_INTERIOR_RADIUS: f32 = HEX_RADIUS - HEX_RADIUS / 10.0;

struct HexApp {
    position: CubicVector,
    full_rings: Vec<usize>,
    moving_rings: Vec<(usize, CubicVector, RingIter)>,
}

impl HexApp {
    fn new(position: CubicVector) -> Self {
        let new_ring = |radius: usize| -> (usize, CubicVector, RingIter) {
            let mut iter = position.ring_iter(radius);
            let next = iter.next();
            if iter.peek().is_some() {
                (radius, next.expect("next"), iter)
            } else {
                (radius, next.expect("next"), position.ring_iter(radius))
            }
        };
        Self {
            position,
            full_rings: vec![1, 3],
            moving_rings: vec![new_ring(2), new_ring(4)],
        }
    }

    fn advance(&mut self, num: u64) {
        let position = self.position;
        let adv = |radius: usize, iter: &mut RingIter| -> CubicVector {
            let next = iter.next().expect("next");
            if iter.peek().is_none() {
                *iter = position.ring_iter(radius);
            }
            next
        };
        for ring in &mut self.moving_rings {
            for _ in 0..num {
                ring.1 = adv(ring.0, &mut ring.2);
            }
        }
    }

    fn draw_axes(&self) {
        let length = 5.0;
        unsafe {
            gl::Begin(gl::GL_LINES);
            gl::Color3f(1.0, 0.0, 0.0);
            gl::Vertex3f(-length, 0.0, 0.0);
            gl::Color3f(0.5, 0.0, 0.0);
            gl::Vertex3f(length, 0.0, 0.0);

            gl::Color3f(0.0, 1.0, 0.0);
            gl::Vertex3f(0.0, -length, 0.0);
            gl::Color3f(0.0, 0.5, 0.0);
            gl::Vertex3f(0.0, length, 0.0);

            gl::Color3f(0.0, 0.0, 1.0);
            gl::Vertex3f(0.0, 0.0, -length);
            gl::Color3f(0.0, 0.0, 0.5);
            gl::Vertex3f(0.0, 0.0, length);
            gl::End();
        }
    }

    fn draw(&self) {
        let position = self.position;
        //Self::draw_hex(position);
        for radius in &self.full_rings {
            for hex in position.ring_iter(*radius) {
                Self::draw_hex(hex);
            }
        }
        for ring in &self.moving_rings {
            Self::draw_hex(ring.1);
        }
    }

    fn draw_hex(position: CubicVector) {
        let col = position.x() + (position.z() - (position.z() & 1)) / 2;
        let row = position.z();

        let big = HEX_INTERIOR_RADIUS;
        let small = HEX_INTERIOR_RADIUS * f32::sqrt(3.0) / 2.0;

        unsafe {
            gl::PushMatrix();

            gl::Translatef(
                -HEX_RADIUS * f32::sqrt(3.0) * ((col as f32) + (position.z() & 1) as f32 / 2.0),
                HEX_RADIUS * row as f32 * 1.5,
                0.0,
            );

            gl::Begin(gl::GL_LINE_STRIP);
            gl::Color3f(1.0, 1.0, 1.0);
            gl::Vertex3f(0.0, big, 0.0);
            gl::Vertex3f(small, big / 2.0, 0.0);
            gl::Vertex3f(small, -big / 2.0, 0.0);
            gl::Vertex3f(0.0, -big, 0.0);
            gl::Vertex3f(-small, -big / 2.0, 0.0);
            gl::Vertex3f(-small, big / 2.0, 0.0);
            gl::Vertex3f(0.0, big, 0.0);
            gl::End();

            gl::PopMatrix();
        }
    }
}

fn resize(width: f64, height: f64) {
    let min = 2.0;
    let (near_width, near_height) = if width > height {
        (min * width / height, min)
    } else {
        (min, min * height / width)
    };
    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::MatrixMode(gl::GL_PROJECTION);
        gl::LoadIdentity();
        gl::Frustum(
            -near_width,
            near_width,
            -near_height,
            near_height,
            20.,
            200.,
        );
        gl::MatrixMode(gl::GL_MODELVIEW);
        gl::LoadIdentity();
    }
}

fn main() {
    let mut app = HexApp::new(CubicVector::new(0, 0, 0));

    let mut window: GlutinWindow = WindowSettings::new("Rhombus Viewer", [WIDTH, HEIGHT])
        .graphics_api(OpenGL::V2_1)
        .exit_on_esc(true)
        .build()
        .unwrap();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::LineWidth(1.);
    }

    let Size { width, height } = window.draw_size();
    resize(width, height);

    let mut events = Events::new(EventSettings::new().swap_buffers(true));

    let start_time = Instant::now();
    let mut prev_millis = 0;

    while let Some(event) = events.next(&mut window) {
        let now_millis = {
            let duration = Instant::now().duration_since(start_time);
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        };
        let adv = (now_millis - prev_millis + (prev_millis % 100)) / 100;
        prev_millis = now_millis;

        if adv > 0 {
            app.advance(adv);
        }

        if let Some(_args) = event.render_args() {
            unsafe {
                gl::Clear(gl::GL_COLOR_BUFFER_BIT);
                gl::LoadIdentity();
            }
            glu::look_at(-10.0, -10.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
            app.draw_axes();
            app.draw();
        }

        if let Some(args) = event.resize_args() {
            let width = args.draw_size[0] as f64;
            let height = args.draw_size[1] as f64;
            resize(width, height);
        }
    }
}
