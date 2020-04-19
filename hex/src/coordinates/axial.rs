use crate::coordinates::cubic::CubicVector;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Add, Sub)]
pub struct AxialVector {
    q: isize,
    r: isize,
}

impl AxialVector {
    pub fn new(q: isize, r: isize) -> Self {
        Self { q, r }
    }

    pub fn q(&self) -> isize {
        self.q
    }

    pub fn r(&self) -> isize {
        self.r
    }

    pub fn distance(self, other: Self) -> isize {
        CubicVector::from(self).distance(CubicVector::from(other))
    }
}

#[test]
fn test_new_axial_vector() {
    assert_eq!(AxialVector::new(1, -3), AxialVector { q: 1, r: -3 })
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
