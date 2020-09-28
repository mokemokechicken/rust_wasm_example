use crate::geometry::line::Line;
use crate::geometry::vec2d::Vec2d;

#[derive(Clone)]
pub struct Polygon {
    pub points: Vec<Vec2d>,
    pub lines: Vec<Line>,
}

// 凸な多角形
impl Polygon {
    pub fn new(points: Vec<Vec2d>) -> Self {
        let lines = Polygon::each_lines(&points);
        Self { points, lines }
    }

    pub fn each_lines(points: &Vec<Vec2d>) -> Vec<Line> {
        if points.len() < 3 {
            return vec![];
        }
        let mut lines: Vec<Line> = Vec::with_capacity(points.len() - 1);

        let mut _p1: Option<&Vec2d> = None;
        for p2 in points.iter().chain(vec![points.first().unwrap()]) {
            if let Some(p1) = _p1 {
                lines.push(Line::new(p1.clone(), p2.clone()));
            }
            _p1 = Some(p2);
        }
        lines
    }

    pub fn is_include(&self, point: Vec2d) -> bool {
        // https://www.nttpc.co.jp/technology/number_algorithm.html
        // 1.1.Crossing Number Algorithm（交差数判定）
        assert!(self.points.len() >= 3);
        let mut cross_count = 0;
        for line in self.lines.iter() {
            if Self::is_cross_with_line_to_right(&point, line) {
                cross_count += 1;
            }
        }
        cross_count % 2 == 1
    }

    /// point から右側に水平にのばした線が、lineと交差するか?
    #[inline]
    fn is_cross_with_line_to_right(point: &Vec2d, line: &Line) -> bool {
        let max_x = line.p1.x.max(line.p2.x).max(point.x) + 1.;
        let line_to_right = Line::new(
            point.clone(),
            Vec2d {
                x: max_x,
                y: point.y,
            },
        );
        Line::is_intersect(&line_to_right, line)
    }
}
