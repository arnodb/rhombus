use crate::{dispose::Dispose, world::RhombusViewerWorld};
use amethyst::prelude::*;
use rhombus_core::hex::{coordinates::axial::AxialVector, storage::hash::RectHashStorage};

pub trait HexRenderer {
    type Hex: Dispose;

    fn new_hex(&mut self, wall: bool, visible: bool) -> Self::Hex;

    fn set_cell_radius(&mut self, cell_radius: usize);

    fn update_world<'a, StorageHex, MapHex, Wall, Visible>(
        &mut self,
        hexes: &mut RectHashStorage<StorageHex>,
        is_wall_hex: Wall,
        is_visible_hex: Visible,
        get_renderer_hex: MapHex,
        visible_only: bool,
        force: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        StorageHex: 'a + Dispose,
        MapHex: Fn(&mut StorageHex) -> &mut Self::Hex,
        Wall: Fn(AxialVector, &StorageHex) -> bool,
        Visible: Fn(AxialVector, &StorageHex) -> bool;

    fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>);
}
