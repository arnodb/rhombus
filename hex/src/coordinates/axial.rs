#[derive(Debug, PartialEq, Eq, Clone, Copy, Add)]
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
