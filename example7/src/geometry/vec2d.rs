use core::f64::consts::PI;
use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

#[allow(dead_code)]
impl Vec2d {
    pub fn normalize(&self) -> Self {
        let d = (self.x * self.x + self.y * self.y).sqrt();
        if d == 0. {
            Vec2d { x: 0., y: 0. }
        } else {
            Vec2d {
                x: self.x / d,
                y: self.y / d,
            }
        }
    }

    #[inline]
    pub fn distance_from(&self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    #[inline]
    pub fn square(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// 0~PI な角度を返す
    pub fn arg(p1: Self, p2: Self) -> f64 {
        let cos_theta = (p1.x * p2.x + p1.y * p2.y) / (p1.square() * p2.square()).sqrt();
        cos_theta.acos()
    }

    /// 0~2PI な角度を返す
    pub fn arg_2pi(p1: Self, p2: Self) -> f64 {
        let arg = Vec2d::arg(p1, p2);
        let outer_product = p1.x * p2.y - p1.y * p2.x;
        if outer_product < 0. {
            2.0 * PI - arg
        } else {
            arg
        }
    }

    pub fn unit_vector_between(v1: Self, v2: Self) -> Self {
        (v1 - v2).div(v1.distance_from(v2))
    }

    pub fn mul(&self, k: f64) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
        }
    }

    pub fn div(&self, k: f64) -> Self {
        if k == 0. {
            Self { x: 0., y: 0. }
        } else {
            Self {
                x: self.x / k,
                y: self.y / k,
            }
        }
    }

    #[inline]
    pub fn middle_point(p1: Vec2d, p2: Vec2d) -> Vec2d {
        Vec2d {
            x: (p1.x + p2.x) / 2.,
            y: (p1.y + p2.y) / 2.,
        }
    }
}

impl Display for Vec2d {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({:.3},{:.3})", self.x, self.y)
    }
}

impl Add for Vec2d {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2d {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
