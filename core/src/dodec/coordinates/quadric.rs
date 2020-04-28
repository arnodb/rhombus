use crate::vector::Vector4ISize;
use derive_more::Add;
use std::ops::Mul;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Add, AddAssign, Sub, SubAssign)]
pub struct QuadricVector(Vector4ISize);

impl QuadricVector {
    pub fn new(x: isize, y: isize, z: isize, t: isize) -> Self {
        if x + y + z + t != 0 {
            panic!(
                "Invalid QuadricVector values x = {}, y = {}, z = {}, t = {}",
                x, y, z, t
            );
        }
        Self(Vector4ISize { x, y, z, t })
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

    pub fn t(&self) -> isize {
        self.0.t
    }

    pub fn distance(self, other: Self) -> isize {
        let vector = self - other;
        (isize::abs(vector.x())
            + isize::abs(vector.y())
            + isize::abs(vector.z())
            + isize::abs(vector.t()))
            / 2
    }

    pub fn neighbor(&self, direction: usize) -> Self {
        *self + Self::direction(direction)
    }

    pub fn sphere_iter(&self, radius: usize) -> SphereIter {
        SphereIter::new(radius, *self)
    }
}

impl Mul<isize> for QuadricVector {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<QuadricVector> for isize {
    type Output = QuadricVector;

    fn mul(self, rhs: QuadricVector) -> Self::Output {
        rhs * self
    }
}

const NUM_DIRECTIONS: usize = 12;

// Don't use constructor and lazy_static so that the compiler can actually optimize the use
// of directions.
const DIRECTIONS: [QuadricVector; NUM_DIRECTIONS] = [
    QuadricVector(Vector4ISize {
        x: 1,
        y: -1,
        z: 0,
        t: 0,
    }),
    QuadricVector(Vector4ISize {
        x: 1,
        y: 0,
        z: -1,
        t: 0,
    }),
    QuadricVector(Vector4ISize {
        x: 0,
        y: 1,
        z: -1,
        t: 0,
    }),
    QuadricVector(Vector4ISize {
        x: 1,
        y: 0,
        z: 0,
        t: -1,
    }),
    QuadricVector(Vector4ISize {
        x: 0,
        y: 1,
        z: 0,
        t: -1,
    }),
    QuadricVector(Vector4ISize {
        x: 0,
        y: 0,
        z: 1,
        t: -1,
    }),
    QuadricVector(Vector4ISize {
        x: -1,
        y: 1,
        z: 0,
        t: 0,
    }),
    QuadricVector(Vector4ISize {
        x: -1,
        y: 0,
        z: 1,
        t: 0,
    }),
    QuadricVector(Vector4ISize {
        x: 0,
        y: -1,
        z: 1,
        t: 0,
    }),
    QuadricVector(Vector4ISize {
        x: -1,
        y: 0,
        z: 0,
        t: 1,
    }),
    QuadricVector(Vector4ISize {
        x: 0,
        y: -1,
        z: 0,
        t: 1,
    }),
    QuadricVector(Vector4ISize {
        x: 0,
        y: 0,
        z: -1,
        t: 1,
    }),
];

struct SphereRingIter {
    edge_lengths: [usize; 2],
    direction: usize,
    next: QuadricVector,
    edge_index: usize,
}

impl SphereRingIter {
    fn new(edge_lengths: [usize; 2], next: QuadricVector) -> Self {
        let mut direction = 0;
        // Drain all but last edge so that:
        //     - the state is ready for next iteration
        //     - the ring of size 0 case is handled correctly (it returns
        //       the first value, and no more then)
        while direction < 5 && edge_lengths[direction & 1] == 0 {
            direction += 1;
        }
        Self {
            edge_lengths,
            direction,
            next,
            edge_index: 1,
        }
    }

    pub fn peek(&mut self) -> Option<&QuadricVector> {
        if self.direction < 6 {
            Some(&self.next)
        } else {
            None
        }
    }
}

const SPHERE_RING_ITER_DIRECTIONS: [usize; 6] = [0, 1, 2, 6, 7, 8];

impl Iterator for SphereRingIter {
    type Item = QuadricVector;

