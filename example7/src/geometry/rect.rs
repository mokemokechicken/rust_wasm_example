use crate::geometry::vec2d::Vec2d;

pub struct Rect {
    pub pos: Vec2d,
    pub size: Vec2d,
}

#[allow(dead_code)]
impl Rect {
    pub fn is_overlap(&self, other: &Self) -> bool {
        let center1 = self.pos + self.size.mul(0.5);
        let center2 = other.pos + other.size.mul(0.5);
        let x_diff = (center1.x - center2.x).abs();
        let y_diff = (center1.y - center2.y).abs();
        x_diff < (self.size.x + other.size.x) / 2. && y_diff < (self.size.y + other.size.y) / 2.
    }

    pub fn is_in(&self, point: Vec2d) -> bool {
        self.pos.x <= point.x
            && point.x < self.pos.x + self.size.x
            && self.pos.y <= point.y
            && point.y < self.pos.y + self.size.y
    }
}
