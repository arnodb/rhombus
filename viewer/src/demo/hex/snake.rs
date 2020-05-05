use crate::{
    demo::{Color, RhombusViewerAssets, Snake},
    system::cubic::CubicPositionSystem,
};
use amethyst::{
    core::{math::Vector3, timing::Time, transform::Transform},
    ecs::prelude::*,
    input::is_key_down,
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::cubic::{CubicVector, RingIter};
use std::{collections::VecDeque, ops::Deref, sync::Arc};

pub struct HexSnakeDemo {
    position: CubicVector,
    snakes: Vec<Snake<Entity, RingIter>>,
    remaining_millis: u64,
}

impl HexSnakeDemo {
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            snakes: Vec::new(),
            remaining_millis: 0,
        }
    }

    fn new_snake(
        position: CubicVector,
        radius: usize,
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
    ) -> Snake<Entity, RingIter> {
        let mut state = VecDeque::new();
        let mut iter = Self::snake_center(position).ring_iter(radius);
        state.push_back(Self::push_hex(
            iter.next().expect("first"),
            data,
            &assets,
            Color::Red,
        ));
        Snake {
            radius,
            state,
            iter,
        }
    }

    fn snake_center(position: CubicVector) -> CubicVector {
        position
    }

    fn snake_tail_size(radius: usize) -> usize {
        3 * radius
    }

    fn push_hex(
        hex: CubicVector,
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        color: Color,
    ) -> Entity {
        let pos = hex.into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 0.8));
        CubicPositionSystem::transform(pos, &mut transform);
        let color_data = assets.color_data[&color].clone();
        data.world
            .create_entity()
            .with(assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .with(pos)
            .build()
    }
}

impl SimpleState for HexSnakeDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();

        self.snakes = vec![
            Self::new_snake(self.position, 1, &mut data, &assets),
            Self::new_snake(self.position, 3, &mut data, &assets),
        ];
        self.remaining_millis = 0;
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        for snake in &mut self.snakes {
            while let Some(entity) = snake.state.pop_front() {
                data.world.delete_entity(entity).expect("delete entity");
            }
        }
        self.snakes.clear();
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Pop
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();

        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 100;
        self.remaining_millis = delta_millis % 100;
        for snake in &mut self.snakes {
            for _ in 0..num {
                if let Some(hex) = snake.iter.next() {
                    snake
                        .state
                        .push_back(Self::push_hex(hex, data, &assets, Color::Red));
                } else {
                    snake.iter = Self::snake_center(self.position).ring_iter(snake.radius);
                    snake.state.push_back(Self::push_hex(
                        snake.iter.next().expect("first"),
                        data,
                        &assets,
                        Color::Red,
                    ));
                }
                while snake.state.len() > Self::snake_tail_size(snake.radius) {
                    if let Some(entity) = snake.state.pop_front() {
                        data.world.delete_entity(entity).expect("delete entity");
                    }
                }
            }
        }
        Trans::None
    }
}
