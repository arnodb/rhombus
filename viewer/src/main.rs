use crate::color::{Color, BLUE, DARK_BLUE, DARK_GREEN, DARK_RED, GREEN, GREY, RED, WHITE};
use crate::demo::dodec::directions::DodecDirectionsDemo;
use crate::demo::dodec::snake::DodecSnakeDemo;
use crate::demo::dodec::sphere::DodecSphereDemo;
use crate::demo::hex::directions::HexDirectionsDemo;
use crate::demo::hex::ring::HexRingDemo;
use crate::demo::hex::snake::HexSnakeDemo;
use crate::demo::{Demo, DemoGraphics};
use glutin_window::GlutinWindow;
use piston_window::*;
use rhombus_core::dodec::coordinates::quadric::QuadricVector;
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::time::Instant;

mod gl;
mod glu;

mod color;
mod demo;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const HEX_RADIUS: f32 = 1.0;
const HEX_RADIUS_RATIO: f32 = 0.8;

const NUM_DEMOS: usize = 6;

enum RhombusViewerDemo {
    HexDirections(HexDirectionsDemo),
    HexRing(HexRingDemo),
    HexSnake(HexSnakeDemo),
    DodecDirections(DodecDirectionsDemo),
    DodecSphere(DodecSphereDemo),
    DodecSnake(DodecSnakeDemo),
}

impl RhombusViewerDemo {
    fn advance(&mut self, millis: u64) {
        match self {
            Self::HexDirections(demo) => demo.advance(millis),
            Self::HexRing(demo) => demo.advance(millis),
            Self::HexSnake(demo) => demo.advance(millis),
            Self::DodecDirections(demo) => demo.advance(millis),
            Self::DodecSphere(demo) => demo.advance(millis),
            Self::DodecSnake(demo) => demo.advance(millis),
        }
    }

    fn draw(&self, graphics: &dyn DemoGraphics) {
        match self {
            Self::HexDirections(demo) => demo.draw(graphics),
            Self::HexRing(demo) => demo.draw(graphics),
            Self::HexSnake(demo) => demo.draw(graphics),
            Self::DodecDirections(demo) => demo.draw(graphics),
            Self::DodecSphere(demo) => demo.draw(graphics),
            Self::DodecSnake(demo) => demo.draw(graphics),
        }
    }
}

enum RhombusViewerAnimation {
    Rotating { last_millis: u64, demo_num: usize },
}

struct RhombusViewer {
    position: QuadricVector,
    demo: RhombusViewerDemo,
    animation: RhombusViewerAnimation,
}

impl RhombusViewer {
    fn new(position: QuadricVector) -> Self {
        Self {
            position,
            demo: Self::new_demo(0, position),
            animation: RhombusViewerAnimation::Rotating {
                last_millis: 0,
                demo_num: 0,
            },
        }
    }

    fn new_demo(num: usize, position: QuadricVector) -> RhombusViewerDemo {
        match num % 6 {
            0 => RhombusViewerDemo::HexDirections(HexDirectionsDemo::new(CubicVector::new(
                position.x(),
                position.y(),
                position.z(),
            ))),
            1 => RhombusViewerDemo::HexRing(HexRingDemo::new(CubicVector::new(
                position.x(),
                position.y(),
                position.z(),
            ))),
            2 => RhombusViewerDemo::HexSnake(HexSnakeDemo::new(CubicVector::new(
                position.x(),
                position.y(),
                position.z(),
            ))),
            3 => RhombusViewerDemo::DodecDirections(DodecDirectionsDemo::new(position)),
            4 => RhombusViewerDemo::DodecSphere(DodecSphereDemo::new(position)),
            5 => RhombusViewerDemo::DodecSnake(DodecSnakeDemo::new(position)),
            _ => unreachable!(),
        }
    }

    fn advance(&mut self, millis: u64) {
        match &mut self.animation {
            RhombusViewerAnimation::Rotating {
                last_millis,
                demo_num,
            } => {
                if *last_millis + millis <= 5000 {
                    *last_millis += millis;
                    self.demo.advance(millis);
                } else {
                    let next_demo_num = *demo_num + 1 % NUM_DEMOS;
                    self.demo = Self::new_demo(next_demo_num, self.position);
                    *last_millis = 0;
                    *demo_num = next_demo_num;
                }
            }
        }
    }

    fn draw(&self) {
        let position = self.position;
        let p2d = CubicVector::new(position.x(), position.y(), position.z());
        self.draw_axes();
        if false {
            self.draw_hex(p2d, 1.0, WHITE);
            self.draw_dodec(position, 1.0, GREY);
        }
        self.demo.draw(self);
    }

    fn set_color(color: Color) {
        unsafe {
            gl::Color3f(color.0, color.1, color.2);
        }
    }

    fn draw_axes(&self) {
        let length = 5.0;
        unsafe {
            gl::Begin(gl::GL_LINES);
            Self::set_color(RED);
            gl::Vertex3f(-length, 0.0, 0.0);
            Self::set_color(DARK_RED);
            gl::Vertex3f(length, 0.0, 0.0);

            Self::set_color(GREEN);
            gl::Vertex3f(0.0, -length, 0.0);
            Self::set_color(DARK_GREEN);
            gl::Vertex3f(0.0, length, 0.0);

            Self::set_color(BLUE);
            gl::Vertex3f(0.0, 0.0, -length);
            Self::set_color(DARK_BLUE);
            gl::Vertex3f(0.0, 0.0, length);
            gl::End();
        }
    }
}

