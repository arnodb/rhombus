use crate::{
    hex::coordinates::{
        axial::AxialVector, cubic::CubicVector, direction::HexagonalDirection, HexagonalVector,
    },
    vector::Vector2ISize,
};
use std::{cmp::Ordering, fmt::Debug};

#[derive(Default, Debug)]
pub struct FieldOfView<V: HexagonalVector> {
    center: V,
    radius: usize,
    arcs: Vec<Arc>,
}

impl<V: HexagonalVector + HexagonalDirection + Into<VertexVector>> FieldOfView<V> {
    pub fn start(&mut self, center: V) {
        self.center = center;
        self.radius = 1;
        self.arcs.clear();
        self.arcs.push(Arc {
            start: ArcEnd {
                polar_index: 0,
                vector: VertexVector(Vector2ISize { x: 3, y: 0 }),
            },
            stop: ArcEnd {
                polar_index: 3,
                vector: VertexVector(Vector2ISize { x: -3, y: 0 }),
            },
        });
        self.arcs.push(Arc {
            start: ArcEnd {
                polar_index: 3,
                vector: VertexVector(Vector2ISize { x: -3, y: 0 }),
            },
            stop: ArcEnd {
                polar_index: 6,
                vector: VertexVector(Vector2ISize { x: 3, y: 0 }),
            },
        });
    }

    pub fn next_radius<F>(&mut self, is_obstacle: &F)
    where
        F: Fn(V) -> bool,
    {
        let radius = self.radius;
        let mut expanded_arcs = Vec::new();
        for arc in &mut self.arcs {
            expanded_arcs.extend(
                arc.clone()
                    .split(self.center, radius, is_obstacle)
                    .into_iter()
                    .map(|mut arc| {
                        arc.expand::<V>(radius);
                        arc
                    }),
            );
        }
        self.arcs = expanded_arcs;
        self.radius = radius + 1;
    }

    pub fn iter(&self) -> ArcsIter<'_, V> {
        ArcsIter::new(self.radius, self.arcs.iter())
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Arc {
    start: ArcEnd,
    stop: ArcEnd,
}

impl Arc {
    fn is_zero_angle(&self) -> bool {
        if self.start.polar_index > self.stop.polar_index {
            return true;
        }
        match self.start.vector.turns(&self.stop.vector) {
            Turn::Right => true,
            // The good thing with VertexVector holding integral values is that there is no need
            // to introduce an epsilon here. However vectors could be opposite because
            // the system is pushed to PI angles inclusive. This probably would not be necessary
            // if the field of view was split into 3 slices or 4 quadrants like in the reference
            // implementation.
            // It is difficult to tell which implementation is more optimal CPU-wise without
            // proper measurement.
            Turn::Straight
                if self.start.vector.0.x < 0 && self.stop.vector.0.x < 0
                    || self.start.vector.0.x > 0 && self.stop.vector.0.x > 0
                    || self.start.vector.0.y < 0 && self.stop.vector.0.y < 0
                    || self.start.vector.0.y > 0 && self.stop.vector.0.y > 0 =>
            {
                true
            }
            Turn::Left | Turn::Straight => false,
        }
    }

    fn split<V: HexagonalDirection + Into<VertexVector>, F>(
        mut self,
        center: V,
        radius: usize,
        is_obstacle: &F,
    ) -> Vec<Arc>
    where
        F: Fn(V) -> bool,
    {
        let mut split = Vec::new();
        loop {
            // Contract start
            while self.start.polar_index <= self.stop.polar_index {
                let vector = ArcEnd::polar_index_to_vector(self.start.polar_index, radius);
                if is_obstacle(center + vector) {
                    self.start.contract_start(vector);
                    self.start.polar_index += 1;
                } else {
                    break;
                }
            }
            // Find stop obstacle
            let mut polar_index = self.start.polar_index;
            while polar_index <= self.stop.polar_index {
                let vector = ArcEnd::polar_index_to_vector(polar_index, radius);
                if is_obstacle(center + vector) {
                    let mut arc = self.clone();
                    // Contract stop
                    arc.stop.contract_stop(vector);
                    arc.stop.polar_index = polar_index - 1;
                    // Push if non zero angle
                    if !arc.is_zero_angle() {
                        split.push(arc);
                    }
                    // Reset start for next iteration
                    self.start.contract_start(vector);
                    self.start.polar_index = polar_index + 1;
                    break;
                } else {
                    polar_index += 1;
                }
            }
            // No obstacle found
            if polar_index > self.stop.polar_index {
                break;
            }
        }
        // Push last if non zero angle
        if !self.is_zero_angle() {
            split.push(self);
        }
        split
    }

