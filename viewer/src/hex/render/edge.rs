use crate::{hex::render::renderer::HexRenderer, world::RhombusViewerWorld};
use amethyst::{
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rhombus_core::hex::coordinates::{axial::AxialVector, direction::HexagonalDirection};
use std::collections::{btree_map::Entry, BTreeMap};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Edge {
    Void = 0,
    DecreaseAltitude = 1,
    SameAltitude = 2,
    IncreaseAltitude = 3,
}

#[derive(Debug)]
struct Cell {
    wall: bool,
    visible: bool,
    edges: [Edge; 6],
}

pub struct EdgeRenderer {
    world: BTreeMap<AxialVector, Cell>,
    cell_radius: usize,
    dirty: bool,
    entity: Option<Entity>,
}

impl EdgeRenderer {
    pub fn new() -> Self {
        Self {
            world: BTreeMap::new(),
            cell_radius: 1,
            dirty: false,
            entity: None,
        }
    }

    fn add_lines(&self, debug_lines: &mut DebugLinesComponent, world: &RhombusViewerWorld) {
        let scale_factor = if self.cell_radius > 1 {
            (1.6 * self.cell_radius as f32).max(1.0)
        } else {
            1.0
        };
        for (position, cell) in &self.world {
            let translation = world.axial_translation((*position, 0.0).into());
            let small = 3.0_f32.sqrt() / 2.0;
            let color_factor = if cell.visible { 1.0 } else { 0.5 };
            let color = |edge: Edge| match edge {
                Edge::Void => {
                    if cell.wall {
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
                    if cell.wall {
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
                if *first_half || cell.edges[*dir] != Edge::SameAltitude {
                    debug_lines.add_line(
                        [
                            translation[0] + vertices[0].0 * scale_factor,
                            translation[1] + if cell.wall { 2.0 } else { 0.0 },
                            translation[2] + vertices[0].1 * scale_factor,
                        ]
                        .into(),
                        [
                            translation[0] + vertices[1].0 * scale_factor,
                            translation[1] + if cell.wall { 2.0 } else { 0.0 },
                            translation[2] + vertices[1].1 * scale_factor,
                        ]
                        .into(),
                        color(cell.edges[*dir]),
                    );
                }
            }
        }
    }
}

impl HexRenderer for EdgeRenderer {
    fn insert_cell(
        &mut self,
        position: AxialVector,
        wall: bool,
        visible: bool,
        _data: &mut StateData<'_, GameData<'_, '_>>,
        _world: &RhombusViewerWorld,
    ) {
        let mut cell = Cell {
            wall,
            visible,
            edges: [Edge::Void; 6],
        };
        for dir in 0..6 {
            let adjacent_pos = position + AxialVector::direction(dir);
            if let Some(adjacent) = self.world.get_mut(&adjacent_pos) {
                let (cell_edge, adjacent_edge) = if adjacent.visible == cell.visible {
                    match (cell.wall, adjacent.wall) {
                        (true, true) | (false, false) => (Edge::SameAltitude, Edge::SameAltitude),
                        (true, false) => (Edge::DecreaseAltitude, Edge::IncreaseAltitude),
                        (false, true) => (Edge::IncreaseAltitude, Edge::DecreaseAltitude),
                    }
                } else {
                    (Edge::Void, Edge::Void)
                };
                cell.edges[dir] = cell_edge;
                adjacent.edges[(dir + 3) % 6] = adjacent_edge;
            } else {
                cell.edges[dir] = Edge::Void
            }
        }
        match self.world.entry(position) {
            Entry::Vacant(entry) => {
                entry.insert(cell);
            }
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = cell;
            }
        }
        self.dirty = true;
    }

    fn update_cell(
        &mut self,
        position: AxialVector,
        wall: bool,
        visible: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        if self.world.remove(&position).is_some() {
            self.insert_cell(position, wall, visible, data, world);
            self.dirty = true;
        }
    }

    fn set_cell_radius(&mut self, cell_radius: usize, _data: &mut StateData<'_, GameData<'_, '_>>) {
        if self.cell_radius != cell_radius {
            self.cell_radius = cell_radius;
            self.dirty = true;
        }
    }

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
        Visible: Fn(AxialVector, &C) -> bool,
    {
        self.world.clear();
        for (position, cell) in cells {
            self.insert_cell(
                *position,
                is_wall_cell(*position, cell),
                is_visible_cell(*position, cell),
                data,
                world,
            );
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>, world: &RhombusViewerWorld) {
        if !self.dirty {
            return;
        }
        if let Some(entity) = self.entity {
            let mut debug_lines_storage = data.world.write_storage::<DebugLinesComponent>();
            let debug_lines = debug_lines_storage.get_mut(entity).expect("Debug lines");
            debug_lines.clear();
            self.add_lines(debug_lines, world);
        } else {
            let mut debug_lines = DebugLinesComponent::with_capacity(self.world.len() * 6);
            self.add_lines(&mut debug_lines, world);
            self.entity = Some(data.world.create_entity().with(debug_lines).build());
        }
        self.dirty = false;
    }

    fn clear(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        self.world = BTreeMap::new();
        if let Some(entity) = self.entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
        self.dirty = false;
    }
}
