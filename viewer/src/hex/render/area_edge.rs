use crate::{
    assets::Color, dispose::Dispose, hex::render::renderer::HexRenderer, world::RhombusViewerWorld,
};
use amethyst::{
    core::{math::Vector3, Transform},
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rhombus_core::hex::{
    coordinates::{
        axial::AxialVector,
        direction::{HexagonalDirection, NUM_DIRECTIONS},
    },
    storage::hash::RectHashStorage,
};
use smallvec::alloc::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Edge {
    None = 0,
    Void = 1,
    WallToOpen = 2,
    OpenToWall = 3,
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

pub struct AreaEdgeRenderer {
    cell_radius: usize,
    plane: Option<Entity>,
    entity: Option<Entity>,
    previous_visible_only: bool,
}

impl AreaEdgeRenderer {
    pub fn new() -> Self {
        Self {
            cell_radius: 1,
            plane: None,
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
        let mut visible_lines = [
            BTreeMap::<isize, Vec<isize>>::new(),
            BTreeMap::<isize, Vec<isize>>::new(),
            BTreeMap::<isize, Vec<isize>>::new(),
        ];
        let mut invisible_lines = [
            BTreeMap::<isize, Vec<isize>>::new(),
            BTreeMap::<isize, Vec<isize>>::new(),
            BTreeMap::<isize, Vec<isize>>::new(),
        ];
        for (position, hex) in hexes.iter_mut() {
            let hex = get_renderer_hex(hex);
            if visible_only && !hex.visible {
                continue;
            }
            for edge_num in 0..NUM_DIRECTIONS {
                let edge = hex.edges[edge_num];
                if edge == Edge::None || edge == Edge::Void {
                    continue;
                }
                let lines = if hex.visible {
                    &mut visible_lines[edge_num % 3]
                } else {
                    &mut invisible_lines[edge_num % 3]
                };
                match edge_num {
                    0 => lines
                        .entry(position.q())
                        .or_insert_with(Vec::new)
                        .push(position.r() * 2),
                    1 => lines
                        .entry(position.r())
                        .or_insert_with(Vec::new)
                        .push(position.q() * 2),
                    2 => lines
                        .entry(position.q() + position.r())
                        .or_insert_with(Vec::new)
                        .push(position.q() * 2),
                    3 => lines
                        .entry(position.q() - 1)
                        .or_insert_with(Vec::new)
                        .push(position.r() * 2 + 1),
                    4 => lines
                        .entry(position.r() + 1)
                        .or_insert_with(Vec::new)
                        .push(position.q() * 2 - 1),
                    5 => lines
                        .entry(position.q() + position.r() + 1)
                        .or_insert_with(Vec::new)
                        .push(position.q() * 2 + 1),
                    _ => unreachable!(),
                }
            }
        }
        let small = 3.0_f32.sqrt() / 2.0;
        let small_1_2 = small * 0.5;
        let small_3_2 = small * 1.5;
        for (visible, lines) in [(true, visible_lines), (false, invisible_lines)].iter_mut() {
            let floor_color = if *visible {
                Srgba::new(0.5, 0.5, 0.0, 1.0)
            } else {
                Srgba::new(0.1, 0.1, 0.1, 1.0)
            };
            let ceiling_color = if *visible {
                Srgba::new(0.3, 0.0, 0.0, 1.0)
            } else {
                Srgba::new(0.15, 0.0, 0.0, 1.0)
            };
            for (index, lines) in &mut lines[0] {
                if lines.is_empty() {
                    continue;
                }
                lines.sort();
                let add = |debug_lines: &mut DebugLinesComponent, start: isize, end: isize| {
                    let start_tr = world.axial_translation(
                        (AxialVector::new(*index, start.div_euclid(2)), 0.0).into(),
                    );
                    let start_x = if start & 1 == 0 { small_1_2 } else { small };
                    let start_z = if start & 1 == 0 { 0.75 } else { 0.0 };
                    let end_tr = world.axial_translation(
                        (AxialVector::new(*index, end.div_euclid(2)), 0.0).into(),
                    );
                    let end_x = if end & 1 == 0 { small } else { small_3_2 };
                    let end_z = if end & 1 == 0 { 0.0 } else { -0.75 };
                    debug_lines.add_line(
                        [start_tr[0] + start_x, 0.0, start_tr[2] + start_z].into(),
                        [end_tr[0] + end_x, 0.0, end_tr[2] + end_z].into(),
                        floor_color,
                    );
                    debug_lines.add_line(
                        [start_tr[0] + start_x, 1.0, start_tr[2] + start_z].into(),
                        [end_tr[0] + end_x, 1.0, end_tr[2] + end_z].into(),
                        ceiling_color,
                    );
                };
                let mut state = (lines[0], lines[0]);
                for next in &lines[1..] {
                    if state.1 + 1 == *next {
                        state.1 = *next;
                    } else {
                        add(debug_lines, state.0, state.1);
                        state = (*next, *next)
                    }
                }
                add(debug_lines, state.0, state.1);
            }
            for (index, lines) in &mut lines[1] {
                if lines.is_empty() {
                    continue;
                }
                lines.sort();
                let add = |debug_lines: &mut DebugLinesComponent, start: isize, end: isize| {
                    let start_tr = world.axial_translation(
                        (AxialVector::new(start.div_euclid(2), *index), 0.0).into(),
                    );
                    let start_x = if start & 1 == 0 {
                        -small_1_2
                    } else {
                        small_1_2
                    };
                    let end_tr = world.axial_translation(
                        (AxialVector::new(end.div_euclid(2), *index), 0.0).into(),
                    );
                    let end_x = if end & 1 == 0 { small_1_2 } else { small_3_2 };
                    debug_lines.add_line(
                        [start_tr[0] + start_x, 0.0, start_tr[2] + 0.75].into(),
                        [end_tr[0] + end_x, 0.0, end_tr[2] + 0.75].into(),
                        floor_color,
                    );
                    debug_lines.add_line(
                        [start_tr[0] + start_x, 1.0, start_tr[2] + 0.75].into(),
                        [end_tr[0] + end_x, 1.0, end_tr[2] + 0.75].into(),
                        ceiling_color,
                    );
                };
                let mut state = (lines[0], lines[0]);
                for next in &lines[1..] {
                    if state.1 + 1 == *next {
                        state.1 = *next;
                    } else {
                        add(debug_lines, state.0, state.1);
                        state = (*next, *next)
                    }
                }
                add(debug_lines, state.0, state.1);
            }
            for (index, lines) in &mut lines[2] {
                if lines.is_empty() {
                    continue;
                }
                lines.sort();
                let add = |debug_lines: &mut DebugLinesComponent, start: isize, end: isize| {
                    let start_tr = world.axial_translation(
                        (
                            AxialVector::new(start.div_euclid(2), *index - start.div_euclid(2)),
                            0.0,
                        )
                            .into(),
                    );
                    let start_x = if start & 1 == 0 { -small } else { -small_1_2 };
                    let start_z = if start & 1 == 0 { 0.0 } else { 0.75 };
                    let end_tr = world.axial_translation(
                        (
                            AxialVector::new(end.div_euclid(2), *index - end.div_euclid(2)),
                            0.0,
                        )
                            .into(),
                    );
                    let end_x = if end & 1 == 0 { -small_1_2 } else { 0.0 };
                    let end_z = if end & 1 == 0 { 0.75 } else { 1.5 };
                    debug_lines.add_line(
                        [start_tr[0] + start_x, 0.0, start_tr[2] + start_z].into(),
                        [end_tr[0] + end_x, 0.0, end_tr[2] + end_z].into(),
                        floor_color,
                    );
                    debug_lines.add_line(
                        [start_tr[0] + start_x, 1.0, start_tr[2] + start_z].into(),
                        [end_tr[0] + end_x, 1.0, end_tr[2] + end_z].into(),
                        ceiling_color,
                    );
                };
                let mut state = (lines[0], lines[0]);
                for next in &lines[1..] {
                    if state.1 + 1 == *next {
                        state.1 = *next;
                    } else {
                        add(debug_lines, state.0, state.1);
                        state = (*next, *next)
                    }
                }
                add(debug_lines, state.0, state.1);
            }
        }
    }
}

impl HexRenderer for AreaEdgeRenderer {
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
        _force: bool,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) where
        StorageHex: 'a + Dispose,
        MapHex: Fn(&mut StorageHex) -> &mut Self::Hex,
        Wall: Fn(AxialVector, &StorageHex) -> bool,
        Visible: Fn(AxialVector, &StorageHex) -> bool,
    {
        if self.plane.is_none() {
            let mut transform = Transform::default();
            transform.set_translation_xyz(0.0, -1.0, 0.0);
            transform.set_rotation_x_axis(-std::f32::consts::FRAC_PI_2);
            transform.set_scale(Vector3::new(100.0, 100.0, 1.0));
            self.plane = Some(
                data.world
                    .create_entity()
                    .with(world.assets.square_handle.clone())
                    .with(world.assets.color_data[&Color::White].dark.clone())
                    .with(transform)
                    .build(),
            )
        }

        let mut dirty = self.entity.is_none() || self.previous_visible_only != visible_only;
        for (position, mut hex_with_adjacents) in hexes.positions_and_hexes_with_adjacents_mut() {
            let wall = is_wall_hex(position, hex_with_adjacents.hex());
            let visible = is_visible_hex(position, hex_with_adjacents.hex());
            let hex = get_renderer_hex(hex_with_adjacents.hex());
            hex.wall = wall;
            hex.visible = visible;
            for edge_num in 0..NUM_DIRECTIONS {
                let dir_1 = edge_num;
                let adjacent_1_wall = hex_with_adjacents.adjacent(dir_1).and_then(|adj| {
                    let adj_wall = is_wall_hex(position.neighbor(dir_1), adj);
                    let adj_visible = is_visible_hex(position.neighbor(dir_1), adj);
                    if adj_visible == visible {
                        Some(adj_wall)
                    } else {
                        None
                    }
                });
                let dir_2 = (edge_num + 1) % NUM_DIRECTIONS;
                let adjacent_2_wall = hex_with_adjacents.adjacent(dir_2).and_then(|adj| {
                    let adj_wall = is_wall_hex(position.neighbor(dir_2), adj);
                    let adj_visible = is_visible_hex(position.neighbor(dir_2), adj);
                    if adj_visible == visible {
                        Some(adj_wall)
                    } else {
                        None
                    }
                });
                get_renderer_hex(hex_with_adjacents.hex()).edges[edge_num] =
                    match (adjacent_1_wall, adjacent_2_wall) {
                        (Some(adjacent_1_wall), Some(adjacent_2_wall)) => {
                            if wall != adjacent_1_wall && adjacent_1_wall == adjacent_2_wall {
                                if wall {
                                    Edge::WallToOpen
                                } else {
                                    Edge::OpenToWall
                                }
                            } else {
                                Edge::None
                            }
                        }
                        (Some(_), None) | (None, Some(_)) => Edge::None,
                        (None, None) => Edge::Void,
                    };
            }
            dirty = true;
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
        if let Some(plane) = self.plane.take() {
            data.world.delete_entity(plane).expect("delete entity");
        }
    }
}