    fn expand<V: HexagonalDirection + Into<VertexVector>>(&mut self, radius: usize) {
        self.start.expand_start::<V>(radius);
        self.stop.expand_stop::<V>(radius);
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct ArcEnd {
    polar_index: usize,
    vector: VertexVector,
}

impl ArcEnd {
    fn polar_index_to_vector<V: HexagonalDirection>(polar_index: usize, radius: usize) -> V {
        let side = (polar_index / radius) % 6;
        let side_offset = polar_index % radius;
        V::direction(side) * radius as isize + V::direction((side + 2) % 6) * side_offset as isize
    }

    fn is_right_of_arc<V: HexagonalDirection + Into<VertexVector>>(&self, radius: usize) -> bool {
        let vector = ArcEnd::polar_index_to_vector::<V>(self.polar_index, radius);
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = vector.into() + *local_vertex;
            if self.vector.turns(&vertex) == Turn::Right {
                return true;
            }
        }
        false
    }

    fn is_left_of_arc<V: HexagonalDirection + Into<VertexVector>>(&self, radius: usize) -> bool {
        let vector = ArcEnd::polar_index_to_vector::<V>(self.polar_index, radius);
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = vector.into() + *local_vertex;
            if self.vector.turns(&vertex) == Turn::Left {
                return true;
            }
        }
        false
    }

    fn contract_start<V: HexagonalVector + Into<VertexVector>>(&mut self, vector: V) {
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = vector.into() + *local_vertex;
            if self.vector.turns(&vertex) == Turn::Left {
                self.vector = vertex;
            }
        }
    }

