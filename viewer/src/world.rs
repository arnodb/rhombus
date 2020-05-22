use crate::{assets::RhombusViewerAssets, systems::follow_me::FollowMeTag};
use amethyst::{controls::ArcBallControlTag, core::Transform, ecs::prelude::*, prelude::*};
use rhombus_core::{
    dodec::coordinates::quadric::QuadricVector, hex::coordinates::axial::AxialVector,
};
use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

#[derive(Debug, new)]
pub struct RhombusViewerWorld {
    pub assets: RhombusViewerAssets,
    pub origin: Entity,
    pub origin_camera: Entity,
    pub follower: Entity,
    pub follower_camera: Entity,

    #[new(value = "Arc::new(Mutex::new(None))")]
    follow_mode: Arc<Mutex<Option<(bool, FollowSettings)>>>,
}

#[derive(Debug)]
struct FollowSettings {
    target: Entity,
    rotation_target: Option<Entity>,
}

impl RhombusViewerWorld {
    pub fn transform_axial(&self, position: AxialPosition, transform: &mut Transform) {
        let col = position.pos().q() + (position.pos().r() - (position.pos().r() & 1)) / 2;
        let row = position.pos().r();
        let altitude = position.alt();
        transform.set_translation_xyz(
            f32::sqrt(3.0) * ((col as f32) + (row & 1) as f32 / 2.0),
            altitude,
            -row as f32 * 1.5,
        );
    }

    pub fn transform_quadric(&self, position: QuadricPosition, transform: &mut Transform) {
        let col = position.0.x() + (position.0.z() - (position.0.z() & 1)) / 2;
        let row = position.0.z();
        let depth = position.0.t();
        let small2 = 1.0 / (2.0 * f32::sqrt(2.0));
        transform.set_translation_xyz(
            f32::sqrt(3.0) * ((col as f32) + ((row & 1) as f32 + depth as f32) / 2.0),
            -(1.0 + small2) * depth as f32,
            -1.5 * row as f32 - depth as f32 / 2.0,
        );
    }

    pub fn follow(
        &self,
        data: &StateData<'_, GameData<'_, '_>>,
        target: Entity,
        rotation_target: Option<Entity>,
    ) {
        let mut lock = self.follow_mode.lock().unwrap();
        let mode = lock.deref_mut();
        *mode = Some((
            false,
            FollowSettings {
                target,
                rotation_target,
            },
        ));
        self.follow_internal(data, mode.as_mut().unwrap());
    }

    pub fn follow_origin(&self, data: &StateData<'_, GameData<'_, '_>>) {
        let mut lock = self.follow_mode.lock().unwrap();
        let mode = lock.deref_mut();
        *mode = None;
        self.follow_internal(
            data,
            &mut (
                true,
                FollowSettings {
                    target: self.origin,
                    rotation_target: None,
                },
            ),
        );
    }

    pub fn toggle_follow(&self, data: &StateData<'_, GameData<'_, '_>>) {
        let mut lock = self.follow_mode.lock().unwrap();
        let mode = lock.deref_mut();
        if let Some(mode) = mode {
            mode.0 = !mode.0;
            if mode.0 {
                self.follow_internal(
                    data,
                    &mut (
                        true,
                        FollowSettings {
                            target: self.origin,
                            rotation_target: None,
                        },
                    ),
                );
            } else {
                self.follow_internal(data, mode);
            }
        }
    }

    fn follow_internal(
        &self,
        data: &StateData<'_, GameData<'_, '_>>,
        mode: &mut (bool, FollowSettings),
    ) {
        let mut follow_me_storage = data.world.write_storage::<FollowMeTag>();
        if let Some(tag) = follow_me_storage.get_mut(self.follower) {
            tag.target = Some((mode.1.target, 0.1));
            tag.rotation_target = mode.1.rotation_target.map(|t| (t, 0.1));
        }
        if mode.1.rotation_target.is_some() {
            let mut transform_storage = data.world.write_storage::<Transform>();
            let rotation = transform_storage
                .get(self.origin_camera)
                .map(Transform::rotation)
                .cloned();
            if let Some(rotation) = rotation {
                if let Some(transform) = transform_storage.get_mut(self.follower_camera) {
                    *transform.rotation_mut() = rotation;
                }
            }
        }
        if let Some(tag) = follow_me_storage.get_mut(self.follower_camera) {
            tag.rotation_target = mode.1.rotation_target.map(|_| (self.origin_camera, 0.01));
        }
    }

    pub fn set_camera_distance(&self, data: &StateData<'_, GameData<'_, '_>>, distance: f32) {
        let mut arc_ball_control_tag_storage = data.world.write_storage::<ArcBallControlTag>();
        for mut tag in (&mut arc_ball_control_tag_storage).join() {
            tag.distance = distance;
        }
    }
}

#[derive(Clone, Copy, From, Debug)]
pub struct AxialPosition(AxialVector, f32);

impl AxialPosition {
    fn pos(&self) -> &AxialVector {
        &self.0
    }

    fn alt(&self) -> f32 {
        self.1
    }
}

#[derive(Clone, Copy, From, Debug)]
pub struct QuadricPosition(QuadricVector);
