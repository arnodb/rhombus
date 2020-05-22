use amethyst::{core::Transform, derive::SystemDesc, ecs::prelude::*};
use std::collections::{hash_map::Entry, HashMap};

pub struct FollowMeTag {
    pub target: Option<(Entity, f32)>,
    pub rotation_target: Option<(Entity, f32)>,
}

impl Component for FollowMeTag {
    type Storage = HashMapStorage<FollowMeTag>;
}

#[derive(SystemDesc)]
pub struct FollowMeSystem;

const STAY_HERE_THRESHOLD: f32 = 0.01;

impl<'s> System<'s> for FollowMeSystem {
    type SystemData = (WriteStorage<'s, Transform>, ReadStorage<'s, FollowMeTag>);

    fn run(&mut self, (mut transforms, follow_me_tags): Self::SystemData) {
        let mut target_transforms = HashMap::new();
        for follow_me_tag in (&follow_me_tags).join() {
            for target in follow_me_tag
                .target
                .map(|t| t.0)
                .iter()
                .chain(follow_me_tag.rotation_target.map(|t| t.0).iter())
            {
                let entry = target_transforms.entry(*target);
                match entry {
                    Entry::Occupied(..) => {}
                    Entry::Vacant(..) => {
                        if let Some(target_transform) = transforms.get(*target) {
                            entry.or_insert_with(|| target_transform.clone());
                        }
                    }
                }
            }
        }
        for (transform, follow_me_tag) in (&mut transforms, &follow_me_tags).join() {
            if let Some((target, lerp_ratio)) = &follow_me_tag.target {
                if let Some(target_transform) = target_transforms.get(target) {
                    let delta = target_transform.translation() - transform.translation();
                    if delta[0].abs() >= STAY_HERE_THRESHOLD
                        || delta[1].abs() >= STAY_HERE_THRESHOLD
                        || delta[2].abs() >= STAY_HERE_THRESHOLD
                    {
                        transform.prepend_translation(delta * *lerp_ratio);
                    }
                }
            }
            if let Some((rotation_target, lerp_ratio)) = &follow_me_tag.rotation_target {
                if let Some(target_transform) = target_transforms.get(rotation_target) {
                    let target_rot = target_transform.rotation();
                    *transform.rotation_mut() =
                        transform.rotation().slerp(&target_rot, *lerp_ratio);
                }
            }
        }
    }
}

pub struct FollowMyRotationTag {
    pub targets: [Entity; 2],
    pub lerp_ratio: f32,
}

impl Component for FollowMyRotationTag {
    type Storage = HashMapStorage<FollowMyRotationTag>;
}

#[derive(SystemDesc)]
pub struct FollowMyRotationSystem;

impl<'s> System<'s> for FollowMyRotationSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, FollowMyRotationTag>,
    );

    fn run(&mut self, (mut transforms, follow_my_rotation_tags): Self::SystemData) {
        let mut target_transforms = HashMap::new();
        for follow_my_rotation_tag in (&follow_my_rotation_tags).join() {
            for target in &follow_my_rotation_tag.targets {
                let entry = target_transforms.entry(target);
                match entry {
                    Entry::Occupied(..) => {}
                    Entry::Vacant(..) => {
                        if let Some(target_transform) = transforms.get(*target) {
                            entry.or_insert_with(|| target_transform.clone());
                        }
                    }
                }
            }
        }
        for (transform, follow_my_rotation_tag) in
            (&mut transforms, &follow_my_rotation_tags).join()
        {
            if let (Some(target1_transform), Some(target2_transform)) = (
                target_transforms.get(&follow_my_rotation_tag.targets[0]),
                target_transforms.get(&follow_my_rotation_tag.targets[1]),
            ) {
                let target_rot = target2_transform.rotation() * target1_transform.rotation();
                *transform.rotation_mut() = transform
                    .rotation()
                    .slerp(&target_rot, follow_my_rotation_tag.lerp_ratio);
            }
        }
    }
}
