use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

use glam::{Vec4, Vec4Swizzles};

#[derive(Clone, Debug)]
pub struct Color {
    pub(crate) inner: Vec4,
}
impl Color {
    pub fn approx_luminance(&self) -> f32 {
        let [r, g, b] = self.inner.xyz().to_array();
        (r + r + b + g + g + g) / 6.0
    }
    pub fn r(&self) -> f32 {
        self.inner.x
    }
    pub fn g(&self) -> f32 {
        self.inner.y
    }
    pub fn b(&self) -> f32 {
        self.inner.z
    }
    pub fn a(&self) -> f32 {
        self.inner.w
    }
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

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.inner *= rhs.inner;
        self
    }
}
impl Mul<&Color> for Color {
    type Output = Color;

    fn mul(mut self, rhs: &Self) -> Self::Output {
        self.inner *= rhs.inner;
        self
    }
}
impl Mul<Color> for &Color {
    type Output = Color;

    fn mul(self, mut rhs: Color) -> Self::Output {
        rhs.inner = self.inner * rhs.inner;
        rhs
    }
}
impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        self.inner = self.inner * rhs.inner;
    }
}
impl Add for Color {
    type Output = Color;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.inner += rhs.inner;
        self
    }
}
impl Sub for Color {
    type Output = Color;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.inner -= rhs.inner;
        self
    }
}
impl Add<&Color> for Color {
    type Output = Color;

    fn add(mut self, rhs: &Self) -> Self::Output {
        self.inner += rhs.inner;
        self
    }
}
impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.inner *= rhs;
        self
    }
}
impl Div<f32> for Color {
    type Output = Color;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.inner /= rhs;
        self
    }
}

impl<T> From<T> for Color
where
    T: Into<Vec4>,
{
    fn from(input: T) -> Self {
        Color {
            inner: input.into(),
        }
    }
}
