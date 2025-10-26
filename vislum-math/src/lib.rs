use std::ops::{Add, AddAssign, Mul, Sub};

use cgmath::SquareMatrix;

macro_rules! impl_vector {
    ($ident:ident; $components:expr; $ty:ty; $constructor:ident; $inner:path; $($field:ident),*) => {
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

        impl Into<[$ty; $components]> for $ident {
            fn into(self) -> [$ty; $components] {
                self.inner.into()
            }
        }

        impl Into<$inner> for $ident {
            fn into(self) -> $inner {
                self.inner
            }
        }

        impl From<[$ty; $components]> for $ident {
            fn from(value: [$ty; $components]) -> Self {
                Self { inner: <$inner>::from(value) }
            }
        }
    };

    (@zero_field $field:ident) => { Default::default() };
}

impl_vector!(Vector4; 4; f32; vec4; cgmath::Vector4<f32>; x, y, z, w);
impl_vector!(Vector3; 3; f32; vec3; cgmath::Vector3<f32>; x, y, z);
impl_vector!(Vector2; 2; f32; vec2; cgmath::Vector2<f32>; x, y);

impl_vector!(Vector2I; 2; i32; vec2i; cgmath::Vector2<i32>; x, y);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    inner: cgmath::Matrix4<f32>,
}

impl Matrix4 {
    pub fn new(col1: Vector4, col2: Vector4, col3: Vector4, col4: Vector4) -> Self {
        Self {
            inner: cgmath::Matrix4::from_cols(col1.into(), col2.into(), col3.into(), col4.into()),
        }
    }
}
impl std::ops::Add for Matrix4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner + rhs.inner,
        }
    }
}

impl std::ops::AddAssign for Matrix4 {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl std::ops::Sub for Matrix4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner - rhs.inner,
        }
    }
}

impl std::ops::SubAssign for Matrix4 {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner;
    }
}

impl std::ops::Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner * rhs.inner,
        }
    }
}

impl std::ops::Mul<Vector4> for Matrix4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Self::Output {
        Vector4 {
            inner: self.inner * rhs.inner,
        }
    }
}

impl std::ops::Mul<f32> for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            inner: self.inner * rhs,
        }
    }
}

impl std::ops::MulAssign<f32> for Matrix4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.inner *= rhs;
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self {
            inner: cgmath::Matrix4::identity(),
        }
    }
}
