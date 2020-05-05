use crate::{
    demo::{Color, RhombusViewerAssets},
    system::cubic::CubicPositionSystem,
};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    input::{get_key, ElementState},
    prelude::*,
    winit::VirtualKeyCode,
};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::{collections::BTreeMap, ops::Deref, sync::Arc};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum HexState {
    Open,
    Wall,
}

struct HexData {
    state: HexState,
    entity: Entity,
}

pub struct HexFlatBuilderDemo {
    position: CubicVector,
    world: BTreeMap<(isize, isize), HexData>,
    direction: usize,
}

impl HexFlatBuilderDemo {
    pub fn new(position: CubicVector) -> Self {
        Self {
            position,
            world: BTreeMap::new(),
            direction: 0,
        }
    }

    fn to_world_key(position: CubicVector) -> (isize, isize) {
        let axial = AxialVector::from(position);
        (axial.q(), axial.r())
    }

    fn create_ground(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
    ) -> Entity {
        let pos = position.into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 0.8));
        CubicPositionSystem::transform(pos, &mut transform);
        let color_data = assets.color_data[&Color::White].clone();
        data.world
            .create_entity()
            .with(assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .with(pos)
            .build()
    }

    fn create_wall(
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
    ) -> Entity {
        let pos = position.into();
        let mut transform = Transform::default();
        transform.set_scale(Vector3::new(0.8, 0.8, 3.0));
        CubicPositionSystem::transform(pos, &mut transform);
        let color_data = assets.color_data[&Color::Red].clone();
        data.world
            .create_entity()
            .with(assets.hex_handle.clone())
            .with(color_data.texture)
            .with(color_data.material)
            .with(transform)
            .with(pos)
            .build()
    }

    fn wallize(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        assets: &Arc<RhombusViewerAssets>,
        position: CubicVector,
    ) {
        self.world
            .entry(Self::to_world_key(position))
            .or_insert_with(|| HexData {
                state: HexState::Wall,
                entity: Self::create_wall(data, assets, position),
            });
    }
}

impl SimpleState for HexFlatBuilderDemo {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let assets = data
            .world
            .read_resource::<Arc<RhombusViewerAssets>>()
            .deref()
            .clone();
        self.world.insert(
            Self::to_world_key(self.position),
            HexData {
                state: HexState::Open,
                entity: Self::create_ground(&mut data, &assets, self.position),
            },
        );
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        for (_, hex) in self.world.iter() {
            data.world.delete_entity(hex.entity).expect("delete entity");
        }
        self.world.clear();
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            let mut trans = Trans::None;
            let assets = data
                .world
                .read_resource::<Arc<RhombusViewerAssets>>()
                .deref()
                .clone();
            match get_key(&event) {
                Some((VirtualKeyCode::Escape, ElementState::Pressed)) => {
                    trans = Trans::Pop;
                }
                Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                    self.direction = (self.direction + 1) % 6;
                }
                Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                    self.direction = (self.direction + 5) % 6;
                }
                Some((VirtualKeyCode::Up, ElementState::Pressed)) => {
                    let next = self.position.neighbor(self.direction);
                    let mut new = false;
                    let next_state =
                        self.world
                            .entry(Self::to_world_key(next))
                            .or_insert_with(|| {
                                new = true;
                                HexData {
                                    state: HexState::Open,
                                    entity: Self::create_ground(&mut data, &assets, next),
                                }
                            });
                    match next_state.state {
                        HexState::Open => {
                            if new {
                                // Left
                                self.wallize(
                                    &mut data,
                                    &assets,
                                    self.position.neighbor((self.direction + 1) % 6),
                                );
                                // Right
                                self.wallize(
                                    &mut data,
                                    &assets,
                                    self.position.neighbor((self.direction + 5) % 6),
                                );
                                // Ahead
                                let ahead_left = next.neighbor((self.direction + 1) % 6);
                                let ahead = next.neighbor(self.direction);
                                let ahead_right = next.neighbor((self.direction + 5) % 6);
                                match (
                                    self.world
                                        .get(&Self::to_world_key(ahead_left))
                                        .map(|h| h.state),
                                    self.world.get(&Self::to_world_key(ahead)).map(|h| h.state),
                                    self.world
                                        .get(&Self::to_world_key(ahead_right))
                                        .map(|h| h.state),
                                ) {
                                    (Some(HexState::Open), _, _) | (_, _, Some(HexState::Open)) => {
                                        self.wallize(&mut data, &assets, ahead);
                                    }
                                    (_, Some(HexState::Open), _) => {
                                        self.wallize(&mut data, &assets, ahead_left);
                                        self.wallize(&mut data, &assets, ahead_right);
                                    }
                                    _ => {}
                                }
                            }
                            self.position = next;
                        }
                        HexState::Wall => {}
                    }
                }
                _ => {}
            }
            trans
        } else {
            Trans::None
        }
    }
}
