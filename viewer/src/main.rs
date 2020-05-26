#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate derive_new;

pub mod assets;
pub mod dodec;
pub mod hex;
pub mod input;
pub mod snake;
pub mod systems;
pub mod world;

use crate::{
    assets::{Color, ColorData, RhombusViewerAssets},
    dodec::{directions::DodecDirectionsDemo, snake::DodecSnakeDemo, sphere::DodecSphereDemo},
    hex::{
        bumpy_builder::HexBumpyBuilderDemo, cellular::builder::HexCellularBuilder,
        directions::HexDirectionsDemo, flat_builder::HexFlatBuilderDemo, ring::HexRingDemo,
        snake::HexSnakeDemo,
    },
    systems::{
        camera_distance::CameraDistanceSystemDesc,
        follow_me::{FollowMeSystem, FollowMeTag, FollowMyRotationSystem, FollowMyRotationTag},
    },
    world::RhombusViewerWorld,
};
use amethyst::{
    assets::{AssetLoaderSystemData, ProgressCounter},
    controls::{ArcBallControlBundle, ArcBallControlTag, FlyControlTag},
    core::{
        math::Vector3,
        timing::Time,
        transform::{Parent, Transform, TransformBundle},
    },
    ecs::prelude::*,
    input::{is_key_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        camera::{Camera, Perspective, Projection},
        debug_drawing::DebugLinesComponent,
        formats::mesh::ObjFormat,
        light::{Light, PointLight},
        palette::{Srgb, Srgba},
        plugins::{RenderDebugLines, RenderToWindow},
        rendy::texture::palette::load_from_srgba,
        types::{DefaultBackend, Mesh, Texture},
        Material, MaterialDefaults, RenderShaded3D, RenderingBundle,
    },
    utils::application_root_dir,
    winit::VirtualKeyCode,
    Application, Error, GameDataBuilder, LoggerConfig, SimpleState, StateEvent,
};
use rhombus_core::hex::coordinates::axial::AxialVector;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf, sync::Arc};
use structopt::StructOpt;

const LOGGER_CONFIG: &str = "config/logger.yaml";

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const MAX_ROTATED_DEMOS: usize = 6;

const DEMO_HEX_DIRECTIONS: usize = 0;
const DEMO_HEX_RING: usize = 1;
const DEMO_HEX_SNAKE: usize = 2;
const DEMO_DODEC_DIRECTIONS: usize = 3;
const DEMO_DODEC_SPHERE: usize = 4;
const DEMO_DODEC_SNAKE: usize = 5;

const HEX_FLAT_BUILDER: usize = 100;
const HEX_BUMPY_BUILDER: usize = 101;
const HEX_CELLULAR_BUILDER: usize = 102;

enum RhombusViewerAnimation {
    Fixed { demo_num: usize },
    Rotating { demo_num: usize },
}

struct RhombusViewer {
    animation: RhombusViewerAnimation,
    last_resume_time: f64,
    progress_counter: ProgressCounter,
    origin: Option<Entity>,
    follower: Option<Entity>,
    draw_axes: bool,
}

impl RhombusViewer {
    fn new(demo_num: Option<usize>, draw_axes: bool) -> Self {
        let first_demo_num = demo_num.unwrap_or(0);
        Self {
            animation: if demo_num.is_some() {
                RhombusViewerAnimation::Fixed {
                    demo_num: first_demo_num,
                }
            } else {
                RhombusViewerAnimation::Rotating {
                    demo_num: first_demo_num,
                }
            },
            last_resume_time: 0.0,
            progress_counter: ProgressCounter::default(),
            origin: None,
            follower: None,
            draw_axes,
        }
    }

    fn transition(demo_num: usize) -> SimpleTrans {
        let new_state: Box<dyn State<GameData<'static, 'static>, StateEvent>> = match demo_num {
            // Simple demos
            DEMO_HEX_DIRECTIONS => Box::new(HexDirectionsDemo::new()),
            DEMO_HEX_RING => Box::new(HexRingDemo::new()),
            DEMO_HEX_SNAKE => Box::new(HexSnakeDemo::new()),
            DEMO_DODEC_DIRECTIONS => Box::new(DodecDirectionsDemo::new()),
            DEMO_DODEC_SPHERE => Box::new(DodecSphereDemo::new()),
            DEMO_DODEC_SNAKE => Box::new(DodecSnakeDemo::new()),
            // Flat hex builders
            HEX_FLAT_BUILDER => Box::new(HexFlatBuilderDemo::new()),
            // Bumpy hex builders
            HEX_BUMPY_BUILDER => Box::new(HexBumpyBuilderDemo::new()),
            // Cellular hex builders
            HEX_CELLULAR_BUILDER => Box::new(HexCellularBuilder::new_edge()),
            _ => unimplemented!(),
        };
        Trans::Push(new_state)
    }
}

