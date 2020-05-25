use crate::world::RhombusViewerWorld;
use amethyst::prelude::*;
use rhombus_core::hex::coordinates::axial::AxialVector;

pub trait HexRenderer {
    fn insert_cell(
        &mut self,
        position: AxialVector,
        wall: bool,
        visible: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    );

    fn update_cell(
        &mut self,
        position: AxialVector,
        wall: bool,
        visible: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    );

    fn set_cell_radius(&mut self, cell_radius: usize, data: &mut StateData<'_, GameData<'_, '_>>);

    fn update_world<'a, C, I, Wall, Visible>(
        &mut self,
        cells: I,
        is_wall_cell: Wall,
        is_visible_cell: Visible,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        C: 'a,
        I: Iterator<Item = (&'a AxialVector, &'a C)>,
        Wall: Fn(AxialVector, &C) -> bool,
        Visible: Fn(AxialVector, &C) -> bool;

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>, _world: &RhombusViewerWorld) {
    }

    fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>);
}
