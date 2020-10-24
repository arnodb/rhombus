use crate::dispose::Dispose;
use crate::hex::render::renderer::HexRenderer;
use crate::world::RhombusViewerWorld;
use amethyst::prelude::*;
use rhombus_core::hex::coordinates::axial::AxialVector;
use rhombus_core::hex::storage::hash::RectHashStorage;

pub struct MultiRenderer<R1, R2> {
    r1: R1,
    r2: R2,
}

impl<R1, R2> MultiRenderer<R1, R2> {
    pub fn new(r1: R1, r2: R2) -> Self {
        Self { r1, r2 }
    }
}

impl<R1: HexRenderer, R2: HexRenderer> HexRenderer for MultiRenderer<R1, R2>
where
    R1: HexRenderer,
    R2: HexRenderer,
{
    type Hex = (R1::Hex, R2::Hex);

    fn new_hex(&mut self, wall: bool, visible: bool) -> Self::Hex {
        (
            self.r1.new_hex(wall, visible),
            self.r2.new_hex(wall, visible),
        )
    }

    fn set_cell_radius(&mut self, cell_radius: usize) {
        self.r1.set_cell_radius(cell_radius);
        self.r2.set_cell_radius(cell_radius);
    }

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
        Visible: Fn(AxialVector, &StorageHex) -> bool,
    {
        self.r1.update_world(
            hexes,
            &is_wall_hex,
            &is_visible_hex,
            // Ref of tuple to tuple of refs: it is supposedly safe because both the input ref and
            // the output ref are bound together, despite the fact that the ref to the tuple
            // returned by get_renderer_hex is floating in the middle.
            |hex| unsafe { &mut *(&mut get_renderer_hex(hex).0 as *mut R1::Hex) },
            visible_only,
            force,
            data,
            world,
        );
        self.r2.update_world(
            hexes,
            &is_wall_hex,
            &is_visible_hex,
            // Ref of tuple to tuple of refs: it is supposedly safe because both the input ref and
            // the output ref are bound together, despite the fact that the ref to the tuple
            // returned by get_renderer_hex is floating in the middle.
            |hex| unsafe { &mut *(&mut get_renderer_hex(hex).1 as *mut R2::Hex) },
            visible_only,
            force,
            data,
            world,
        );
    }

    fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.r1.clear(data);
        self.r2.clear(data);
    }
}
