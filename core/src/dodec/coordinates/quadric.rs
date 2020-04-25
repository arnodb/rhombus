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
