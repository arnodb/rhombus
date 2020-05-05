use amethyst::{
    assets::Handle,
    renderer::{
        types::{Mesh, Texture},
        Material,
    },
};
use std::collections::{HashMap, VecDeque};

pub mod dodec;
pub mod hex;

#[derive(Debug)]
pub struct RhombusViewerAssets {
    pub hex_handle: Handle<Mesh>,
    pub color_data: HashMap<Color, ColorData>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Color {
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
}

#[derive(Debug, Clone)]
pub struct ColorData {
    pub texture: Handle<Texture>,
    pub material: Handle<Material>,
}

struct Snake<V, I> {
    radius: usize,
    state: VecDeque<V>,
    iter: I,
}
