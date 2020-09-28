use crate::{
    assets::Color,
    hex::{cellular::world::FovState, pointer::HexPointer, shape::cubic_range::CubicRangeShape},
    world::RhombusViewerWorld,
};
use amethyst::{
    core::{math::Vector3, transform::Transform},
    ecs::prelude::*,
    prelude::*,
    renderer::{debug_drawing::DebugLinesComponent, palette::Srgba},
};
use rand::{thread_rng, Rng};
use rhombus_core::hex::coordinates::{axial::AxialVector, cubic::CubicVector};
use std::sync::Arc;

const CELL_RADIUS: usize = 3;

pub struct World {
    shape: CubicRangeShape,
    shape_positions: Vec<AxialVector>,
    limits_entity: Option<Entity>,
    rooms: Vec<(CubicRangeShape, Entity)>,
    perimeter_entities: Vec<Entity>,
    pointer: Option<(HexPointer, FovState)>,
}

impl World {
    pub fn new() -> Self {
        Self {
            shape: CubicRangeShape::default(),
            shape_positions: Vec::new(),
            limits_entity: None,
            rooms: Vec::new(),
            perimeter_entities: Vec::new(),
            pointer: None,
        }
    }

    pub fn set_shape_and_reset_world(
        &mut self,
        shape: CubicRangeShape,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        self.shape = shape;
        let mut r = 0;
        loop {
            let mut end = true;
            for pos in self.shape.center().big_ring_iter(CELL_RADIUS, r) {
                let mut positions = Vec::new();
                for v in pos.ring_iter(CELL_RADIUS) {
                    if self.shape.contains_position(v) {
                        positions.push(v);
                    }
                }
                if positions.is_empty() {
                    continue;
                }
                end = false;
                self.shape_positions.extend(positions);
                for s in 0..CELL_RADIUS {
                    for v in pos.ring_iter(s) {
                        if self.shape.contains_position(v) {
                            self.shape_positions.push(v);
                        }
                    }
                }
            }
            if end {
                break;
            }
            r += 1;
        }
        self.reset_world(data);
    }

    pub fn reset_world(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.clear(data, &world);

        if let Some(entity) = self.limits_entity {
            let mut debug_lines_storage = data.world.write_storage::<DebugLinesComponent>();
            let debug_lines = debug_lines_storage.get_mut(entity).expect("Debug lines");
            debug_lines.clear();
            Self::add_limit_lines(
                &self.shape,
                Srgba::new(0.2, 0.2, 0.2, 1.0),
                debug_lines,
                &world,
            );
        } else {
            let mut debug_lines = DebugLinesComponent::with_capacity(6);
            Self::add_limit_lines(
                &self.shape,
                Srgba::new(0.2, 0.2, 0.2, 1.0),
                &mut debug_lines,
                &world,
            );
            self.limits_entity = Some(data.world.create_entity().with(debug_lines).build());
        }

        for pos in self.shape.perimeter() {
            let mut transform = Transform::default();
            transform.set_scale(Vector3::new(0.8, 0.08, 0.8));
            let pos = (pos, 0.0).into();
            world.transform_axial(pos, &mut transform);
            let material = world.assets.color_data[&Color::Red].light.clone();
            let entity = data
                .world
                .create_entity()
                .with(world.assets.hex_handle.clone())
                .with(material)
                .with(transform)
                .build();
            self.perimeter_entities.push(entity);
        }
    }

    pub fn clear(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        for entity in self.perimeter_entities.iter() {
            data.world.delete_entity(*entity).expect("delete entity");
        }
        self.perimeter_entities.clear();
        self.delete_pointer(data, world);
        for (_, entity) in self.rooms.iter() {
            data.world.delete_entity(*entity).expect("delete entity");
        }
        self.rooms.clear();
        if let Some(entity) = self.limits_entity.take() {
            data.world.delete_entity(entity).expect("delete entity");
        }
    }

    fn delete_pointer(
        &mut self,
        data: &mut StateData<'_, GameData<'_, '_>>,
        world: &RhombusViewerWorld,
    ) {
        if let Some((mut pointer, _)) = self.pointer.take() {
            pointer.delete_entities(data, world);
        }
    }

    fn add_limit_lines(
        shape: &CubicRangeShape,
        color: Srgba,
        debug_lines: &mut DebugLinesComponent,
        world: &RhombusViewerWorld,
    ) {
        let translations = shape
            .vertices()
            .iter()
            .map(|v| world.axial_translation((*v, 2.0).into()))
            .collect::<Vec<[f32; 3]>>();
        debug_lines.add_line(translations[0].into(), translations[1].into(), color);
        debug_lines.add_line(translations[1].into(), translations[2].into(), color);
        debug_lines.add_line(translations[2].into(), translations[3].into(), color);
        debug_lines.add_line(translations[3].into(), translations[4].into(), color);
        debug_lines.add_line(translations[4].into(), translations[5].into(), color);
        debug_lines.add_line(translations[5].into(), translations[0].into(), color);
    }

