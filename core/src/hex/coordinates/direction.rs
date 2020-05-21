use crate::hex::coordinates::HexagonalVector;

pub const NUM_DIRECTIONS: usize = 6;

pub trait HexagonalDirection: HexagonalVector {
    fn direction(direction: usize) -> Self;

    fn neighbor(&self, direction: usize) -> Self {
        *self + Self::direction(direction)
    }
}
