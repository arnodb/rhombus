use crate::{
    hex::coordinates::{
        axial::AxialVector, cubic::CubicVector, direction::HexagonalDirection, HexagonalVector,
    },
    vector::Vector2ISize,
};
use std::fmt::Debug;

#[derive(Default, Debug)]
pub struct FieldOfView<V: HexagonalVector> {
    center: V,
    radius: usize,
    arcs: Vec<Arc>,
}

impl<V: HexagonalVector + HexagonalDirection + Into<VertexVector>> FieldOfView<V> {
    pub fn start<F>(&mut self, center: V, is_obstacle: &F)
    where
        F: Fn(V) -> bool,
    {
        self.center = center;
        self.radius = 1;
        self.arcs.clear();
        for arc in [
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 }),
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 }),
                },
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 }),
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 }),
                },
            },
        ]
        .iter()
        {
            self.arcs.extend(arc.clone().split(center, 1, is_obstacle));
        }
    }

    pub fn next_radius<F>(&mut self, is_obstacle: &F)
    where
        F: Fn(V) -> bool,
    {
        let new_radius = self.radius + 1;
        let mut split_arcs = Vec::new();
        for arc in &mut self.arcs {
            arc.expand::<V>(self.radius);
            split_arcs.extend(arc.clone().split(self.center, new_radius, is_obstacle));
        }
        self.arcs = split_arcs;
        self.radius = new_radius;
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
        return false;
    }

    fn is_left_of_arc<V: HexagonalDirection + Into<VertexVector>>(&self, radius: usize) -> bool {
        let vector = ArcEnd::polar_index_to_vector::<V>(self.polar_index, radius);
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = vector.into() + *local_vertex;
            if self.vector.turns(&vertex) == Turn::Left {
                return true;
            }
        }
        return false;
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
        if cross > 0 {
            Turn::Left
        } else if cross < 0 {
            Turn::Right
        } else {
            Turn::Straight
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

#[test]
fn test_field_of_view_1_0() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(0));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
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
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: 1, y: -1 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_1() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(1));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 2,
                    vector: VertexVector(Vector2ISize { x: 0, y: 4 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_2() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(2));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
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
                    vector: VertexVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_3() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(3));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 0,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 2,
                    vector: VertexVector(Vector2ISize { x: -1, y: 1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 4,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_4() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(4));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
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
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_5() {
    use std::collections::HashSet;

    let center =
        AxialVector::default() + AxialVector::direction(0) * 1 + AxialVector::direction(1) * 2;
    let obstacles = {
        let mut set = HashSet::new();
        set.insert(center + AxialVector::direction(5));
        set
    };
    let mut fov = FieldOfView::default();
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
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
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 4,
                    vector: VertexVector(Vector2ISize { x: 0, y: -2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 2, y: -2 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: 3, y: 0 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_024() {
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
    fov.start(center, &|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 1);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 1,
                    vector: VertexVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    vector: VertexVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 5,
                    vector: VertexVector(Vector2ISize { x: 1, y: -1 })
                }
            },
        ]
    );
}

#[test]
fn test_field_of_view_1_024_2_() {
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
    fov.start(center, &|pos| obstacles.contains(&pos));
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
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
fn test_field_of_view_1_024_2_1357911() {
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
    fov.start(center, &|pos| obstacles.contains(&pos));
    fov.next_radius(&|pos| obstacles.contains(&pos));
    assert_eq!(fov.center, center);
    assert_eq!(fov.radius, 2);
    assert_eq!(
        fov.arcs,
        vec![
            Arc {
                start: ArcEnd {
                    polar_index: 2,
                    vector: VertexVector(Vector2ISize { x: 2, y: 4 })
                },
                stop: ArcEnd {
                    polar_index: 2,
                    vector: VertexVector(Vector2ISize { x: 1, y: 5 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: 1 })
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
                    polar_index: 6,
                    vector: VertexVector(Vector2ISize { x: -3, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 10,
                    vector: VertexVector(Vector2ISize { x: 1, y: -5 })
                },
                stop: ArcEnd {
                    polar_index: 10,
                    vector: VertexVector(Vector2ISize { x: 2, y: -4 })
                }
            },
        ]
    );
}
