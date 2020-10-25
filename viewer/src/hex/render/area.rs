use crate::{dispose::Dispose, hex::render::renderer::HexRenderer, world::RhombusViewerWorld};
use amethyst::{
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rhombus_core::hex::{
    coordinates::axial::AxialVector, largest_area::LargestAreaIterator,
    storage::hash::RectHashStorage,
};

pub struct AreaRenderer {
    entity: Option<Entity>,
}

impl AreaRenderer {
    pub fn new() -> Self {
        Self { entity: None }
    }
}

impl HexRenderer for AreaRenderer {
    type Hex = ();

    fn new_hex(&mut self, _wall: bool, _visible: bool) {
        ()
    }

    fn update_world<'a, StorageHex, MapHex, Wall, Visible>(
        &mut self,
        hexes: &mut RectHashStorage<StorageHex>,
        is_wall_hex: Wall,
        is_visible_hex: Visible,
        _get_renderer_hex: MapHex,
        visible_only: bool,
        _force: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        StorageHex: 'a + Dispose,
        MapHex: Fn(&mut StorageHex) -> &mut Self::Hex,
        Wall: Fn(AxialVector, &StorageHex) -> bool,
        Visible: Fn(AxialVector, &StorageHex) -> bool,
    {
        self.clear(data);

        let mut wall_lai = LargestAreaIterator::default();
        let mut wall_acc = wall_lai.start_accumulation();
        let mut ground_lai = LargestAreaIterator::default();
        let mut ground_acc = ground_lai.start_accumulation();

        for (position, hex) in hexes.iter() {
            if !visible_only || is_visible_hex(position, hex) {
                if is_wall_hex(position, hex) {
                    &mut wall_acc
                } else {
                    &mut ground_acc
                }
                .push(position);
            }
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
