use std::ops::{Add, AddAssign, Mul, MulAssign};

pub mod axial;
pub mod cubic;
pub mod direction;
pub mod ring;

pub trait HexagonalVector:
    Sized + Clone + Copy + Add<Output = Self> + AddAssign + Mul<isize, Output = Self> + MulAssign<isize>
{
}
