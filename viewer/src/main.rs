use crate::color::{Color, BLUE, DARK_BLUE, DARK_GREEN, DARK_RED, GREEN, GREY, RED, WHITE};
use crate::demo::dodec::directions::DodecDirectionsDemo;
use crate::demo::dodec::snake::DodecSnakeDemo;
use crate::demo::dodec::sphere::DodecSphereDemo;
use crate::demo::hex::directions::HexDirectionsDemo;
use crate::demo::hex::flat_builder::HexFlatBuilderDemo;
use crate::demo::hex::ring::HexRingDemo;
use crate::demo::hex::snake::HexSnakeDemo;
use crate::demo::{Demo, DemoGraphics};
use glutin_window::GlutinWindow;
use piston_window::*;
use rhombus_core::dodec::coordinates::quadric::QuadricVector;
use rhombus_core::hex::coordinates::cubic::CubicVector;
use std::time::Instant;
use structopt::StructOpt;

mod gl;
mod glu;

mod color;
mod demo;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const HEX_RADIUS: f32 = 1.0;
const HEX_RADIUS_RATIO: f32 = 0.8;

const MAX_ROTATED_DEMOS: usize = 6;

const DEMO_HEX_DIRECTIONS: usize = 0;
const DEMO_HEX_RING: usize = 1;
const DEMO_HEX_SNAKE: usize = 2;
const DEMO_DODEC_DIRECTIONS: usize = 3;
const DEMO_DODEC_SPHERE: usize = 4;
const DEMO_DODEC_SNAKE: usize = 5;

const HEX_FLAT_BUILDER: usize = 100;

enum RhombusViewerDemo {
    HexDirections(HexDirectionsDemo),
    HexRing(HexRingDemo),
    HexSnake(HexSnakeDemo),
    DodecDirections(DodecDirectionsDemo),
    DodecSphere(DodecSphereDemo),
    DodecSnake(DodecSnakeDemo),
    HexFlatBuilder(HexFlatBuilderDemo),
}

impl RhombusViewerDemo {
    fn demo(&self) -> &dyn Demo {
        match self {
            Self::HexDirections(demo) => demo,
            Self::HexRing(demo) => demo,
            Self::HexSnake(demo) => demo,
            Self::DodecDirections(demo) => demo,
            Self::DodecSphere(demo) => demo,
            Self::DodecSnake(demo) => demo,
            Self::HexFlatBuilder(demo) => demo,
        }
    }

    fn demo_mut(&mut self) -> &mut dyn Demo {
        match self {
            Self::HexDirections(demo) => demo,
            Self::HexRing(demo) => demo,
            Self::HexSnake(demo) => demo,
            Self::DodecDirections(demo) => demo,
            Self::DodecSphere(demo) => demo,
            Self::DodecSnake(demo) => demo,
            Self::HexFlatBuilder(demo) => demo,
        }
    }
}

enum RhombusViewerAnimation {
    Fixed { last_millis: u64 },
    Rotating { last_millis: u64, demo_num: usize },
}

struct RhombusViewer {
    position: QuadricVector,
    demo: RhombusViewerDemo,
    animation: RhombusViewerAnimation,
}

impl RhombusViewer {
    fn new(position: QuadricVector, demo_num: Option<usize>) -> Self {
        let first_demo_num = demo_num.unwrap_or(0);
        Self {
            position,
            demo: Self::new_demo(first_demo_num, position),
            animation: if demo_num.is_some() {
                RhombusViewerAnimation::Fixed { last_millis: 0 }
            } else {
                RhombusViewerAnimation::Rotating {
                    last_millis: 0,
                    demo_num: first_demo_num,
                }
            },
        }
    }

