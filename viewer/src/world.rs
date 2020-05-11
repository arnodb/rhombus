use crate::{assets::RhombusViewerAssets, systems::follow_me::FollowMeTag};
use amethyst::{core::Transform, ecs::prelude::*, prelude::*};
use rhombus_core::{
    dodec::coordinates::quadric::QuadricVector, hex::coordinates::cubic::CubicVector,
};

#[derive(Debug)]
pub struct RhombusViewerWorld {
    pub assets: RhombusViewerAssets,
    pub origin: Entity,
    pub origin_camera: Entity,
    pub follower: Entity,
    pub follower_camera: Entity,
}

impl RhombusViewerWorld {
    pub fn transform_cubic(&self, position: CubicPosition, transform: &mut Transform) {
        let col = position.pos().x() + (position.pos().z() - (position.pos().z() & 1)) / 2;
        let row = position.pos().z();
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
        let mut follow_me_storage = data.world.write_storage::<FollowMeTag>();
        follow_me_storage.get_mut(self.follower).map(|tag| {
            tag.target = Some((target, 0.1));
            tag.rotation_target = rotation_target.map(|t| (t, 0.1));
        });
        if rotation_target.is_some() {
            let mut transform_storage = data.world.write_storage::<Transform>();
            let rotation = transform_storage
                .get(self.origin_camera)
                .map(Transform::rotation)
                .cloned();
            if let Some(rotation) = rotation {
                transform_storage
                    .get_mut(self.follower_camera)
                    .map(|transform| {
                        *transform.rotation_mut() = rotation;
                    });
            }
        }
        follow_me_storage.get_mut(self.follower_camera).map(|tag| {
            tag.rotation_target = rotation_target.map(|_| (self.origin_camera, 0.01));
        });
    }
}

#[derive(Debug, Clone, Copy, From)]
pub struct CubicPosition(CubicVector, f32);

impl CubicPosition {
    fn pos(&self) -> &CubicVector {
        &self.0
    }

    fn alt(&self) -> f32 {
        self.1
    }
}

#[derive(Debug, Clone, Copy, From)]
pub struct QuadricPosition(QuadricVector);
