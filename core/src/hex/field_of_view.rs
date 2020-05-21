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

impl<V: HexagonalVector + HexagonalDirection + Into<PlaneVector>> FieldOfView<V> {
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
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 }),
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 }),
                },
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 }),
                },
                stop: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 }),
                },
            },
        ]
        .iter()
        {
            self.arcs.extend(arc.clone().split(center, 1, is_obstacle));
        }
    }

    fn _next_radius<F>(&mut self, is_obstacle: &F)
    where
        F: Fn(V) -> bool,
    {
        let new_radius = self.radius + 1;
        let mut split_arcs = Vec::new();
        for arc in &self.arcs {
            // TODO expand arc
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
        match self.start.position.turns(&self.stop.position) {
            Turn::Right => true,
            // The good thing with PlaneVector holding integral values is that there is no need
            // to introduce an epsilon here. However positions could be opposite because
            // the system is pushed to PI angles inclusive. This probably would not be necessary
            // if the field of view was split into 3 slices or 4 quadrants like in the reference
            // implementation.
            // It is difficult to tell which implementation is more optimal CPU-wise without
            // proper measurement.
            Turn::Straight
                if self.start.position.0.x < 0 && self.stop.position.0.x < 0
                    || self.start.position.0.x > 0 && self.stop.position.0.x > 0
                    || self.start.position.0.y < 0 && self.stop.position.0.y < 0
                    || self.start.position.0.y > 0 && self.stop.position.0.y > 0 =>
            {
                true
            }
            Turn::Left | Turn::Straight => false,
        }
    }

    fn split<V: HexagonalDirection + Into<PlaneVector>, F>(
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
                let pos = polar_index_to_position(self.start.polar_index, radius);
                if is_obstacle(center + pos) {
                    self.start.contract_start(pos);
                    self.start.polar_index += 1;
                } else {
                    break;
                }
            }
            // Find stop obstacle
            let mut polar_index = self.start.polar_index;
            while polar_index <= self.stop.polar_index {
                let pos = polar_index_to_position(polar_index, radius);
                if is_obstacle(center + pos) {
                    let mut arc = self.clone();
                    // Contract stop
                    arc.stop.contract_stop(pos);
                    arc.stop.polar_index = polar_index - 1;
                    // Push if non zero angle
                    if !arc.is_zero_angle() {
                        split.push(arc);
                    }
                    // Reset start for next iteration
                    self.start.contract_start(pos);
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
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct ArcEnd {
    polar_index: usize,
    position: PlaneVector,
}

impl ArcEnd {
    fn contract_start<V: HexagonalVector + Into<PlaneVector>>(&mut self, pos: V) {
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = pos.into() + *local_vertex;
            if self.position.turns(&vertex) == Turn::Left {
                self.position = vertex;
            }
        }
    }

    fn contract_stop<V: HexagonalVector + Into<PlaneVector>>(&mut self, pos: V) {
        for local_vertex in HEX_PLANE_VERTICES.iter() {
            let vertex = pos.into() + *local_vertex;
            if self.position.turns(&vertex) == Turn::Right {
                self.position = vertex;
            }
        }
    }
}

fn polar_index_to_position<V: HexagonalDirection>(polar_index: usize, radius: usize) -> V {
    let side = (polar_index / radius) % 6;
    let side_offset = polar_index % radius;
    V::direction(side) * radius as isize + V::direction((side + 2) % 6) * side_offset as isize
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Turn {
    Left,
    Straight,
    Right,
}

#[derive(PartialEq, Eq, Clone, Copy, Add, AddAssign, Sub, SubAssign, Debug)]
pub struct PlaneVector(Vector2ISize);

impl PlaneVector {
    fn turns(&self, other: &PlaneVector) -> Turn {
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

const HEX_PLANE_VERTICES: [PlaneVector; 6] = [
    PlaneVector(Vector2ISize { x: 1, y: -1 }),
    PlaneVector(Vector2ISize { x: 1, y: 1 }),
    PlaneVector(Vector2ISize { x: 0, y: 2 }),
    PlaneVector(Vector2ISize { x: -1, y: 1 }),
    PlaneVector(Vector2ISize { x: -1, y: -1 }),
    PlaneVector(Vector2ISize { x: 0, y: -2 }),
];

impl From<AxialVector> for PlaneVector {
    fn from(axial: AxialVector) -> Self {
        PlaneVector(Vector2ISize {
            x: 2 * axial.q() + axial.r(),
            y: -3 * axial.r(),
        })
    }
}

impl From<CubicVector> for PlaneVector {
    fn from(cubic: CubicVector) -> Self {
        PlaneVector(Vector2ISize {
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
                    position: PlaneVector(Vector2ISize { x: 2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 5,
                    position: PlaneVector(Vector2ISize { x: 1, y: -1 })
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
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 0,
                    position: PlaneVector(Vector2ISize { x: 2, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 2,
                    position: PlaneVector(Vector2ISize { x: 0, y: 4 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
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
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 1,
                    position: PlaneVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
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
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 2,
                    position: PlaneVector(Vector2ISize { x: -1, y: 1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 4,
                    position: PlaneVector(Vector2ISize { x: -1, y: -1 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
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
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    position: PlaneVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
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
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 4,
                    position: PlaneVector(Vector2ISize { x: 0, y: -2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 2, y: -2 })
                },
                stop: ArcEnd {
                    polar_index: 6,
                    position: PlaneVector(Vector2ISize { x: 3, y: 0 })
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
                    position: PlaneVector(Vector2ISize { x: 2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 1,
                    position: PlaneVector(Vector2ISize { x: 0, y: 2 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -2, y: 2 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -3, y: 0 })
                },
                stop: ArcEnd {
                    polar_index: 3,
                    position: PlaneVector(Vector2ISize { x: -1, y: -1 })
                }
            },
            Arc {
                start: ArcEnd {
                    polar_index: 5,
                    position: PlaneVector(Vector2ISize { x: 0, y: -4 })
                },
                stop: ArcEnd {
                    polar_index: 5,
                    position: PlaneVector(Vector2ISize { x: 1, y: -1 })
                }
            },
        ]
    );
}