    fn new_demo(demo_num: usize, position: QuadricVector) -> RhombusViewerDemo {
        match demo_num {
            // Simple demos
            DEMO_HEX_DIRECTIONS => RhombusViewerDemo::HexDirections(HexDirectionsDemo::new(
                CubicVector::new(position.x(), position.y(), position.z()),
            )),
            DEMO_HEX_RING => RhombusViewerDemo::HexRing(HexRingDemo::new(CubicVector::new(
                position.x(),
                position.y(),
                position.z(),
            ))),
            DEMO_HEX_SNAKE => RhombusViewerDemo::HexSnake(HexSnakeDemo::new(CubicVector::new(
                position.x(),
                position.y(),
                position.z(),
            ))),
            DEMO_DODEC_DIRECTIONS => {
                RhombusViewerDemo::DodecDirections(DodecDirectionsDemo::new(position))
            }
            DEMO_DODEC_SPHERE => RhombusViewerDemo::DodecSphere(DodecSphereDemo::new(position)),
            DEMO_DODEC_SNAKE => RhombusViewerDemo::DodecSnake(DodecSnakeDemo::new(position)),
            // Flat builders
            HEX_FLAT_BUILDER => RhombusViewerDemo::HexFlatBuilder(HexFlatBuilderDemo::new(
                CubicVector::new(position.x(), position.y(), position.z()),
            )),
            _ => unreachable!(),
        }
    }

    fn advance(&mut self, millis: u64) {
        match &mut self.animation {
            RhombusViewerAnimation::Fixed { last_millis, .. } => {
                *last_millis += millis;
                self.demo.demo_mut().advance(millis);
            }
            RhombusViewerAnimation::Rotating {
                last_millis,
                demo_num,
            } => {
                if *last_millis + millis <= 5000 {
                    *last_millis += millis;
                    self.demo.demo_mut().advance(millis);
                } else {
                    let next_demo_num = (*demo_num + 1) % MAX_ROTATED_DEMOS;
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
        self.demo.demo().draw(self);
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

    fn handle_button_args(&mut self, args: &ButtonArgs) {
        self.demo.demo_mut().handle_button_args(args);
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

    fn draw_hex_arrow(&self, from: CubicVector, rotation_z: f32, color: Color) {
        let col = from.x() + (from.z() - (from.z() & 1)) / 2;
        let row = from.z();

        let small = HEX_RADIUS * f32::sqrt(3.0) / 2.0;

        unsafe {
            gl::PushMatrix();

            gl::Translatef(
                HEX_RADIUS * f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
                -HEX_RADIUS * row as f32 * 1.5,
                0.0,
            );
            gl::Rotatef(rotation_z, 0.0, 0.0, 1.0);

            gl::Begin(gl::GL_TRIANGLE_FAN);
            Self::set_color(color);
            gl::Vertex3f(small - 0.1, 0.0, 0.0);
            gl::Vertex3f(small - 0.3, HEX_RADIUS * 0.3, 0.0);
            gl::Vertex3f(small + 0.3, 0.0, 0.0);
            gl::Vertex3f(small - 0.3, -HEX_RADIUS * 0.3, 0.0);
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

#[derive(StructOpt, Debug)]
enum DemoOption {
    #[structopt(name = "hex-directions")]
    HexDirections = DEMO_HEX_DIRECTIONS as isize,
    #[structopt(name = "hex-ring")]
    HexRing = DEMO_HEX_RING as isize,
    #[structopt(name = "hex-snake")]
    HexSnake = DEMO_HEX_SNAKE as isize,
    #[structopt(name = "dodec-directions")]
    DodecDirections = DEMO_DODEC_DIRECTIONS as isize,
    #[structopt(name = "dodec-sphere")]
    DodecSphere = DEMO_DODEC_SPHERE as isize,
    #[structopt(name = "dodec-snake")]
    DodecSnake = DEMO_DODEC_SNAKE as isize,

    #[structopt(name = "hex-flat-builder")]
    HexFlatBuilder = HEX_FLAT_BUILDER as isize,
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(subcommand)]
    demo: Option<DemoOption>,
}

fn main() {
    let options = Options::from_args();
    let mut app = RhombusViewer::new(
        QuadricVector::new(0, 0, 0, 0),
        options.demo.map(|demo| demo as usize),
    );

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

        if let Some(args) = event.button_args() {
            app.handle_button_args(&args);
        }
    }
}
