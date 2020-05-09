use amethyst::{core::Transform, derive::SystemDesc, ecs::prelude::*};
use std::collections::{hash_map::Entry, HashMap};

pub struct FollowMeTag {
    pub target: Entity,
}

impl Component for FollowMeTag {
    // we can use HashMapStorage here because, according to the specs doc, this storage should be
    // use when the component is used with few entity, I think there will rarely more than one
    // camera
    type Storage = HashMapStorage<FollowMeTag>;
}

#[derive(SystemDesc)]
pub struct FollowMeSystem;

impl<'s> System<'s> for FollowMeSystem {
    type SystemData = (WriteStorage<'s, Transform>, ReadStorage<'s, FollowMeTag>);

    fn run(&mut self, (mut transforms, follow_me_tags): Self::SystemData) {
        let mut target_transforms = HashMap::new();
        for follow_me_tag in (&follow_me_tags).join() {
            let entry = target_transforms.entry(follow_me_tag.target);
            match entry {
                Entry::Occupied(..) => {}
                Entry::Vacant(..) => {
                    if let Some(target_transform) = transforms.get(follow_me_tag.target) {
                        entry.or_insert(target_transform.clone());
                    }
                }
            }
        }
        for (transform, follow_me_tag) in (&mut transforms, &follow_me_tags).join() {
            if let Some(target_transform) = target_transforms.get(&follow_me_tag.target) {
                let delta = target_transform.translation() - transform.translation();
                if delta[0].abs() >= 0.1 || delta[1].abs() >= 0.01 || delta[2].abs() >= 0.01 {
                    transform.prepend_translation(delta / 10.0);
                }
            }
        }
    }
}