impl SimpleState for RhombusViewer {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.last_resume_time = data
            .world
            .read_resource::<Time>()
            .absolute_real_time_seconds();
        if self.draw_axes {
            let mut debug_lines_component = DebugLinesComponent::with_capacity(100);
            debug_lines_component.add_direction(
                [-1.0, 0.0, 0.0].into(),
                [5.0, 0.0, 0.0].into(),
                Srgba::new(0.5, 0.0, 0.0, 1.0),
            );
            debug_lines_component.add_direction(
                [0.0, -1.0, 0.0].into(),
                [0.0, 5.0, 0.0].into(),
                Srgba::new(0.0, 0.5, 0.0, 1.0),
            );
            debug_lines_component.add_direction(
                [0.0, 0.0, -1.0].into(),
                [0.0, 0.0, 5.0].into(),
                Srgba::new(0.0, 0.0, 0.5, 1.0),
            );
            data.world
                .create_entity()
                .with(debug_lines_component)
                .build();
        }
        let assets = {
            let hex_handle = data.world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load("mesh/hex.obj", ObjFormat, &mut self.progress_counter)
            });
            let dodec_handle = data.world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load("mesh/dodec.obj", ObjFormat, &mut self.progress_counter)
            });
            let pointer_handle = data.world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load("mesh/pointer.obj", ObjFormat, &mut self.progress_counter)
            });
            let mat_defaults = data.world.read_resource::<MaterialDefaults>().0.clone();
            let color_data = [
                (Color::Black, (0.0, 0.0, 0.0, 1.0), (0.0, 0.0, 0.0, 1.0)),
                (Color::Red, (1.0, 0.0, 0.0, 1.0), (0.5, 0.0, 0.0, 1.0)),
                (Color::Green, (0.0, 1.0, 0.0, 1.0), (0.0, 0.5, 0.0, 1.0)),
                (Color::Blue, (0.0, 0.0, 1.0, 1.0), (0.0, 0.0, 0.5, 1.0)),
                (Color::Yellow, (1.0, 1.0, 0.0, 1.0), (0.5, 0.5, 0.0, 1.0)),
                (Color::Magenta, (1.0, 0.0, 1.0, 1.0), (0.5, 0.0, 0.5, 1.0)),
                (Color::Cyan, (0.0, 1.0, 1.0, 1.0), (0.0, 0.5, 0.5, 1.0)),
                (Color::White, (1.0, 1.0, 1.0, 1.0), (0.5, 0.5, 0.5, 1.0)),
            ]
            .iter()
            .map(|(color, light_rgba, dark_rgba)| {
                let mut load_color = |rgba: &(f32, f32, f32, f32)| {
                    let texture = data
                        .world
                        .exec(|loader: AssetLoaderSystemData<'_, Texture>| {
                            loader.load_from_data(
                                load_from_srgba(Srgba::new(rgba.0, rgba.1, rgba.2, rgba.3)).into(),
                                &mut self.progress_counter,
                            )
                        });
                    let material =
                        data.world
                            .exec(|loader: AssetLoaderSystemData<'_, Material>| {
                                loader.load_from_data(
                                    Material {
                                        albedo: texture.clone(),
                                        ..mat_defaults.clone()
                                    },
                                    &mut self.progress_counter,
                                )
                            });
                    material
                };
                let light = load_color(light_rgba);
                let dark = load_color(dark_rgba);
                (*color, ColorData { light, dark })
            })
            .collect::<HashMap<_, _>>();

            RhombusViewerAssets {
                hex_handle,
                dodec_handle,
                pointer_handle,
                color_data,
            }
        };

        for r in 0..4 {
            for pos in AxialVector::default()
                .ring_iter(r * 24)
                .step_by(if r != 0 { 24 } else { 1 })
            {
                let light: Light = PointLight {
                    intensity: 30.0,
                    color: Srgb::new(1.0, 1.0, 1.0),
                    radius: 5.0,
                    smoothness: 4.0,
                }
                .into();

                let mut light_transform = Transform::default();

                let col = pos.q() + (pos.r() - (pos.r() & 1)) / 2;
                let row = pos.r();
                light_transform.set_translation_xyz(
                    f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
                    10.0,
                    -row as f32 * 1.5,
                );

                data.world
                    .create_entity()
                    .with(light)
                    .with(light_transform)
                    .build();
            }
        }

        // Origin with default orientation
        let origin = data
            .world
            .create_entity()
            .with(Transform::default())
            .build();
        self.origin = Some(origin);

        // Origin with camera orientation
        let mut origin_camera_transform = Transform::default();
        origin_camera_transform.append_rotation_y_axis(-std::f32::consts::PI / 2.0);
        origin_camera_transform.append_rotation_x_axis(-std::f32::consts::PI / 10.0);
        let origin_camera = data
            .world
            .create_entity()
            .with(Parent { entity: origin })
            .with(origin_camera_transform)
            .build();

        // Follower with default orientation
        let mut follower_transform = Transform::default();
        //follower_transform.set_scale(Vector3::new(0.2, 0.05, 0.2));
        follower_transform.prepend_rotation_y_axis(std::f32::consts::PI / 2.0);
        let follower = data
            .world
            .create_entity()
            .with(follower_transform)
            //.with(assets.pointer_handle.clone())
            //.with(assets.color_data[&Color::Magenta].light.clone())
            .with(FollowMeTag {
                target: Some((origin, 0.1)),
                rotation_target: None,
            })
            .build();
        self.follower = Some(follower);

        // Follower with camera orientation
        let mut follower_camera_transform = Transform::default();
        follower_camera_transform.append_translation_xyz(-9.0, 15.0, -6.0);
        follower_camera_transform
            .face_towards(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        let follower_camera = data
            .world
            .create_entity()
            .with(Parent { entity: follower })
            .with(follower_camera_transform)
            .with(FlyControlTag)
            .with(FollowMeTag {
                target: None,
                rotation_target: None,
            })
            .build();

        let world = Arc::new(RhombusViewerWorld::new(
            assets,
            origin,
            origin_camera,
            follower,
            follower_camera,
        ));
        data.world.insert(world);

        let mut camera = Camera::standard_3d(WIDTH as f32, HEIGHT as f32);
        camera.set_projection(Projection::Perspective(Perspective::new(
            1.3,
            std::f32::consts::FRAC_PI_4,
            0.1,
            2000.0,
        )));

        data.world
            .create_entity()
            .with(camera)
            .with(Transform::default())
            .with(FollowMyRotationTag {
                targets: [follower_camera, follower],
                lerp_ratio: 1.0,
            })
            .with(ArcBallControlTag {
                target: follower,
                distance: 15.0,
            })
            .build();
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(follower) = self.follower.take() {
            data.world.delete_entity(follower).expect("delete entity");
        }
        if let Some(origin) = self.origin.take() {
            data.world.delete_entity(origin).expect("delete entity");
        }
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.last_resume_time = data
            .world
            .read_resource::<Time>()
            .absolute_real_time_seconds();
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let time = data
            .world
            .read_resource::<Time>()
            .absolute_real_time_seconds();
        if !self.progress_counter.is_complete() {
            return Trans::None;
        }
        if time - self.last_resume_time > 1.0 {
            match &mut self.animation {
                RhombusViewerAnimation::Fixed { demo_num } => Self::transition(*demo_num),
                RhombusViewerAnimation::Rotating { demo_num } => {
                    let trans = Self::transition(*demo_num);
                    let next_demo_num = (*demo_num + 1) % MAX_ROTATED_DEMOS;
                    *demo_num = next_demo_num;
                    trans
                }
            }
        } else {
            Trans::None
        }
    }
}

