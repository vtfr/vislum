use std::ops::{Add, AddAssign, Mul, Sub};

macro_rules! impl_vector {
    ($ident:ident; $ty:ty; $constructor:ident; $inner:path; $($field:ident),*) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[repr(transparent)]
        pub struct $ident {
            inner: $inner,
        }

        #[doc = concat!("A ", stringify!($ident), " vector.")]
        impl $ident {
            pub const fn new($($field: $ty),*) -> Self {
                Self { inner: <$inner>::new($($field),*) }
            }

            $(
                pub fn $field(&self) -> $ty {
                    self.inner.$field
                }
            )*
        }

        pub const fn $constructor($($field: $ty),*) -> $ident {
            $ident::new($($field),*)
        }

        impl Default for $ident {
            fn default() -> Self {
                <$ident>::new(
                    $(impl_vector!(@zero_field $field)),*
                )
            }
        }

        impl Add for $ident {
            type Output = Self;
            
            fn add(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner + rhs.inner }
            }
        }

        impl AddAssign for $ident {
            fn add_assign(&mut self, rhs: Self) {
                self.inner += rhs.inner;
            }
        }

        impl Sub for $ident {
            type Output = Self;
            
            fn sub(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner - rhs.inner }
            }
        }

        impl Mul<$ty> for $ident {
            type Output = Self;

            fn mul(self, rhs: $ty) -> Self::Output {
                Self { inner: self.inner * rhs }
            }
        }
    };

    (@zero_field $field:ident) => { Default::default() };
}

impl_vector!(Vector4; f32; vec4; cgmath::Vector4<f32>; x, y, z, w);
impl_vector!(Vector3; f32; vec3; cgmath::Vector3<f32>; x, y, z);
impl_vector!(Vector2; f32; vec2; cgmath::Vector2<f32>; x, y);

impl_vector!(Vector2I; i32; vec2i; cgmath::Vector2<i32>; x, y);