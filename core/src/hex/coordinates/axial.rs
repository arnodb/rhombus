use crate::{
    hex::coordinates::{
        cubic::CubicVector,
        direction::{HexagonalDirection, NUM_DIRECTIONS},
        ring::{BigRingIter, RingIter},
        HexagonalVector,
    },
    vector::Vector2ISize,
};
use std::ops::{Mul, MulAssign};

#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Debug,
)]
pub struct AxialVector(Vector2ISize);

impl AxialVector {
    pub fn new(q: isize, r: isize) -> Self {
        Self(Vector2ISize { x: q, y: r })
    }

    pub fn q(&self) -> isize {
        self.0.x
    }

    pub fn r(&self) -> isize {
        self.0.y
    }

    pub fn distance(self, other: Self) -> isize {
        CubicVector::from(self).distance(CubicVector::from(other))
    }

    pub fn ring_iter(&self, radius: usize) -> RingIter<Self> {
        RingIter::new(radius, *self)
    }

    pub fn big_ring_iter(&self, cell_radius: usize, radius: usize) -> BigRingIter<Self> {
        BigRingIter::new(cell_radius, radius, *self)
    }
}

impl Mul<isize> for AxialVector {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl MulAssign<isize> for AxialVector {
    fn mul_assign(&mut self, rhs: isize) {
        self.0 *= rhs
    }
}

impl Mul<AxialVector> for isize {
    type Output = AxialVector;

    fn mul(self, rhs: AxialVector) -> Self::Output {
        rhs * self
    }
}

impl HexagonalVector for AxialVector {}

// Don't use constructor and lazy_static so that the compiler can actually optimize the use
// of directions.
const DIRECTIONS: [AxialVector; NUM_DIRECTIONS] = [
    AxialVector(Vector2ISize { x: 1, y: 0 }),
    AxialVector(Vector2ISize { x: 1, y: -1 }),
    AxialVector(Vector2ISize { x: 0, y: -1 }),
    AxialVector(Vector2ISize { x: -1, y: 0 }),
    AxialVector(Vector2ISize { x: -1, y: 1 }),
    AxialVector(Vector2ISize { x: 0, y: 1 }),
];

impl HexagonalDirection for AxialVector {
    fn direction(direction: usize) -> Self {
        DIRECTIONS[direction]
    }
}

#[test]
fn test_new_axial_vector() {
    assert_eq!(
        AxialVector::new(1, -3),
        AxialVector(Vector2ISize { x: 1, y: -3 })
    )
}

#[test]
fn test_axial_vector_q() {
    assert_eq!(AxialVector::new(1, -3).q(), 1);
}

#[test]
fn test_axial_vector_r() {
    assert_eq!(AxialVector::new(1, -3).r(), -3);
}

#[test]
fn test_axial_vector_addition() {
    assert_eq!(
        AxialVector::new(1, -3) + AxialVector::new(-10, 30),
        AxialVector::new(-9, 27)
    );
}

#[test]
fn test_axial_vector_subtraction() {
    assert_eq!(
        AxialVector::new(1, -3) - AxialVector::new(-10, 30),
        AxialVector::new(11, -33)
    );
}

#[test]
fn test_axial_vector_distance() {
    let a = AxialVector::new(1, -3);
    let b = AxialVector::new(-2, 5);
    assert_eq!(a.distance(b), 8);
    assert_eq!(b.distance(a), 8);
}

#[test]
fn test_axial_directions_are_unique() {
    for dir1 in 0..NUM_DIRECTIONS - 1 {
        for dir2 in dir1 + 1..NUM_DIRECTIONS {
            assert_ne!(DIRECTIONS[dir1], DIRECTIONS[dir2])
        }
    }
}

#[test]
fn test_axial_directions_have_opposite() {
    for dir in 0..NUM_DIRECTIONS / 2 {
        assert_eq!(
            DIRECTIONS[dir] + DIRECTIONS[dir + NUM_DIRECTIONS / 2],
            AxialVector::default()
        );
    }
}

#[test]
fn test_axial_neighbor() {
    assert_eq!(AxialVector::new(-1, 1).neighbor(0), AxialVector::new(0, 1));
}

#[cfg(test)]
fn do_test_axial_ring_iter(radius: usize, expected: &Vec<AxialVector>) {
    let center = AxialVector::default();
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
fn test_axial_ring_iter0() {
    do_test_axial_ring_iter(0, &vec![AxialVector::default()]);
}

#[test]
fn test_axial_ring_iter1() {
    do_test_axial_ring_iter(
        1,
        &vec![
            AxialVector::new(-1, 1),
            AxialVector::new(0, 1),
            AxialVector::new(1, 0),
            AxialVector::new(1, -1),
            AxialVector::new(0, -1),
            AxialVector::new(-1, 0),
        ],
    );
}

#[test]
fn test_axial_ring_iter2() {
    do_test_axial_ring_iter(
        2,
        &vec![
            AxialVector::new(-2, 2),
            AxialVector::new(-1, 2),
            AxialVector::new(0, 2),
            AxialVector::new(1, 1),
            AxialVector::new(2, 0),
            AxialVector::new(2, -1),
            AxialVector::new(2, -2),
            AxialVector::new(1, -2),
            AxialVector::new(0, -2),
            AxialVector::new(-1, -1),
            AxialVector::new(-2, 0),
            AxialVector::new(-2, 1),
        ],
    );
}
