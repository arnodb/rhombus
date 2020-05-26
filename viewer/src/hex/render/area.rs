use crate::{hex::render::renderer::HexRenderer, world::RhombusViewerWorld};
use amethyst::{
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rhombus_core::hex::{coordinates::axial::AxialVector, largest_area::LargestAreaIterator};

pub struct AreaRenderer {
    entity: Option<Entity>,
}

impl AreaRenderer {
    pub fn new() -> Self {
        Self { entity: None }
    }
}

impl HexRenderer for AreaRenderer {
    fn insert_cell(
        &mut self,
        _position: AxialVector,
        _wall: bool,
        _visible: bool,
        _data: &mut StateData<'_, GameData<'_, '_>>,
        _world: &RhombusViewerWorld,
    ) {
    }

    fn update_cell(
        &mut self,
        _position: AxialVector,
        _wall: bool,
        _visible: bool,
        _data: &mut StateData<'_, GameData<'_, '_>>,
        _world: &RhombusViewerWorld,
    ) {
    }

    fn set_cell_radius(
        &mut self,
        _cell_radius: usize,
        _data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
    }

    fn update_world<'a, C, I, Wall, Visible>(
        &mut self,
        cells: I,
        is_wall_cell: Wall,
        _is_visible_cell: Visible,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        C: 'a,
        I: Iterator<Item = (&'a AxialVector, &'a C)>,
        Wall: Fn(AxialVector, &C) -> bool,
        Visible: Fn(AxialVector, &C) -> bool,
    {
        self.clear(data);

        let mut wall_lai = LargestAreaIterator::default();
        let mut wall_acc = wall_lai.start_accumulation();
        let mut ground_lai = LargestAreaIterator::default();
        let mut ground_acc = ground_lai.start_accumulation();

        for (position, cell) in cells {
            if is_wall_cell(*position, cell) {
                &mut wall_acc
            } else {
                &mut ground_acc
            }
            .push(*position);
        }

        let mut debug_lines = DebugLinesComponent::with_capacity(100);

        for (lai, color) in [
            (wall_lai, Srgba::new(1.0, 0.0, 0.0, 1.0)),
            (ground_lai, Srgba::new(1.0, 1.0, 1.0, 1.0)),
        ]
        .iter_mut()
        {
            loop {
                let area = lai.next_largest_area();
                if area.1.is_none() {
                    break;
                }
                if let Some((range_q, range_r)) = area.1 {
                    let mut p1 = world.axial_translation(
                        (AxialVector::new(*range_q.start(), *range_r.start()), 1.0).into(),
                    );
                    p1[0] -= 3.0_f32.sqrt() / 2.0;
                    p1[2] += 0.5;
                    let mut p2 = world.axial_translation(
                        (AxialVector::new(*range_q.start(), *range_r.end()), 1.0).into(),
                    );
                    p2[0] -= 1.0 / (3.0_f32.sqrt() * 2.0);
                    p2[2] -= 0.5;
                    let mut p3 = world.axial_translation(
                        (AxialVector::new(*range_q.end(), *range_r.end()), 1.0).into(),
                    );
                    p3[0] += 3.0_f32.sqrt() / 2.0;
                    p3[2] -= 0.5;
                    let mut p4 = world.axial_translation(
                        (AxialVector::new(*range_q.end(), *range_r.start()), 1.0).into(),
                    );
                    p4[0] += 1.0 / (3.0_f32.sqrt() * 2.0);
                    p4[2] += 0.5;
                    debug_lines.add_line(p1.into(), p2.into(), *color);
                    debug_lines.add_line(p2.into(), p3.into(), *color);
                    debug_lines.add_line(p3.into(), p4.into(), *color);
                    debug_lines.add_line(p4.into(), p1.into(), *color);
                }
            }
        }

        self.entity = Some(data.world.create_entity().with(debug_lines).build());
    }

    fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some(entity) = self.entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
    }
}