impl DemoGraphics for RhombusViewer {
    fn draw_hex(&self, position: CubicVector, radius_ratio: f32, color: Color) {
        let col = position.x() + (position.z() - (position.z() & 1)) / 2;
        let row = position.z();

        let radius = HEX_RADIUS * HEX_RADIUS_RATIO * radius_ratio;
        let big = radius;
        let small = radius * f32::sqrt(3.0) / 2.0;

        unsafe {
            gl::PushMatrix();

            gl::Translatef(
                HEX_RADIUS * f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
                -HEX_RADIUS * row as f32 * 1.5,
                0.0,
            );

            gl::Begin(gl::GL_LINE_LOOP);
            Self::set_color(color);
            gl::Vertex3f(0.0, big, 0.0);
            gl::Vertex3f(small, big / 2.0, 0.0);
            gl::Vertex3f(small, -big / 2.0, 0.0);
            gl::Vertex3f(0.0, -big, 0.0);
            gl::Vertex3f(-small, -big / 2.0, 0.0);
            gl::Vertex3f(-small, big / 2.0, 0.0);
            gl::End();

            gl::PopMatrix();
        }
    }

    fn draw_dodec(&self, position: QuadricVector, radius_ratio: f32, color: Color) {
        let col = position.x() + (position.z() - (position.z() & 1)) / 2;
        let row = position.z();
        let depth = position.t();

        let radius = HEX_RADIUS * HEX_RADIUS_RATIO * radius_ratio;
        let big = radius;
        let small = radius * f32::sqrt(3.0) / 2.0;
        let small2 = radius / (2.0 * f32::sqrt(2.0));
        // Fun fact: those two values are analytically identical.
        //let big2 = small2 + radius / f32::tan(2.0 * f32::atan(1.0 / f32::sqrt(2.0)));
        let big2 = (small2 + big) / 2.0;

        unsafe {
            gl::PushMatrix();

            gl::Translatef(
                HEX_RADIUS
                    * f32::sqrt(3.0)
                    * ((col as f32) + ((row & 1) as f32 + depth as f32) / 2.0),
                -HEX_RADIUS * 1.5 * row as f32 - depth as f32 / 2.0,
                -HEX_RADIUS * (1.0 + small2) * depth as f32,
            );

            let p_a = (0.0, 0.0, -big);

            let p_b = (small, big / 2.0, -big2);
            let p_c = (-small, big / 2.0, -big2);
            let p_d = (0.0, -big, -big2);

            let p_e = (0.0, big, -small2);
            let p_f = (-small, -big / 2.0, -small2);
            let p_g = (small, -big / 2.0, -small2);

            let p_h = (small, big / 2.0, small2);
            let p_i = (-small, big / 2.0, small2);
            let p_j = (0.0, -big, small2);

            let p_k = (0.0, big, big2);
            let p_l = (-small, -big / 2.0, big2);
            let p_m = (small, -big / 2.0, big2);

            let p_n = (0.0, 0.0, big);

            let lines = vec![
                // -big -big2
                (p_a, p_b),
                (p_a, p_c),
                (p_a, p_d),
                // -big2 -small2
                (p_b, p_e),
                (p_b, p_g),
                (p_c, p_e),
                (p_c, p_f),
                (p_d, p_f),
                (p_d, p_g),
                // -big2 small2
                (p_b, p_h),
                (p_c, p_i),
                (p_d, p_j),
                // -small2 big2
                (p_e, p_k),
                (p_f, p_l),
                (p_g, p_m),
                // small2 big2
                (p_h, p_m),
                (p_h, p_k),
                (p_i, p_k),
                (p_i, p_l),
                (p_j, p_l),
                (p_j, p_m),
                // big2 big
                (p_k, p_n),
                (p_l, p_n),
                (p_m, p_n),
            ];

            gl::Begin(gl::GL_LINES);
            Self::set_color(color);
            for (from, to) in lines {
                gl::Vertex3f(from.0, from.1, from.2);
                gl::Vertex3f(to.0, to.1, to.2);
            }
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
            1000.,
        );
        gl::MatrixMode(gl::GL_MODELVIEW);
        gl::LoadIdentity();
    }
}

fn main() {
    let mut app = RhombusViewer::new(QuadricVector::new(0, 0, 0, 0));

    let mut window: GlutinWindow = WindowSettings::new("Rhombus Viewer", [WIDTH, HEIGHT])
        .graphics_api(OpenGL::V2_1)
        .exit_on_esc(true)
        .build()
        .unwrap();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::Enable(gl::GL_DEPTH_TEST);
        gl::LineWidth(2.);
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
        let adv_millis = now_millis - prev_millis;
        prev_millis = now_millis;

        if adv_millis > 0 {
            app.advance(adv_millis);
        }

        if let Some(_args) = event.render_args() {
            unsafe {
                gl::Clear(gl::GL_COLOR_BUFFER_BIT | gl::GL_DEPTH_BUFFER_BIT);
                gl::LoadIdentity();
            }
            glu::look_at(-20.0, -30.0, 50.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
            app.draw();
        }

        if let Some(args) = event.resize_args() {
            let width = args.draw_size[0] as f64;
            let height = args.draw_size[1] as f64;
            resize(width, height);
        }
    }
}
