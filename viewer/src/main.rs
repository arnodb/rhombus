use glutin_window::GlutinWindow;
use piston_window::*;
use rhombus_core::dodec::coordinates::quadric::{QuadricVector, SphereIter};
use rhombus_core::hex::coordinates::cubic::{CubicVector, RingIter};
use std::time::Instant;

mod gl;
mod glu;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const DARK_RED: (f32, f32, f32) = (0.5, 0.0, 0.0);
const DARK_GREEN: (f32, f32, f32) = (0.0, 0.5, 0.0);
const DARK_BLUE: (f32, f32, f32) = (0.0, 0.0, 0.5);
const DARK_YELLOW: (f32, f32, f32) = (0.5, 0.5, 0.0);
const DARK_MAGENTA: (f32, f32, f32) = (0.5, 0.0, 0.5);
const DARK_CYAN: (f32, f32, f32) = (0.0, 0.5, 0.5);
const GREY: (f32, f32, f32) = (0.5, 0.5, 0.5);

const RED: (f32, f32, f32) = (1.0, 0.0, 0.0);
const GREEN: (f32, f32, f32) = (0.0, 1.0, 0.0);
const BLUE: (f32, f32, f32) = (0.0, 0.0, 1.0);
const YELLOW: (f32, f32, f32) = (1.0, 1.0, 0.0);
const MAGENTA: (f32, f32, f32) = (1.0, 0.0, 1.0);
const CYAN: (f32, f32, f32) = (0.0, 1.0, 1.0);
const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);

const COLORS: [(f32, f32, f32); 6] = [RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN];

const HEX_RADIUS: f32 = 1.0;
const HEX_RADIUS_RATIO: f32 = 0.8;

struct Snake<V, I> {
    radius: usize,
    state: Vec<V>,
    iter: I,
}

struct HexApp {
    position: QuadricVector,
    full_rings: Vec<usize>,
    snake_rings: Vec<Snake<CubicVector, RingIter>>,
    full_spheres: Vec<usize>,
    snake_spheres: Vec<Snake<QuadricVector, SphereIter>>,
}

impl HexApp {
    fn new(position: QuadricVector) -> Self {
        let new_ring = |radius: usize| -> Snake<CubicVector, RingIter> {
            let mut iter = Self::snake_ring_center(position).ring_iter(radius);
            Snake {
                radius,
                state: vec![iter.next().expect("first")],
                iter,
            }
        };
        let new_spheres = |radius: usize| -> Snake<QuadricVector, SphereIter> {
            let mut iter = Self::snake_sphere_center(position).sphere_iter(radius);
            Snake {
                radius,
                state: vec![iter.next().expect("first")],
                iter,
            }
        };
        Self {
            position,
            full_rings: vec![2],
            snake_rings: vec![new_ring(1), new_ring(3)],
            full_spheres: vec![1],
            snake_spheres: vec![new_spheres(2)],
        }
    }

    fn snake_ring_center(position: QuadricVector) -> CubicVector {
        let p2d = CubicVector::new(position.x(), position.y(), position.z());
        p2d + 6 * CubicVector::direction(4) + 3 * CubicVector::direction(3)
    }

    fn snake_ring_tail_size(radius: usize) -> usize {
        3 * radius
    }

    fn snake_sphere_center(position: QuadricVector) -> QuadricVector {
        position + 6 * QuadricVector::direction(1) + 3 * QuadricVector::direction(0)
    }

    fn snake_sphere_tail_size(radius: usize) -> usize {
        12 * radius
    }

    fn advance(&mut self, num: u64) {
        let position = self.position;
        for snake in &mut self.snake_rings {
            for _ in 0..num {
                if let Some(hex) = snake.iter.next() {
                    snake.state.push(hex);
                } else {
                    snake.iter = Self::snake_ring_center(position).ring_iter(snake.radius);
                    let slice = snake.state.as_mut_slice();
                    let len = slice.len().min(Self::snake_ring_tail_size(snake.radius));
                    slice.copy_within(slice.len() - len..slice.len(), 0);
                    snake.state.truncate(len);
                    snake.state.push(snake.iter.next().expect("first"));
                }
            }
        }
        for snake in &mut self.snake_spheres {
            for _ in 0..num {
                if let Some(dodec) = snake.iter.next() {
                    snake.state.push(dodec);
                } else {
                    snake.iter = Self::snake_sphere_center(position).sphere_iter(snake.radius);
                    let slice = snake.state.as_mut_slice();
                    let len = slice.len().min(Self::snake_sphere_tail_size(snake.radius));
                    slice.copy_within(slice.len() - len..slice.len(), 0);
                    snake.state.truncate(len);
                    snake.state.push(snake.iter.next().expect("first"));
                }
            }
        }
    }

