use amethyst::winit::{
    ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent,
};

pub fn get_key_and_modifiers(
    event: &Event,
) -> Option<(VirtualKeyCode, ElementState, ModifiersState)> {
    match *event {
        Event::WindowEvent { ref event, .. } => match *event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(ref virtual_keycode),
                        state,
                        modifiers,
                        ..
                    },
                ..
            } => Some((*virtual_keycode, state, modifiers)),
            _ => None,
        },
        _ => None,
    }
}
