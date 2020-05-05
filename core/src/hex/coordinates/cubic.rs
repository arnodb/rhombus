use crate::{hex::coordinates::axial::AxialVector, vector::Vector3ISize};
use derive_more::Add;
use std::ops::{Mul, MulAssign};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Add, AddAssign, Sub, SubAssign)]
pub struct CubicVector(Vector3ISize);

impl CubicVector {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        if x + y + z != 0 {
            panic!("Invalid CubicVector values x = {}, y = {}, z = {}", x, y, z);
        }
        Self(Vector3ISize { x, y, z })
    }

    pub fn direction(direction: usize) -> Self {
        DIRECTIONS[direction]
    }

    pub fn x(&self) -> isize {
        self.0.x
    }

    pub fn y(&self) -> isize {
        self.0.y
    }

    pub fn z(&self) -> isize {
        self.0.z
    }

    pub fn distance(self, other: Self) -> isize {
        let vector = self - other;
        (isize::abs(vector.x()) + isize::abs(vector.y()) + isize::abs(vector.z())) / 2
    }

    pub fn neighbor(&self, direction: usize) -> Self {
        *self + Self::direction(direction)
    }

    pub fn ring_iter(&self, radius: usize) -> RingIter {
        RingIter::new(radius, *self)
    }
}

impl Mul<isize> for CubicVector {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl MulAssign<isize> for CubicVector {
    fn mul_assign(&mut self, rhs: isize) {
        self.0 *= rhs
    }
}

impl Mul<CubicVector> for isize {
    type Output = CubicVector;

    fn mul(self, rhs: CubicVector) -> Self::Output {
        rhs * self
    }
}

impl From<AxialVector> for CubicVector {
    fn from(axial: AxialVector) -> Self {
        let x = axial.q();
        let z = axial.r();
        let y = -x - z;
        Self(Vector3ISize { x, y, z })
    }
}

impl From<CubicVector> for AxialVector {
    fn from(cubic: CubicVector) -> Self {
        Self::new(cubic.x(), cubic.z())
    }
}

const NUM_DIRECTIONS: usize = 6;

// Don't use constructor and lazy_static so that the compiler can actually optimize the use
// of directions.
const DIRECTIONS: [CubicVector; NUM_DIRECTIONS] = [
    CubicVector(Vector3ISize { x: 1, y: -1, z: 0 }),
    CubicVector(Vector3ISize { x: 1, y: 0, z: -1 }),
    CubicVector(Vector3ISize { x: 0, y: 1, z: -1 }),
    CubicVector(Vector3ISize { x: -1, y: 1, z: 0 }),
    CubicVector(Vector3ISize { x: -1, y: 0, z: 1 }),
    CubicVector(Vector3ISize { x: 0, y: -1, z: 1 }),
];

pub struct RingIter {
    edge_length: usize,
    direction: usize,
    next: CubicVector,
    edge_index: usize,
}

impl RingIter {
    fn new(radius: usize, center: CubicVector) -> Self {
        Self {
            edge_length: radius,
            direction: 0,
            next: center + radius as isize * CubicVector::direction(4),
            edge_index: 1,
        }
    }

    pub fn peek(&mut self) -> Option<&CubicVector> {
        if self.direction < 6 {
            Some(&self.next)
        } else {
            None
        }
    }
}

impl Iterator for RingIter {
    type Item = CubicVector;

    fn next(&mut self) -> Option<Self::Item> {
        let edge_length = self.edge_length;
        let direction = self.direction;
        if direction < 6 {
            let next = self.next;
            self.next = next.neighbor(direction);
            let ei = self.edge_index;
            if ei < edge_length {
                self.edge_index = ei + 1;
            } else {
                self.edge_index = 1;
                self.direction = direction + 1;
                while self.direction < 6 && edge_length == 0 {
                    self.direction += 1;
                }
            }
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let el = self.edge_length;
        if el > 0 {
            let length = el * 6;
            (length, Some(length))
        } else {
            (1, Some(1))
        }
    }
}

#[test]
fn test_new_cubic_vector() {
    assert_eq!(
        CubicVector::new(1, 2, -3),
        CubicVector(Vector3ISize { x: 1, y: 2, z: -3 })
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
    for dir1 in 0..NUM_DIRECTIONS - 1 {
        for dir2 in dir1 + 1..NUM_DIRECTIONS {
            assert_ne!(DIRECTIONS[dir1], DIRECTIONS[dir2])
        }
    }
}

#[test]
fn test_all_directions_have_opposite() {
    for dir in 0..NUM_DIRECTIONS / 2 {
        assert_eq!(
            DIRECTIONS[dir] + DIRECTIONS[dir + NUM_DIRECTIONS / 2],
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

#[cfg(test)]
fn do_test_ring_iter(radius: usize, expected: &Vec<CubicVector>) {
    let center = CubicVector::new(0, 0, 0);
    let mut iter = center.ring_iter(radius);
    let mut peeked = iter.peek().cloned();
    assert!(peeked.is_some());
    let mut i = 0;
    loop {
        let next = iter.next();
        assert_eq!(next, peeked);
        peeked = iter.peek().cloned();
        if i < expected.len() {
            assert_eq!(next, Some(expected[i]));
            assert_eq!(expected[i].distance(center), radius as isize);
        } else {
            assert_eq!(next, None);
            break;
        }
        i += 1;
    }
    assert_eq!(peeked, None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.size_hint(), (expected.len(), Some(expected.len())));
}

#[test]
fn test_ring_iter0() {
    do_test_ring_iter(0, &vec![CubicVector::new(0, 0, 0)]);
}

#[test]
fn test_ring_iter1() {
    do_test_ring_iter(
        1,
        &vec![
            CubicVector::new(-1, 0, 1),
            CubicVector::new(0, -1, 1),
            CubicVector::new(1, -1, 0),
            CubicVector::new(1, 0, -1),
            CubicVector::new(0, 1, -1),
            CubicVector::new(-1, 1, 0),
        ],
    );
}

#[test]
fn test_ring_iter2() {
    do_test_ring_iter(
        2,
        &vec![
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
        ],
    );
}