    fn set_color(color: (f32, f32, f32)) {
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

    fn draw(&self) {
        let position = self.position;
        let p2d = CubicVector::new(position.x(), position.y(), position.z());
        Self::draw_hex(p2d, HEX_RADIUS, WHITE);
        Self::draw_dodec(position, HEX_RADIUS, GREY);

        if true {
            let center = p2d - 4 * CubicVector::direction(0);

            Self::draw_hex_direction(center, 0, 3, DARK_RED);
            Self::draw_hex_direction(center, 3, 2, RED);

            Self::draw_hex_direction(center, 1, 3, DARK_GREEN);
            Self::draw_hex_direction(center, 4, 2, GREEN);

            Self::draw_hex_direction(center, 2, 3, DARK_BLUE);
            Self::draw_hex_direction(center, 5, 2, BLUE);
        }

        if true {
            let center = position + 4 * QuadricVector::direction(0);

            Self::draw_dodec_direction(center, 0, 3, DARK_RED);
            Self::draw_dodec_direction(center, 6, 2, RED);

            Self::draw_dodec_direction(center, 1, 3, DARK_GREEN);
            Self::draw_dodec_direction(center, 7, 2, GREEN);

            Self::draw_dodec_direction(center, 2, 3, DARK_BLUE);
            Self::draw_dodec_direction(center, 8, 2, BLUE);

            Self::draw_dodec_direction(center, 3, 3, DARK_YELLOW);
            Self::draw_dodec_direction(center, 9, 2, YELLOW);

            Self::draw_dodec_direction(center, 4, 3, DARK_MAGENTA);
            Self::draw_dodec_direction(center, 10, 2, MAGENTA);

            Self::draw_dodec_direction(center, 5, 3, DARK_CYAN);
            Self::draw_dodec_direction(center, 11, 2, CYAN);
        }

        for radius in &self.full_rings {
            for hex in p2d.ring_iter(*radius) {
                Self::draw_hex(hex, HEX_RADIUS, WHITE);
            }
        }
        for snake in &self.snake_rings {
            for (i, hex) in snake
                .state
                .iter()
                .rev()
                .take(Self::snake_ring_tail_size(snake.radius))
                .enumerate()
            {
                Self::draw_hex(*hex, HEX_RADIUS * 0.8, COLORS[i % 6]);
            }
        }
        for radius in &self.full_spheres {
            for dodec in (position + 6 * QuadricVector::direction(10)).sphere_iter(*radius) {
                Self::draw_dodec(dodec, HEX_RADIUS, GREY);
            }
        }
        for snake in &self.snake_spheres {
            for (i, dodec) in snake
                .state
                .iter()
                .rev()
                .take(Self::snake_sphere_tail_size(snake.radius))
                .enumerate()
            {
                Self::draw_dodec(*dodec, HEX_RADIUS * 0.8, COLORS[i % 6]);
            }
        }
    }

    fn draw_hex_direction(
        mut origin: CubicVector,
        direction: usize,
        length: usize,
        color: (f32, f32, f32),
    ) {
        for _ in 0..length {
            origin = origin.neighbor(direction);
            Self::draw_hex(origin, HEX_RADIUS * 0.3, color);
        }
    }

    fn draw_hex(position: CubicVector, radius: f32, color: (f32, f32, f32)) {
        let col = position.x() + (position.z() - (position.z() & 1)) / 2;
        let row = position.z();

        let big = radius * HEX_RADIUS_RATIO;
        let small = radius * HEX_RADIUS_RATIO * f32::sqrt(3.0) / 2.0;

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

    fn draw_dodec_direction(
        mut origin: QuadricVector,
        direction: usize,
        length: usize,
        color: (f32, f32, f32),
    ) {
        for _ in 0..length {
            origin = origin.neighbor(direction);
            Self::draw_dodec(origin, HEX_RADIUS * 0.3, color);
        }
    }

    fn draw_dodec(position: QuadricVector, radius: f32, color: (f32, f32, f32)) {
        let col = position.x() + (position.z() - (position.z() & 1)) / 2;
        let row = position.z();
        let depth = position.t();

        let big = radius * HEX_RADIUS_RATIO;
        let small = radius * HEX_RADIUS_RATIO * f32::sqrt(3.0) / 2.0;
        let small2 = radius * HEX_RADIUS_RATIO / (2.0 * f32::sqrt(2.0));
        // Fun fact: those two values are analytically identical.
        //let big2 = small2 + radius * HEX_RADIUS_RATIO / f32::tan(2.0 * f32::atan(1.0 / f32::sqrt(2.0)));
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
    let mut app = HexApp::new(QuadricVector::new(0, 0, 0, 0));

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
        let adv = (now_millis - prev_millis + (prev_millis % 100)) / 100;
        prev_millis = now_millis;

        if adv > 0 {
            app.advance(adv);
        }

        if let Some(_args) = event.render_args() {
            unsafe {
                gl::Clear(gl::GL_COLOR_BUFFER_BIT | gl::GL_DEPTH_BUFFER_BIT);
                gl::LoadIdentity();
            }
            glu::look_at(-40.0, -60.0, 100.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
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