    fn next(&mut self) -> Option<Self::Item> {
        let edge_lengths = self.edge_lengths;
        let direction = self.direction;
        if direction < 6 {
            let next = self.next;
            self.next = next.neighbor(SPHERE_RING_ITER_DIRECTIONS[direction]);
            let ei = self.edge_index;
            if ei < edge_lengths[direction & 1] {
                self.edge_index = ei + 1;
            } else {
                self.edge_index = 1;
                self.direction = direction + 1;
                while self.direction < 6 && edge_lengths[self.direction & 1] == 0 {
                    self.direction += 1;
                }
            }
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let [el1, el2] = self.edge_lengths;
        if el1 > 0 || el2 > 0 {
            let length = 3 * (el1 + el2);
            (length, Some(length))
        } else {
            (1, Some(1))
        }
    }
}

pub struct SphereIter {
    radius: usize,
    depth: usize,
    max_depth: usize,
    iter: SphereRingIter,
}

impl SphereIter {
    fn new(radius: usize, center: QuadricVector) -> Self {
        Self {
            radius,
            depth: 0,
            max_depth: 2 * (radius + (radius / 3)) + 1,
            iter: SphereRingIter::new(
                [radius % 3, 0],
                center
                    + (radius as isize / 3)
                        * (QuadricVector::direction(0) + QuadricVector::direction(1))
                    - radius as isize * QuadricVector::direction(3),
            ),
        }
    }

