use amethyst::{
    controls::ArcBallControlTag,
    core::{shrev::EventChannel, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputEvent, ScrollDirection, StringBindings},
};

#[derive(SystemDesc)]
#[system_desc(name(CameraDistanceSystemDesc))]
pub struct CameraDistanceSystem {
    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<InputEvent<StringBindings>>,
}

impl CameraDistanceSystem {
    pub fn new(event_reader: ReaderId<InputEvent<StringBindings>>) -> Self {
        CameraDistanceSystem { event_reader }
    }
}

impl<'a> System<'a> for CameraDistanceSystem {
    type SystemData = (
        Read<'a, EventChannel<InputEvent<StringBindings>>>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, ArcBallControlTag>,
    );

    fn run(&mut self, (events, transforms, mut tags): Self::SystemData) {
        for event in events.read(&mut self.event_reader) {
            if let InputEvent::MouseWheelMoved(direction) = *event {
                match direction {
                    ScrollDirection::ScrollUp => {
                        for (_, tag) in (&transforms, &mut tags).join() {
                            tag.distance *= 0.9;
                        }
                    }
                    ScrollDirection::ScrollDown => {
                        for (_, tag) in (&transforms, &mut tags).join() {
                            tag.distance *= 1.1;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
