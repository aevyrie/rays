use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

use glam::{Vec4, Vec4Swizzles};

#[derive(Clone, Debug)]
pub struct Color {
    pub(crate) inner: Vec4,
}
impl Color {
    #[inline(always)]
    pub fn approx_luminance(&self) -> f32 {
        let [r, g, b] = self.inner.xyz().to_array();
        (r + r + b + g + g + g) / 6.0
    }
    #[inline(always)]
    pub fn r(&self) -> f32 {
        self.inner.x
    }
    #[inline(always)]
    pub fn g(&self) -> f32 {
        self.inner.y
    }
    #[inline(always)]
    pub fn b(&self) -> f32 {
        self.inner.z
    }
    #[inline(always)]
    pub fn a(&self) -> f32 {
        self.inner.w
    }
    #[inline(always)]
    pub fn into_bytes(&self) -> [u8; 4] {
        [
            (self.r().min(1.0) * 255.0) as u8,
            (self.g().min(1.0) * 255.0) as u8,
            (self.b().min(1.0) * 255.0) as u8,
            (self.a().min(1.0) * 255.0) as u8,
        ]
    }
}
impl Mul for Color {
    type Output = Color;

    #[inline(always)]
    fn mul(mut self, rhs: Self) -> Self::Output {
        self.inner *= rhs.inner;
        self
    }
}
impl Mul<&Color> for Color {
    type Output = Color;

    #[inline(always)]
    fn mul(mut self, rhs: &Self) -> Self::Output {
        self.inner *= rhs.inner;
        self
    }
}
impl Mul<Color> for &Color {
    type Output = Color;

    #[inline(always)]
    fn mul(self, mut rhs: Color) -> Self::Output {
        rhs.inner = self.inner * rhs.inner;
        rhs
    }
}

impl MulAssign for Color {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.inner = self.inner * rhs.inner;
    }
}
impl Add for Color {
    type Output = Color;

    #[inline(always)]
    fn add(mut self, rhs: Self) -> Self::Output {
        self.inner += rhs.inner;
        self
    }
}
impl Sub for Color {
    type Output = Color;

    #[inline(always)]
    fn sub(mut self, rhs: Self) -> Self::Output {
        self.inner -= rhs.inner;
        self
    }
}
impl Add<&Color> for Color {
    type Output = Color;

    #[inline(always)]
    fn add(mut self, rhs: &Self) -> Self::Output {
        self.inner += rhs.inner;
        self
    }
}
impl AddAssign for Color {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            inner: self.inner * rhs,
        }
    }
}
impl Div<f32> for Color {
    type Output = Color;

    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Color {
            inner: self.inner / rhs,
        }
    }
}

impl Div<f32> for &Color {
    type Output = Color;

    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Color {
            inner: self.inner / rhs,
        }
    }
}

impl<T> From<T> for Color
where
    T: Into<Vec4>,
{
    #[inline(always)]
    fn from(input: T) -> Self {
        Color {
            inner: input.into(),
        }
    }
}