    pub fn peek(&mut self) -> Option<&QuadricVector> {
        self.iter.peek()
    }
}

impl Iterator for SphereIter {
    type Item = QuadricVector;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.iter.next();
        if res.is_some() && self.iter.peek().is_none() {
            let depth = self.depth;
            self.depth = depth + 1;
            if depth < self.max_depth {
                let [el1, el2] = self.iter.edge_lengths;
                let (edge_lengths, next) = if depth < self.radius / 3 {
                    (
                        [el1 + 3, 0],
                        self.iter.next + QuadricVector::direction(6) + QuadricVector::direction(7),
                    )
                } else if el1 == self.radius && el2 < self.radius {
                    ([el1, el2 + 1], self.iter.next + QuadricVector::direction(5))
                } else if el1 > 0 {
                    ([el1 - 1, el2], self.iter.next + QuadricVector::direction(3))
                } else if el2 > self.radius % 3 {
                    (
                        [0, el2 - 3],
                        self.iter.next + QuadricVector::direction(1) + QuadricVector::direction(2),
                    )
                } else {
                    return res;
                };
                self.iter = SphereRingIter::new(edge_lengths, next);
            }
        }
        res
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let radius = self.radius;
        if radius > 0 {
            // Triangles minus shared vertices
            let mut exact = 4 * (1 + radius) * (2 + radius) - 12;
            if radius > 1 {
                // Squares interior
                exact += 6 * (radius - 1) * (radius - 1);
            }
            (exact, Some(exact))
        } else {
            (1, Some(1))
        }
    }
}

#[test]
fn test_new_quadric_vector() {
    assert_eq!(
        QuadricVector::new(1, 2, -7, 4),
        QuadricVector(Vector4ISize {
            x: 1,
            y: 2,
            z: -7,
            t: 4
        })
    )
}

#[test]
#[should_panic]
fn test_new_invalid_quadric_vector() {
    QuadricVector::new(1, 2, -7, 42);
}

#[test]
fn test_quadric_vector_x() {
    assert_eq!(QuadricVector::new(1, 2, -7, 4).x(), 1);
}

#[test]
fn test_quadric_vector_y() {
    assert_eq!(QuadricVector::new(1, 2, -7, 4).y(), 2);
}

#[test]
fn test_quadric_vector_z() {
    assert_eq!(QuadricVector::new(1, 2, -7, 4).z(), -7);
}

#[test]
fn test_quadric_vector_t() {
    assert_eq!(QuadricVector::new(1, 2, -7, 4).t(), 4);
}

#[test]
fn test_quadric_vector_addition() {
    assert_eq!(
        QuadricVector::new(1, 2, -7, 4) + QuadricVector::new(-10, -20, 70, -40),
        QuadricVector::new(-9, -18, 63, -36)
    );
}

#[test]
fn test_quadric_vector_subtraction() {
    assert_eq!(
        QuadricVector::new(1, 2, -7, 4) - QuadricVector::new(-10, -20, 70, -40),
        QuadricVector::new(11, 22, -77, 44)
    );
}

#[test]
fn test_quadric_vector_distance() {
    let a = QuadricVector::new(1, 2, -7, 4);
    let b = QuadricVector::new(-2, -3, 7, -2);
    assert_eq!(a.distance(b), 14);
    assert_eq!(b.distance(a), 14);
}

#[test]
fn test_directions_are_valid() {
    for v in DIRECTIONS.iter() {
        QuadricVector::new(v.x(), v.y(), v.z(), v.t());
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
            QuadricVector::new(0, 0, 0, 0)
        );
    }
}

#[test]
fn test_neighbor() {
    assert_eq!(
        QuadricVector::new(-1, 0, 1, 0).neighbor(0),
        QuadricVector::new(0, -1, 1, 0)
    );
}

#[cfg(test)]
fn do_test_sphere_iter(radius: usize, expected: &Vec<QuadricVector>) {
    let center = QuadricVector::new(0, 0, 0, 0);
    let mut iter = center.sphere_iter(radius);
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
fn test_sphere_iter0() {
    do_test_sphere_iter(0, &vec![QuadricVector::new(0, 0, 0, 0)]);
}

#[test]
fn test_sphere_iter1() {
    do_test_sphere_iter(
        1,
        &vec![
            QuadricVector::new(-1, 0, 0, 1),
            QuadricVector::new(0, -1, 0, 1),
            QuadricVector::new(0, 0, -1, 1),
            QuadricVector::new(-1, 0, 1, 0),
            QuadricVector::new(0, -1, 1, 0),
            QuadricVector::new(1, -1, 0, 0),
            QuadricVector::new(1, 0, -1, 0),
            QuadricVector::new(0, 1, -1, 0),
            QuadricVector::new(-1, 1, 0, 0),
            QuadricVector::new(0, 0, 1, -1),
            QuadricVector::new(1, 0, 0, -1),
            QuadricVector::new(0, 1, 0, -1),
        ],
    );
}

#[test]
fn test_sphere_iter2() {
    do_test_sphere_iter(
        2,
        &vec![
            QuadricVector::new(-2, 0, 0, 2),
            QuadricVector::new(-1, -1, 0, 2),
            QuadricVector::new(0, -2, 0, 2),
            QuadricVector::new(0, -1, -1, 2),
            QuadricVector::new(0, 0, -2, 2),
            QuadricVector::new(-1, 0, -1, 2),
            QuadricVector::new(-2, 0, 1, 1),
            QuadricVector::new(-1, -1, 1, 1),
            QuadricVector::new(0, -2, 1, 1),
            QuadricVector::new(1, -2, 0, 1),
            QuadricVector::new(1, -1, -1, 1),
            QuadricVector::new(1, 0, -2, 1),
            QuadricVector::new(0, 1, -2, 1),
            QuadricVector::new(-1, 1, -1, 1),
            QuadricVector::new(-2, 1, 0, 1),
            QuadricVector::new(-2, 0, 2, 0),
            QuadricVector::new(-1, -1, 2, 0),
            QuadricVector::new(0, -2, 2, 0),
            QuadricVector::new(1, -2, 1, 0),
            QuadricVector::new(2, -2, 0, 0),
            QuadricVector::new(2, -1, -1, 0),
            QuadricVector::new(2, 0, -2, 0),
            QuadricVector::new(1, 1, -2, 0),
            QuadricVector::new(0, 2, -2, 0),
            QuadricVector::new(-1, 2, -1, 0),
            QuadricVector::new(-2, 2, 0, 0),
            QuadricVector::new(-2, 1, 1, 0),
            QuadricVector::new(-1, 0, 2, -1),
            QuadricVector::new(0, -1, 2, -1),
            QuadricVector::new(1, -1, 1, -1),
            QuadricVector::new(2, -1, 0, -1),
            QuadricVector::new(2, 0, -1, -1),
            QuadricVector::new(1, 1, -1, -1),
            QuadricVector::new(0, 2, -1, -1),
            QuadricVector::new(-1, 2, 0, -1),
            QuadricVector::new(-1, 1, 1, -1),
            QuadricVector::new(0, 0, 2, -2),
            QuadricVector::new(1, 0, 1, -2),
            QuadricVector::new(2, 0, 0, -2),
            QuadricVector::new(1, 1, 0, -2),
            QuadricVector::new(0, 2, 0, -2),
            QuadricVector::new(0, 1, 1, -2),
        ],
    );
}

#[test]
fn test_sphere_iter4() {
    println!(
        "{:?}",
        QuadricVector::new(0, 0, 0, 0)
            .sphere_iter(4)
            .collect::<Vec<_>>()
    );
    do_test_sphere_iter(
        4,
        &vec![
            QuadricVector::new(-2, -1, -1, 4),
            QuadricVector::new(-1, -2, -1, 4),
            QuadricVector::new(-1, -1, -2, 4),
            QuadricVector::new(-4, 0, 0, 4),
            QuadricVector::new(-3, -1, 0, 4),
            QuadricVector::new(-2, -2, 0, 4),
            QuadricVector::new(-1, -3, 0, 4),
            QuadricVector::new(0, -4, 0, 4),
            QuadricVector::new(0, -3, -1, 4),
            QuadricVector::new(0, -2, -2, 4),
            QuadricVector::new(0, -1, -3, 4),
            QuadricVector::new(0, 0, -4, 4),
            QuadricVector::new(-1, 0, -3, 4),
            QuadricVector::new(-2, 0, -2, 4),
            QuadricVector::new(-3, 0, -1, 4),
            QuadricVector::new(-4, 0, 1, 3),
            QuadricVector::new(-3, -1, 1, 3),
            QuadricVector::new(-2, -2, 1, 3),
            QuadricVector::new(-1, -3, 1, 3),
            QuadricVector::new(0, -4, 1, 3),
            QuadricVector::new(1, -4, 0, 3),
            QuadricVector::new(1, -3, -1, 3),
            QuadricVector::new(1, -2, -2, 3),
            QuadricVector::new(1, -1, -3, 3),
            QuadricVector::new(1, 0, -4, 3),
            QuadricVector::new(0, 1, -4, 3),
            QuadricVector::new(-1, 1, -3, 3),
            QuadricVector::new(-2, 1, -2, 3),
            QuadricVector::new(-3, 1, -1, 3),
            QuadricVector::new(-4, 1, 0, 3),
            QuadricVector::new(-4, 0, 2, 2),
            QuadricVector::new(-3, -1, 2, 2),
            QuadricVector::new(-2, -2, 2, 2),
            QuadricVector::new(-1, -3, 2, 2),
            QuadricVector::new(0, -4, 2, 2),
            QuadricVector::new(1, -4, 1, 2),
            QuadricVector::new(2, -4, 0, 2),
            QuadricVector::new(2, -3, -1, 2),
            QuadricVector::new(2, -2, -2, 2),
            QuadricVector::new(2, -1, -3, 2),
            QuadricVector::new(2, 0, -4, 2),
            QuadricVector::new(1, 1, -4, 2),
            QuadricVector::new(0, 2, -4, 2),
            QuadricVector::new(-1, 2, -3, 2),
            QuadricVector::new(-2, 2, -2, 2),
            QuadricVector::new(-3, 2, -1, 2),
            QuadricVector::new(-4, 2, 0, 2),
            QuadricVector::new(-4, 1, 1, 2),
            QuadricVector::new(-4, 0, 3, 1),
            QuadricVector::new(-3, -1, 3, 1),
            QuadricVector::new(-2, -2, 3, 1),
            QuadricVector::new(-1, -3, 3, 1),
            QuadricVector::new(0, -4, 3, 1),
            QuadricVector::new(1, -4, 2, 1),
            QuadricVector::new(2, -4, 1, 1),
            QuadricVector::new(3, -4, 0, 1),
            QuadricVector::new(3, -3, -1, 1),
            QuadricVector::new(3, -2, -2, 1),
            QuadricVector::new(3, -1, -3, 1),
            QuadricVector::new(3, 0, -4, 1),
            QuadricVector::new(2, 1, -4, 1),
            QuadricVector::new(1, 2, -4, 1),
            QuadricVector::new(0, 3, -4, 1),
            QuadricVector::new(-1, 3, -3, 1),
            QuadricVector::new(-2, 3, -2, 1),
            QuadricVector::new(-3, 3, -1, 1),
            QuadricVector::new(-4, 3, 0, 1),
            QuadricVector::new(-4, 2, 1, 1),
            QuadricVector::new(-4, 1, 2, 1),
            QuadricVector::new(-4, 0, 4, 0),
            QuadricVector::new(-3, -1, 4, 0),
            QuadricVector::new(-2, -2, 4, 0),
            QuadricVector::new(-1, -3, 4, 0),
            QuadricVector::new(0, -4, 4, 0),
            QuadricVector::new(1, -4, 3, 0),
            QuadricVector::new(2, -4, 2, 0),
            QuadricVector::new(3, -4, 1, 0),
            QuadricVector::new(4, -4, 0, 0),
            QuadricVector::new(4, -3, -1, 0),
            QuadricVector::new(4, -2, -2, 0),
            QuadricVector::new(4, -1, -3, 0),
            QuadricVector::new(4, 0, -4, 0),
            QuadricVector::new(3, 1, -4, 0),
            QuadricVector::new(2, 2, -4, 0),
            QuadricVector::new(1, 3, -4, 0),
            QuadricVector::new(0, 4, -4, 0),
            QuadricVector::new(-1, 4, -3, 0),
            QuadricVector::new(-2, 4, -2, 0),
            QuadricVector::new(-3, 4, -1, 0),
            QuadricVector::new(-4, 4, 0, 0),
            QuadricVector::new(-4, 3, 1, 0),
            QuadricVector::new(-4, 2, 2, 0),
            QuadricVector::new(-4, 1, 3, 0),
            QuadricVector::new(-3, 0, 4, -1),
            QuadricVector::new(-2, -1, 4, -1),
            QuadricVector::new(-1, -2, 4, -1),
            QuadricVector::new(0, -3, 4, -1),
            QuadricVector::new(1, -3, 3, -1),
            QuadricVector::new(2, -3, 2, -1),
            QuadricVector::new(3, -3, 1, -1),
            QuadricVector::new(4, -3, 0, -1),
            QuadricVector::new(4, -2, -1, -1),
            QuadricVector::new(4, -1, -2, -1),
            QuadricVector::new(4, 0, -3, -1),
            QuadricVector::new(3, 1, -3, -1),
            QuadricVector::new(2, 2, -3, -1),
            QuadricVector::new(1, 3, -3, -1),
            QuadricVector::new(0, 4, -3, -1),
            QuadricVector::new(-1, 4, -2, -1),
            QuadricVector::new(-2, 4, -1, -1),
            QuadricVector::new(-3, 4, 0, -1),
            QuadricVector::new(-3, 3, 1, -1),
            QuadricVector::new(-3, 2, 2, -1),
            QuadricVector::new(-3, 1, 3, -1),
            QuadricVector::new(-2, 0, 4, -2),
            QuadricVector::new(-1, -1, 4, -2),
            QuadricVector::new(0, -2, 4, -2),
            QuadricVector::new(1, -2, 3, -2),
            QuadricVector::new(2, -2, 2, -2),
            QuadricVector::new(3, -2, 1, -2),
            QuadricVector::new(4, -2, 0, -2),
            QuadricVector::new(4, -1, -1, -2),
            QuadricVector::new(4, 0, -2, -2),
            QuadricVector::new(3, 1, -2, -2),
            QuadricVector::new(2, 2, -2, -2),
            QuadricVector::new(1, 3, -2, -2),
            QuadricVector::new(0, 4, -2, -2),
            QuadricVector::new(-1, 4, -1, -2),
            QuadricVector::new(-2, 4, 0, -2),
            QuadricVector::new(-2, 3, 1, -2),
            QuadricVector::new(-2, 2, 2, -2),
            QuadricVector::new(-2, 1, 3, -2),
            QuadricVector::new(-1, 0, 4, -3),
            QuadricVector::new(0, -1, 4, -3),
            QuadricVector::new(1, -1, 3, -3),
            QuadricVector::new(2, -1, 2, -3),
            QuadricVector::new(3, -1, 1, -3),
            QuadricVector::new(4, -1, 0, -3),
            QuadricVector::new(4, 0, -1, -3),
            QuadricVector::new(3, 1, -1, -3),
            QuadricVector::new(2, 2, -1, -3),
            QuadricVector::new(1, 3, -1, -3),
            QuadricVector::new(0, 4, -1, -3),
            QuadricVector::new(-1, 4, 0, -3),
            QuadricVector::new(-1, 3, 1, -3),
            QuadricVector::new(-1, 2, 2, -3),
            QuadricVector::new(-1, 1, 3, -3),
            QuadricVector::new(0, 0, 4, -4),
            QuadricVector::new(1, 0, 3, -4),
            QuadricVector::new(2, 0, 2, -4),
            QuadricVector::new(3, 0, 1, -4),
            QuadricVector::new(4, 0, 0, -4),
            QuadricVector::new(3, 1, 0, -4),
            QuadricVector::new(2, 2, 0, -4),
            QuadricVector::new(1, 3, 0, -4),
            QuadricVector::new(0, 4, 0, -4),
            QuadricVector::new(0, 3, 1, -4),
            QuadricVector::new(0, 2, 2, -4),
            QuadricVector::new(0, 1, 3, -4),
            QuadricVector::new(1, 1, 2, -4),
            QuadricVector::new(2, 1, 1, -4),
            QuadricVector::new(1, 2, 1, -4),
        ],
    );
}
