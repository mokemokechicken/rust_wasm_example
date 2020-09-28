use crate::geometry::vec2d::Vec2d;

#[inline]
#[allow(dead_code)]
pub fn euclid_distance(p1: Vec2d, p2: Vec2d) -> f64 {
    p1.distance_from(p2)
}

#[inline]
#[allow(dead_code)]
pub fn poincare_distance(p1: Vec2d, p2: Vec2d) -> f64 {
    let a = (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2);
    let b = (1. - p1.square()) * (1. - p2.square());
    (1. + (2. * a / b)).acosh()
}
