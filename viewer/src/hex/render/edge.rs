use crate::{dispose::Dispose, hex::render::renderer::HexRenderer, world::RhombusViewerWorld};
use amethyst::{
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rhombus_core::hex::{coordinates::axial::AxialVector, storage::hash::RectHashStorage};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Edge {
    Void = 0,
    DecreaseAltitude = 1,
    SameAltitude = 2,
    IncreaseAltitude = 3,
}

#[derive(Debug)]
pub struct Hex {
    wall: bool,
    visible: bool,
    edges: [Edge; 6],
}

impl Dispose for Hex {
    fn dispose(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}

pub struct EdgeRenderer {
    cell_radius: usize,
    entity: Option<Entity>,
    previous_visible_only: bool,
}

impl EdgeRenderer {
    pub fn new() -> Self {
        Self {
            cell_radius: 1,
            entity: None,
            previous_visible_only: false,
        }
    }

    fn add_lines<StorageHex, MapHex>(
        &self,
        hexes: &mut RectHashStorage<StorageHex>,
        get_renderer_hex: MapHex,
        visible_only: bool,
        debug_lines: &mut DebugLinesComponent,
        world: &RhombusViewerWorld,
    ) where
        StorageHex: Dispose,
        MapHex: Fn(&mut StorageHex) -> &mut <Self as HexRenderer>::Hex,
    {
        let scale_factor = if self.cell_radius > 1 {
            (1.6 * self.cell_radius as f32).max(1.0)
        } else {
            1.0
        };
        for (position, hex) in hexes.iter_mut() {
            let hex = get_renderer_hex(hex);
            if visible_only && !hex.visible {
                continue;
            }
            let translation = world.axial_translation((position, 0.0).into());
            let small = 3.0_f32.sqrt() / 2.0;
            let color_factor = if hex.visible { 1.0 } else { 0.5 };
            let color = |edge: Edge| match edge {
                Edge::Void => {
                    if hex.wall {
                        Srgba::new(
                            1.0 * color_factor,
                            0.0 * color_factor,
                            1.0 * color_factor,
                            1.0,
                        )
                    } else {
                        Srgba::new(
                            0.0 * color_factor,
                            1.0 * color_factor,
                            1.0 * color_factor,
                            1.0,
                        )
                    }
                }
                Edge::DecreaseAltitude => Srgba::new(
                    1.0 * color_factor,
                    0.0 * color_factor,
                    0.0 * color_factor,
                    1.0,
                ),
                Edge::SameAltitude => {
                    if hex.wall {
                        Srgba::new(
                            0.3 * color_factor,
                            0.0 * color_factor,
                            0.0 * color_factor,
                            1.0,
                        )
                    } else {
                        Srgba::new(
                            0.3 * color_factor,
                            0.3 * color_factor,
                            0.3 * color_factor,
                            1.0,
                        )
                    }
                }
                Edge::IncreaseAltitude => Srgba::new(
                    0.1 * color_factor,
                    0.1 * color_factor,
                    0.1 * color_factor,
                    1.0,
                ),
            };
            for (dir, vertices, first_half) in [
                (0, [(small, -0.5), (small, 0.5)], true),
                (1, [(small, 0.5), (0.0, 1.0)], true),
                (2, [(0.0, 1.0), (-small, 0.5)], true),
                (3, [(-small, 0.5), (-small, -0.5)], false),
                (4, [(-small, -0.5), (0.0, -1.0)], false),
                (5, [(0.0, -1.0), (small, -0.5)], false),
            ]
            .iter()
            {
                if *first_half || hex.edges[*dir] != Edge::SameAltitude {
                    debug_lines.add_line(
                        [
                            translation[0] + vertices[0].0 * scale_factor,
                            translation[1] + if hex.wall { 2.0 } else { 0.0 },
                            translation[2] + vertices[0].1 * scale_factor,
                        ]
                        .into(),
                        [
                            translation[0] + vertices[1].0 * scale_factor,
                            translation[1] + if hex.wall { 2.0 } else { 0.0 },
                            translation[2] + vertices[1].1 * scale_factor,
                        ]
                        .into(),
                        color(hex.edges[*dir]),
                    );
                }
            }
        }
    }
}

impl HexRenderer for EdgeRenderer {
    type Hex = Hex;

    fn new_hex(&mut self, wall: bool, visible: bool) -> Self::Hex {
        Hex {
            wall,
            visible,
            edges: [Edge::Void; 6],
        }
    }

    fn set_cell_radius(&mut self, cell_radius: usize) {
        self.cell_radius = cell_radius;
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
        let mut dirty = self.entity.is_none() || self.previous_visible_only != visible_only;
        for (position, mut hex_with_adjacents) in hexes.positions_and_hexes_with_adjacents_mut() {
            let wall = is_wall_hex(position, hex_with_adjacents.hex());
            let visible = is_visible_hex(position, hex_with_adjacents.hex());
            if force
                || get_renderer_hex(hex_with_adjacents.hex()).wall != wall
                || get_renderer_hex(hex_with_adjacents.hex()).visible != visible
            {
                for dir in 0..6 {
                    let hex_edge = if let Some(adjacent) = hex_with_adjacents.adjacent(dir) {
                        let adjacent = get_renderer_hex(adjacent);
                        let (hex_edge, adjacent_edge) = if adjacent.visible == visible {
                            match (wall, adjacent.wall) {
                                (true, true) | (false, false) => {
                                    (Edge::SameAltitude, Edge::SameAltitude)
                                }
                                (true, false) => (Edge::DecreaseAltitude, Edge::IncreaseAltitude),
                                (false, true) => (Edge::IncreaseAltitude, Edge::DecreaseAltitude),
                            }
                        } else {
                            (Edge::Void, Edge::Void)
                        };
                        adjacent.edges[(dir + 3) % 6] = adjacent_edge;
                        hex_edge
                    } else {
                        Edge::Void
                    };
                    get_renderer_hex(hex_with_adjacents.hex()).edges[dir] = hex_edge;
                }
                get_renderer_hex(hex_with_adjacents.hex()).wall = wall;
                get_renderer_hex(hex_with_adjacents.hex()).visible = visible;
                dirty = true;
            }
        }
        if dirty {
            if let Some(entity) = self.entity {
                let mut debug_lines_storage = data.world.write_storage::<DebugLinesComponent>();
                let debug_lines = debug_lines_storage.get_mut(entity).expect("Debug lines");
                debug_lines.clear();
                self.add_lines(hexes, get_renderer_hex, visible_only, debug_lines, world);
            } else {
                let mut debug_lines = DebugLinesComponent::with_capacity(100);
                self.add_lines(
                    hexes,
                    get_renderer_hex,
                    visible_only,
                    &mut debug_lines,
                    world,
                );
                self.entity = Some(data.world.create_entity().with(debug_lines).build());
            }
        }
        self.previous_visible_only = visible_only;
    }

    fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        if let Some(entity) = self.entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
    }
}