    pub fn room_phase_step(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let mut deltas = [
            self.shape.range_x().end() - self.shape.range_x().start(),
            self.shape.range_y().end() - self.shape.range_y().start(),
            self.shape.range_z().end() - self.shape.range_z().start(),
        ];
        deltas.sort();
        let radius = deltas[1] / 10;

        let mut rng = thread_rng();

        let mut new_room =
            CubicRangeShape::new((-radius, radius), (-radius, radius), (-radius, radius));
        let funcs: [(
            fn(&mut CubicRangeShape, usize) -> bool,
            fn(&mut CubicRangeShape, usize) -> bool,
        ); 6] = [
            (
                CubicRangeShape::shrink_x_start,
                CubicRangeShape::stretch_x_start,
            ),
            (
                CubicRangeShape::shrink_x_end,
                CubicRangeShape::stretch_x_end,
            ),
            (
                CubicRangeShape::shrink_y_start,
                CubicRangeShape::stretch_y_start,
            ),
            (
                CubicRangeShape::shrink_y_end,
                CubicRangeShape::stretch_y_end,
            ),
            (
                CubicRangeShape::shrink_z_start,
                CubicRangeShape::stretch_z_start,
            ),
            (
                CubicRangeShape::shrink_z_end,
                CubicRangeShape::stretch_z_end,
            ),
        ];
        for (st, sh) in funcs.iter() {
            let d = rng.gen_range(-radius / 3, radius / 3 + 1);
            for _ in 0..d.abs() {
                if d > 0 {
                    st(&mut new_room, 2);
                } else if d < 0 {
                    sh(&mut new_room, 2);
                }
            }
        }

        let random_pos: CubicVector =
            (*&self.shape_positions[rng.gen_range(0, self.shape_positions.len())]).into();

        let mut start_x = new_room.range_x().start() + random_pos.x();
        let delta_x = (start_x - self.shape.range_x().start() + 1) % 2;
        start_x += delta_x;
        let end_x = new_room.range_x().end() + random_pos.x() + delta_x;

        let mut start_y = new_room.range_y().start() + random_pos.y();
        let delta_y = (start_y - self.shape.range_y().start() + 1) % 2;
        start_y += delta_y;
        let end_y = new_room.range_y().end() + random_pos.y() + delta_y;

        let start_z = new_room.range_z().start() + random_pos.z() - delta_x - delta_y;
        let end_z = new_room.range_z().end() + random_pos.z() - delta_x - delta_y;

        let is_inside_shape = self.shape.range_x().start() < start_x
            && self.shape.range_x().end() > end_x
            && self.shape.range_y().start() < start_y
            && self.shape.range_y().end() > end_y
            && self.shape.range_z().start() < start_z
            && self.shape.range_z().end() > end_z;
        let new_room = CubicRangeShape::new((start_x, end_x), (start_y, end_y), (start_z, end_z));

        if is_inside_shape
            && !self
                .rooms
                .iter()
                .any(|(room, _)| room.intersects(&new_room))
        {
            let mut debug_lines = DebugLinesComponent::with_capacity(6);
            let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
            Self::add_limit_lines(
                &new_room,
                Srgba::new(0.5, 0.5, 0.5, 1.0),
                &mut debug_lines,
                &world,
            );
            let new_entity = data.world.create_entity().with(debug_lines).build();

            for pos in new_room.perimeter() {
                let mut transform = Transform::default();
                transform.set_scale(Vector3::new(0.8, 0.08, 0.8));
                let pos = (pos, 0.0).into();
                world.transform_axial(pos, &mut transform);
                let material = world.assets.color_data[&Color::Green].light.clone();
                let entity = data
                    .world
                    .create_entity()
                    .with(world.assets.hex_handle.clone())
                    .with(material)
                    .with(transform)
                    .build();
                self.perimeter_entities.push(entity);
            }

            self.rooms.push((new_room, new_entity));
        }
    }

    pub fn create_pointer(
        &mut self,
        _fov_state: FovState,
        data: &mut StateData<'_, GameData<'_, '_>>,
    ) {
        let world = (*data.world.read_resource::<Arc<RhombusViewerWorld>>()).clone();
        self.delete_pointer(data, &world);

        // TODO
    }

    pub fn update_renderer_world(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) {}
}
