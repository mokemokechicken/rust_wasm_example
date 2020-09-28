use crate::geometry::vec2d::Vec2d;
use std::fmt::{self, Display, Formatter};

#[allow(unused_imports)]
use crate::log;

#[derive(Clone)]
pub struct Line {
    pub p1: Vec2d,
    pub p2: Vec2d,
}

impl Line {
    pub fn new(p1: Vec2d, p2: Vec2d) -> Self {
        Self { p1, p2 }
    }

    pub fn is_intersect(l1: &Line, l2: &Line) -> bool {
        let ret = Line::half_is_intersect(l1, l2) && Line::half_is_intersect(l2, l1);
        // log!("is_intersect({}): {} and {}", ret, l1.repr(), l2.repr());
        ret
    }

    #[inline]
    fn half_is_intersect(l1: &Line, l2: &Line) -> bool {
        let s =
            (l1.p1.x - l1.p2.x) * (l2.p1.y - l1.p1.y) - (l1.p1.y - l1.p2.y) * (l2.p1.x - l1.p1.x);
        let t =
            (l1.p1.x - l1.p2.x) * (l2.p2.y - l1.p1.y) - (l1.p1.y - l1.p2.y) * (l2.p2.x - l1.p1.x);
        s * t <= 0. //  線上でもOKとする
    }

    /// Line に対して どっち側にptがあるか。 return -1. or 1.
    pub fn point_side(&self, pt: Vec2d) -> f64 {
        let l1 = self;
        ((l1.p1.x - l1.p2.x) * (pt.y - l1.p1.y) - (l1.p1.y - l1.p2.y) * (pt.x - l1.p1.x)).signum()
    }

    pub fn intersection(l1: &Line, l2: &Line) -> Option<Vec2d> {
        if !Line::is_intersect(l1, l2) {
            None
        } else {
            let (p1, p2, p3, p4) = (&l1.p1, &l1.p2, &l2.p1, &l2.p2);
            let det = (p1.x - p2.x) * (p4.y - p3.y) - (p4.x - p3.x) * (p1.y - p2.y);
            if det == 0. {
                // 平行はNoneでいいけど、一致するような場合は何を返すべきなんだろう、このmethodは。
                None
            } else {
                let t = ((p4.y - p3.y) * (p4.x - p2.x) + (p3.x - p4.x) * (p4.y - p2.y)) / det;
                let x = t * p1.x + (1.0 - t) * p2.x;
                let y = t * p1.y + (1.0 - t) * p2.y;
                Some(Vec2d { x, y })
            }
        }
    }

    /// 垂直二等分線(線分)
    pub fn vertical_bisector(p1: Vec2d, p2: Vec2d, length: f64) -> Line {
        let mid_point = Vec2d::middle_point(p1, p2);
        let p1p2_unit = Vec2d::unit_vector_between(p1, p2);
        let line_vec = Vec2d {
            x: -p1p2_unit.y,
            y: p1p2_unit.x,
        }; // 90度回転
        Line {
            p1: mid_point - line_vec.mul(length / 2.),
            p2: mid_point + line_vec.mul(length / 2.),
        }
    }

    // #[inline]
    // pub fn is_include(&self, p0: Vec2d) -> bool {
    //     // 3点が1直線上にある条件
    //     let (p1, p2) = (self.p1, self.p2);
    //     let online = p0.x * (p2.y - p1.y) + p1.x * (p0.y - p2.y) + p2.x * (p1.y - p0.y);
    //     // p0 が p1~p2の間にある
    //     let between = (p0.x - p1.x) * (p0.x - p2.x) <= 0. && (p0.y - p1.y) * (p0.y - p2.y) <= 0.;
    //     // log!("online: {}:{}, {}", self, p0, online);
    //     online.abs() < 1e-15 && between
    // }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", self.p1, self.p2)
    }
}
