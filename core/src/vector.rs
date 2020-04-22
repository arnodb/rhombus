use std::ops::Mul;
macro_rules! define_vector {
    ($name:ident, $($fields:ident),+) => {

        #[derive(Debug, PartialEq, Eq, Clone, Copy, Add, Sub)]
        pub struct $name<T> {
            $(pub $fields: T,)+
        }

        impl<T> Mul<T> for $name<T>
        where
            T: Mul<T, Output = T> + Copy,
        {
            type Output = Self;

            fn mul(self, rhs: T) -> Self::Output {
                Self {
                    $($fields: self.$fields * rhs,)+
                }
            }
        }

    };
}

define_vector!(Vector1, x);
define_vector!(Vector2, x, y);
define_vector!(Vector3, x, y, z);
define_vector!(Vector4, x, y, z, t);

pub type Vector1ISize = Vector1<isize>;
pub type Vector2ISize = Vector2<isize>;
pub type Vector3ISize = Vector3<isize>;
pub type Vector4ISize = Vector4<isize>;