fn logger_setup(logger_config_path: Option<PathBuf>) -> Result<(), Error> {
    let is_user_specified = logger_config_path.is_some();

    // If the user specified a logger configuration path, use that.
    // Otherwise fallback to a default.
    let logger_config_path = logger_config_path.unwrap_or_else(|| PathBuf::from(LOGGER_CONFIG));
    let logger_config_path = if logger_config_path.is_relative() {
        let app_dir = application_root_dir()?;
        app_dir.join(logger_config_path)
    } else {
        logger_config_path
    };

    let logger_config: LoggerConfig = if logger_config_path.exists() {
        let logger_file = File::open(&logger_config_path)?;
        let mut logger_file_reader = BufReader::new(logger_file);
        let logger_config = serde_yaml::from_reader(&mut logger_file_reader)?;

        Ok(logger_config)
    } else if is_user_specified {
        let message = format!(
            "Failed to read logger configuration file: `{}`.",
            logger_config_path.display()
        );
        eprintln!("{}", message);

        Err(Error::from_string(message))
    } else {
        Ok(LoggerConfig::default())
    }?;

    amethyst::Logger::from_config(logger_config).start();

    Ok(())
}

#[derive(StructOpt, Debug, Clone, Copy)]
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
    #[structopt(name = "hex-bumpy-builder")]
    HexBumpyBuilder = HEX_BUMPY_BUILDER as isize,
    #[structopt(name = "hex-cellular-builder")]
    HexCellularBuilder = HEX_CELLULAR_BUILDER as isize,
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(subcommand)]
    demo: Option<DemoOption>,
}

fn main() -> amethyst::Result<()> {
    let options = Options::from_args();

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    logger_setup(None)?;

    let draw_axes = options
        .demo
        .map(|demo| demo as usize <= MAX_ROTATED_DEMOS)
        .unwrap_or(true);

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(ArcBallControlBundle::<StringBindings>::new())?
        .with(FollowMeSystem, "follow_me_system", &["arc_ball_rotation"])
        .with(
            FollowMyRotationSystem,
            "follow_my_rotation_system",
            &["arc_ball_rotation"],
        )
        .with_system_desc(
            CameraDistanceSystemDesc::default(),
            "camera_distance_system",
            &["input_system"],
        )
        .with_bundle({
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.05, 0.05, 0.05, 1.0]),
                )
                .with_plugin(RenderShaded3D::default())
                .with_plugin(RenderDebugLines::default())
        })?;

    let app = RhombusViewer::new(options.demo.map(|demo| demo as usize), draw_axes);

    let mut game = Application::new(assets_dir, app, game_data)?;

    game.run();

    Ok(())
}
