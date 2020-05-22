use amethyst::{
    assets::Handle,
    renderer::{
        types::{Mesh, Texture},
        Material,
    },
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RhombusViewerAssets {
    pub hex_handle: Handle<Mesh>,
    pub dodec_handle: Handle<Mesh>,
    pub pointer_handle: Handle<Mesh>,
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
    pub light: TextureAndMaterial,
    pub dark: TextureAndMaterial,
}

#[derive(Debug, Clone)]
pub struct TextureAndMaterial {
    pub texture: Handle<Texture>,
    pub material: Handle<Material>,
}
