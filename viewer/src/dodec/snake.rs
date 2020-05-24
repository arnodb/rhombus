use crate::{assets::Color, snake::Snake, world::RhombusViewerWorld};
use amethyst::{
    core::{math::Vector3, timing::Time, transform::Transform},
    ecs::prelude::*,
    input::is_key_down,
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::dodec::coordinates::quadric::{QuadricVector, SphereIter};
use std::{collections::VecDeque, sync::Arc};

pub struct DodecSnakeDemo {
    position: QuadricVector,
    snakes: Vec<Snake<Entity, SphereIter>>,
    remaining_millis: u64,
}

impl DodecSnakeDemo {
    pub fn new() -> Self {
        Self {
            position: QuadricVector::default(),
            snakes: Vec::new(),
            remaining_millis: 0,
        }
    }

    fn new_snake(
        position: QuadricVector,
        radius: usize,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) -> Snake<Entity, SphereIter> {
        let mut state = VecDeque::new();
        let mut iter = Self::snake_center(position).sphere_iter(radius);
        state.push_back(Self::push_dodec(
            iter.next().expect("first"),
            data,
            world,
            Color::Red,
        ));
        Snake {
            radius,
            state,
            iter,
        }
    }

    fn snake_center(position: QuadricVector) -> QuadricVector {
        position
    }

    fn snake_tail_size(radius: usize) -> usize {
        12 * radius
    }

    fn push_dodec(
        dodec: QuadricVector,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
        color: Color,
    ) -> Entity {
        let pos = dodec.into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 0.8));
        world.transform_quadric(pos, &mut transform);
        let material = world.assets.color_data[&color].light.clone();
        data.world
            .create_entity()
            .with(world.assets.dodec_handle.clone())
            .with(material)
            .with(transform)
            .build()
    }
}

impl SimpleState for DodecSnakeDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.snakes = vec![Self::new_snake(self.position, 2, &mut data, &world)];
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
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        let delta_millis = {
            let duration = data.world.read_resource::<Time>().delta_time();
            duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
        } + self.remaining_millis;
        let num = delta_millis / 100;
        self.remaining_millis = delta_millis % 100;
        for snake in &mut self.snakes {
            for _ in 0..num {
                if let Some(dodec) = snake.iter.next() {
                    snake
                        .state
                        .push_back(Self::push_dodec(dodec, data, &world, Color::Red));
                } else {
                    snake.iter = Self::snake_center(self.position).sphere_iter(snake.radius);
                    snake.state.push_back(Self::push_dodec(
                        snake.iter.next().expect("first"),
                        data,
                        &world,
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
