use crate::coordinates::axial::AxialVector;
use derive_more::Add;
use std::ops::Mul;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Add, Sub)]
pub struct CubicVector {
    x: isize,
    y: isize,
    z: isize,
}

impl CubicVector {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        if x + y + z != 0 {
            panic!("Invalid CubicVector values x = {}, y = {}, z = {}", x, y, z);
        }
        Self { x, y, z }
    }

    pub fn direction(direction: usize) -> Self {
        DIRECTIONS[direction]
    }

    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }

    pub fn z(&self) -> isize {
        self.z
    }

    pub fn neighbor(&self, direction: usize) -> Self {
        *self + Self::direction(direction)
    }

    pub fn distance(self, other: Self) -> isize {
        let vector = self - other;
        (isize::abs(vector.x) + isize::abs(vector.y) + isize::abs(vector.z)) / 2
    }

    pub fn ring_iter(&self, radius: usize) -> RingIter {
        RingIter {
            next: Some(Self::direction(4) * radius as isize),
            direction: 0,
            edge_index: 0,
            radius,
        }
    }
}

impl Mul<isize> for CubicVector {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl From<AxialVector> for CubicVector {
    fn from(axial: AxialVector) -> Self {
        let x = axial.q();
        let z = axial.r();
        let y = -x - z;
        Self { x, y, z }
    }
}

impl From<CubicVector> for AxialVector {
    fn from(cubic: CubicVector) -> Self {
        Self::new(cubic.x(), cubic.z())
    }
}

// Don't use constructor and lazy_static so that the compiler can actually optimize the use
// of directions.
const DIRECTIONS: [CubicVector; 6] = [
    CubicVector { x: 1, y: -1, z: 0 },
    CubicVector { x: 1, y: 0, z: -1 },
    CubicVector { x: 0, y: 1, z: -1 },
    CubicVector { x: -1, y: 1, z: 0 },
    CubicVector { x: -1, y: 0, z: 1 },
    CubicVector { x: 0, y: -1, z: 1 },
];

pub struct RingIter {
    next: Option<CubicVector>,
    direction: usize,
    edge_index: usize,
    radius: usize,
}

impl RingIter {
    pub fn peek(&mut self) -> Option<&CubicVector> {
        self.next.as_ref()
    }
}

impl Iterator for RingIter {
    type Item = CubicVector;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next {
            if self.radius == 0 {
                self.next = None;
                return Some(current);
            }
            let direction = self.direction;
            let edge_index = self.edge_index;
            self.next = if direction < 6 {
                if edge_index + 1 >= self.radius {
                    self.edge_index = 0;
                    self.direction = direction + 1;
                    if direction + 1 < 6 {
                        Some(current.neighbor(direction))
                    } else {
                        None
                    }
                } else {
                    self.edge_index = edge_index + 1;
                    Some(current.neighbor(direction))
                }
            } else {
                None
            };
            Some(current)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let radius = self.radius;
        if radius > 0 {
            (radius * 6, Some(radius * 6))
        } else {
            (1, Some(1))
        }
    }
}

#[test]
fn test_new_cubic_vector() {
    assert_eq!(
        CubicVector::new(1, 2, -3),
        CubicVector { x: 1, y: 2, z: -3 }
    )
}

#[test]
#[should_panic]
fn test_new_invalid_cubic_vector() {
    CubicVector::new(1, 2, 42);
}

#[test]
fn test_cubic_vector_x() {
    assert_eq!(CubicVector::new(1, 2, -3).x(), 1);
}

#[test]
fn test_cubic_vector_y() {
    assert_eq!(CubicVector::new(1, 2, -3).y(), 2);
}

#[test]
fn test_cubic_vector_z() {
    assert_eq!(CubicVector::new(1, 2, -3).z(), -3);
}

#[test]
fn test_axial_to_cubic_vector() {
    assert_eq!(
        CubicVector::from(AxialVector::new(1, -3)),
        CubicVector::new(1, 2, -3)
    );
}

#[test]
fn test_cubic_to_axial_vector() {
    assert_eq!(
        AxialVector::from(CubicVector::new(1, 2, -3)),
        AxialVector::new(1, -3)
    );
}

#[test]
fn test_cubic_vector_addition() {
    assert_eq!(
        CubicVector::new(1, 2, -3) + CubicVector::new(-10, -20, 30),
        CubicVector::new(-9, -18, 27)
    );
}

#[test]
fn test_cubic_vector_subtraction() {
    assert_eq!(
        CubicVector::new(1, 2, -3) - CubicVector::new(-10, -20, 30),
        CubicVector::new(11, 22, -33)
    );
}

#[test]
fn test_cubic_vector_distance() {
    let a = CubicVector::new(1, 2, -3);
    let b = CubicVector::new(-2, -3, 5);
    assert_eq!(a.distance(b), 8);
    assert_eq!(b.distance(a), 8);
}

#[test]
fn test_directions_are_valid() {
    for v in DIRECTIONS.iter() {
        CubicVector::new(v.x(), v.y(), v.z());
    }
}

#[test]
fn test_all_directions_are_unique() {
    for dir1 in 0..5 {
        for dir2 in (dir1 + 1)..6 {
            assert_ne!(DIRECTIONS[dir1], DIRECTIONS[dir2])
        }
    }
}

#[test]
fn test_all_directions_have_opposite() {
    for dir in 0..3 {
        assert_eq!(
            DIRECTIONS[dir] + DIRECTIONS[dir + 3],
            CubicVector::new(0, 0, 0)
        );
    }
}

#[test]
fn test_neighbor() {
    assert_eq!(
        CubicVector::new(-1, 0, 1).neighbor(0),
        CubicVector::new(0, -1, 1)
    );
}

#[test]
fn test_ring_iter0() {
    let iter = CubicVector::new(0, 0, 0).ring_iter(0);
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.collect::<Vec<_>>(), vec![CubicVector::new(0, 0, 0)]);
}

#[test]
fn test_ring_iter1() {
    let iter = CubicVector::new(0, 0, 0).ring_iter(1);
    assert_eq!(iter.size_hint(), (6, Some(6)));
    assert_eq!(
        iter.collect::<Vec<_>>(),
        vec![
            CubicVector::new(-1, 0, 1),
            CubicVector::new(0, -1, 1),
            CubicVector::new(1, -1, 0),
            CubicVector::new(1, 0, -1),
            CubicVector::new(0, 1, -1),
            CubicVector::new(-1, 1, 0),
        ]
    );
}

#[test]
fn test_ring_iter2() {
    let iter = CubicVector::new(0, 0, 0).ring_iter(2);
    assert_eq!(iter.size_hint(), (12, Some(12)));
    assert_eq!(
        iter.collect::<Vec<_>>(),
        vec![
            CubicVector::new(-2, 0, 2),
            CubicVector::new(-1, -1, 2),
            CubicVector::new(0, -2, 2),
            CubicVector::new(1, -2, 1),
            CubicVector::new(2, -2, 0),
            CubicVector::new(2, -1, -1),
            CubicVector::new(2, 0, -2),
            CubicVector::new(1, 1, -2),
            CubicVector::new(0, 2, -2),
            CubicVector::new(-1, 2, -1),
            CubicVector::new(-2, 2, 0),
            CubicVector::new(-2, 1, 1),
        ]
    );
}
