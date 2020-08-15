use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

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

    pub fn distance_from(&self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn mul(&self, k: f64) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
        }
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