    fn contract_stop<V: HexagonalVector + Into<VertexVector>>(&mut self, vector: V) {
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = vector.into() + *local_vertex;
            if self.vector.turns(&vertex) == Turn::Right {
                self.vector = vertex;
            }
        }
    }

    fn expand_start<V: HexagonalDirection + Into<VertexVector>>(&mut self, radius: usize) {
        let side = self.polar_index / radius;
        let side_offset = self.polar_index % radius;
        let new_radius = radius + 1;
        self.polar_index = side * new_radius + side_offset + 1;
        loop {
            if self.is_right_of_arc::<V>(new_radius) {
                // Found one hex crossing the arc.
                // Check whether the next one is in the same case before breaking the loop.
                if self.polar_index > 0 {
                    self.polar_index -= 1;
                    if !self.is_left_of_arc::<V>(new_radius) {
                        self.polar_index += 1;
                    }
                }
                break;
            }
            debug_assert!(self.polar_index > 0);
            self.polar_index -= 1;
        }
    }

    fn expand_stop<V: HexagonalDirection + Into<VertexVector>>(&mut self, radius: usize) {
        let side = self.polar_index / radius;
        let side_offset = self.polar_index % radius;
        let new_radius = radius + 1;
        self.polar_index = side * new_radius + side_offset;
        loop {
            if self.is_left_of_arc::<V>(new_radius) {
                // Found one hex crossing the arc.
                // Check whether the next one is in the same case before breaking the loop.
                if self.polar_index < new_radius * 6 {
                    self.polar_index += 1;
                    if !self.is_right_of_arc::<V>(new_radius) {
                        self.polar_index -= 1;
                    }
                }
                break;
            }
            debug_assert!(self.polar_index < new_radius * 6);
            self.polar_index += 1;
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Turn {
    Left,
    Straight,
    Right,
}

#[derive(PartialEq, Eq, Clone, Copy, Add, AddAssign, Sub, SubAssign, Debug)]
pub struct VertexVector(Vector2ISize);

impl VertexVector {
    fn turns(&self, other: &VertexVector) -> Turn {
        let cross = self.0.x * other.0.y - self.0.y * other.0.x;
        match cross.cmp(&0) {
            Ordering::Greater => Turn::Left,
            Ordering::Less => Turn::Right,
            Ordering::Equal => Turn::Straight,
        }
    }
}

const HEX_PLANE_VERTICES: [VertexVector; 6] = [
    VertexVector(Vector2ISize { x: 1, y: -1 }),
    VertexVector(Vector2ISize { x: 1, y: 1 }),
    VertexVector(Vector2ISize { x: 0, y: 2 }),
    VertexVector(Vector2ISize { x: -1, y: 1 }),
    VertexVector(Vector2ISize { x: -1, y: -1 }),
    VertexVector(Vector2ISize { x: 0, y: -2 }),
];

impl From<AxialVector> for VertexVector {
    fn from(axial: AxialVector) -> Self {
        VertexVector(Vector2ISize {
            x: 2 * axial.q() + axial.r(),
            y: -3 * axial.r(),
        })
    }
}

impl From<CubicVector> for VertexVector {
    fn from(cubic: CubicVector) -> Self {
        VertexVector(Vector2ISize {
            x: 2 * cubic.x() + cubic.z(),
            y: -3 * cubic.z(),
        })
    }
}

pub struct ArcsIter<'a, V> {
    radius: usize,
    arcs: std::slice::Iter<'a, Arc>,
    current: Option<(&'a Arc, usize, usize)>,
    _v: std::marker::PhantomData<V>,
}

impl<'a, V> ArcsIter<'a, V> {
    fn new(radius: usize, mut arcs: std::slice::Iter<'a, Arc>) -> Self {
        let current = arcs
            .next()
            .map(|arc| (arc, arc.start.polar_index, arc.start.polar_index));
        Self {
            radius,
            arcs,
            current,
            _v: Default::default(),
        }
    }
}

impl<'a, V: HexagonalDirection> Iterator for ArcsIter<'a, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((arc, polar_index, first_polar_index)) = &mut self.current {
            let first_polar_index = *first_polar_index;
            let res = Some(ArcEnd::polar_index_to_vector(*polar_index, self.radius));
            let next_polar_index = *polar_index + 1;
            if next_polar_index <= arc.stop.polar_index
                && next_polar_index % (self.radius * 6) != first_polar_index
            {
                *polar_index = next_polar_index;
            } else {
                let prev_polar_index = *polar_index;
                loop {
                    self.current = self
                        .arcs
                        .next()
                        .map(|arc| (arc, arc.start.polar_index, first_polar_index));
                    match &mut self.current {
                        Some((arc, pi, _)) => {
                            if *pi == prev_polar_index {
                                *pi += 1;
                            }
                            if *pi <= arc.stop.polar_index
                                && *pi % (self.radius * 6) != first_polar_index
                            {
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
            res
        } else {
            None
        }
    }
}

#[test]
fn test_field_of_view_2_0() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(0));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 11,
                    vector: VertexVector(Vector2ISize { x: 1, y: -1 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_1() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(1));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: 0, y: 4 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 12,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_2() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(2));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 12,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_3() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(3));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: -1, y: 1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 7,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                },
                stop: ArcEnd {
                    polar_index: 12,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_4() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(4));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 7,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 12,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_5() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(5));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: 0, y: -2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 11,
                    vector: VertexVector(Vector2ISize { x: 2, y: -2 })
                },
                stop: ArcEnd {
                    polar_index: 12,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_024() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(0));
        set.insert(center + AxialVector::direction(2));
        set.insert(center + AxialVector::direction(4));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 7,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 11,
                    vector: VertexVector(Vector2ISize { x: 1, y: -1 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_024_3_() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(0));
        set.insert(center + AxialVector::direction(2));
        set.insert(center + AxialVector::direction(4));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 3);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 7,
                    vector: VertexVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 11,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 13,
                    vector: VertexVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 17,
                    vector: VertexVector(Vector2ISize { x: 1, y: -1 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_2_024_3_1357911() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(0));
        set.insert(center + AxialVector::direction(2));
        set.insert(center + AxialVector::direction(4));
        set.insert(center + AxialVector::direction(0) + AxialVector::direction(1));
        set.insert(center + AxialVector::direction(1) + AxialVector::direction(2));
        set.insert(center + AxialVector::direction(2) + AxialVector::direction(3));
        set.insert(center + AxialVector::direction(3) + AxialVector::direction(4));
        set.insert(center + AxialVector::direction(4) + AxialVector::direction(5));
        set.insert(center + AxialVector::direction(5) + AxialVector::direction(0));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&|pos| obstacles.contains(&pos));
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.radius, 3);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 2,
                    vector: VertexVector(Vector2ISize { x: 2, y: 4 })
                },
                stop: ArcEnd {
                    polar_index: 4,
                    vector: VertexVector(Vector2ISize { x: 1, y: 5 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 8,
                    vector: VertexVector(Vector2ISize { x: -3, y: 1 })
                },
                stop: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 9,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 10,
                    vector: VertexVector(Vector2ISize { x: -3, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 14,
                    vector: VertexVector(Vector2ISize { x: 1, y: -5 })
                },
                stop: ArcEnd {
                    polar_index: 16,
                    vector: VertexVector(Vector2ISize { x: 2, y: -4 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_line_dir_1_0() {
    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let is_obstacle = |pos: AxialVector| -> bool {
        let vector = pos - AxialVector::direction(1) - center;
        let dir_0 = AxialVector::direction(0);
        dir_0.q() * vector.r() - dir_0.r() * vector.q() == 0
    };
    let mut fov = FieldOfView::default();
    fov.start(center);
    fov.next_radius(&is_obstacle);
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 12,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
    for i in 0..42 {
        fov.next_radius(&is_obstacle);
        assert_eq!(fov.radius, i + 3);
        assert_eq!(
            fov.arcs,
            vec![
                Arc {
                    start: ArcEnd {
                        polar_index: 0,
                        vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                    },
                    stop: ArcEnd {
                        polar_index: 1,
                        vector: VertexVector(Vector2ISize {
                            x: 2 * i as isize + 3,
                            y: 1
                        })
                    }
                },
                Arc {
                    start: ArcEnd {
                        polar_index: 3 * i + 8,
                        vector: VertexVector(Vector2ISize {
                            x: -(2 * i as isize + 3),
                            y: 1
                        })
                    },
                    stop: ArcEnd {
                        polar_index: 3 * i + 9,
                        vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                    }
                },
                Arc {
                    start: ArcEnd {
                        polar_index: 3 * i + 9,
                        vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                    },
                    stop: ArcEnd {
                        polar_index: 6 * i + 18,
                        vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                    }
                },
            ]
        );
    }
}
