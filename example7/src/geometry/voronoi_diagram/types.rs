use crate::geometry::line::Line;
use crate::geometry::polygon::Polygon;
use crate::geometry::vec2d::Vec2d;
use core::cmp::Ordering;
use core::f64::consts::PI;
use core::mem::swap;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;

#[allow(unused_imports)]
use crate::log;

//////////////////////////////

pub type ClusterId = u32;
pub type NodeId = u32;

//////////////////////////////

#[derive(Clone)]
pub struct VoronoiCenterPoint {
    pub node_id: NodeId,
    pub pos: Vec2d,
    pub cluster_id: ClusterId,
}

#[derive(Clone, PartialEq)]
pub struct VoronoiPoint {
    pub pos: Vec2d,
    pub is_corner: bool,
    pub outbounds: HashSet<u32>, // 外周ID
}

impl VoronoiPoint {
    pub fn new(pos: Vec2d) -> Self {
        Self {
            pos,
            is_corner: false,
            outbounds: HashSet::new(),
        }
    }

    pub fn corner(pos: Vec2d, outbounds: Vec<u32>) -> Self {
        Self {
            pos,
            is_corner: true,
            outbounds: HashSet::from_iter(outbounds),
        }
    }

    pub fn on_outbound(&self) -> bool {
        self.outbounds.len() > 0
    }

    pub fn uniq_points(points: &Vec<VoronoiPoint>) -> Vec<VoronoiPoint> {
        let mut ret: Vec<VoronoiPoint> = Vec::new();

        // 二重ループとか嫌なんだけど、f64があるとなんかHashとかも使えないし、いい方法が思いつかない。
        for point in points.iter() {
            let mut exist = false;
            for pt in ret.iter() {
                if pt.eq(point) {
                    exist = true;
                    break;
                }
            }
            if !exist {
                ret.push(point.clone());
            }
        }
        ret
    }
}

impl Display for VoronoiPoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.pos.fmt(f)
    }
}

#[derive(Clone)]
pub struct VoronoiLine {
    pub p1: VoronoiPoint,
    pub p2: VoronoiPoint,
    pub line: Line,
    pub cells: HashSet<CellId>, // 隣接するセル
}

impl VoronoiLine {
    pub fn new(p1: VoronoiPoint, p2: VoronoiPoint) -> Self {
        Self {
            line: Line::new(p1.pos.clone(), p2.pos.clone()),
            p1,
            p2,
            cells: HashSet::new(),
        }
    }
    pub fn another_cell(&self, my_cell_id: CellId) -> Option<CellId> {
        for cell_id in self.cells.iter() {
            if *cell_id != my_cell_id {
                return Some(*cell_id);
            }
        }
        None
    }

    pub fn is_outbound(&self) -> bool {
        self.p1.on_outbound() && self.p2.on_outbound()
    }

    // #[inline]
    // pub fn is_include(l1: &VoronoiLine, l2: &VoronoiLine) -> bool {
    //     // l1 is on l2, or , l2 is on l1
    //     let l2_is_on_l1 = l1.line.is_include(l2.line.p1) && l1.line.is_include(l2.line.p2);
    //     let l1_is_on_l2 = l2.line.is_include(l1.line.p1) && l2.line.is_include(l1.line.p2);
    //     l2_is_on_l1 || l1_is_on_l2
    // }
}

impl Display for VoronoiLine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = String::from("cells:");
        for cell in self.cells.iter() {
            s = format!("{} {}", s, *cell);
        }
        write!(f, "VoronoiLine: {} {}", self.line, s)
    }
}

pub struct VoronoiPolygon {
    pub points: Vec<VoronoiPoint>,
    pub polygon: Polygon,
}

impl VoronoiPolygon {
    pub fn new(points: Vec<VoronoiPoint>) -> Self {
        let ps: Vec<Vec2d> = points.iter().map(|p| p.pos.clone()).collect();
        Self {
            points,
            polygon: Polygon::new(ps),
        }
    }

    pub fn from_lines(point: &VoronoiCenterPoint, lines: &Vec<VoronoiLine>) -> VoronoiPolygon {
        let mut _ps: Vec<VoronoiPoint> = Vec::new();
        // CenterPointからの角度が小さい点を加える
        for line in lines.iter() {
            let mut arg1 = arg2pi_from_center(&point.pos, &line.p1.pos);
            let mut arg2 = arg2pi_from_center(&point.pos, &line.p2.pos);
            let rev = (arg1 - arg2).abs() > PI;
            if rev {
                swap(&mut arg1, &mut arg2);
            }
            if arg1 < arg2 {
                _ps.push(line.p1.clone());
            } else {
                _ps.push(line.p2.clone());
            }
        }
        let ps = sort_points_around_center(&point.pos, &_ps);
        VoronoiPolygon::new(ps)
    }

    pub fn is_include(&self, point: Vec2d) -> bool {
        self.polygon.is_include(point)
    }
}

impl Display for VoronoiPolygon {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = String::from("");
        for point in self.points.iter() {
            s = format!("{} ({},{})", s, point.pos.x, point.pos.y);
        }
        write!(f, "VoronoiPolygon:{}", s)
    }
}

pub type CellId = u32;

pub struct VoronoiCell {
    pub cell_id: CellId,
    pub point: VoronoiCenterPoint, // cellの核となる点
    pub bound: VoronoiPolygon,
    pub lines: Vec<VoronoiLine>,
}

impl VoronoiCell {
    pub fn new(cell_id: CellId, point: VoronoiCenterPoint, lines: Vec<VoronoiLine>) -> Self {
        Self {
            cell_id,
            bound: VoronoiPolygon::from_lines(&point, &lines),
            point,
            lines,
        }
    }

    pub fn is_include(&self, point: &VoronoiCenterPoint) -> bool {
        self.bound.is_include(point.pos)
    }

    pub fn get_neighbor_cells(&self) -> HashSet<CellId> {
        let mut cells: HashSet<CellId> = HashSet::new();
        for line in self.lines.iter() {
            cells.extend(line.cells.iter());
        }
        cells
    }

    pub fn intersect_with_bounds(&self, line: &Line, next_cell_id: CellId) -> DivideInfo {
        // 垂直二等分線がぶつかって、残った境界線分
        let mut cut_lines: Vec<VoronoiLine> = Vec::new();
        // 既存セルに残る点
        let mut remain_points: Vec<VoronoiPoint> = Vec::new();
        // 新しいセルにわたす点
        let mut separated_points: Vec<VoronoiPoint> = Vec::new();
        // 衝突したセル境界上の座標
        let mut break_points: Vec<VoronoiPoint> = Vec::new();
        // 新しいセルが隣接するセル
        let mut new_neighbors: HashSet<CellId> = HashSet::new();
        // 中心点の側(-1. or 1.)
        let my_side = line.point_side(self.point.pos); // 中央の側 符号

        for bound_line in self.lines.iter() {
            if let Some(pt) = Line::intersection(&bound_line.line, line) {
                let mut vpt = VoronoiPoint::new(pt);
                if bound_line.is_outbound() {
                    let ob = bound_line
                        .p1
                        .outbounds
                        .intersection(&bound_line.p2.outbounds);
                    for bound in ob {
                        vpt.outbounds.insert(*bound);
                    }
                }
                // cut line
                {
                    let s1 = line.point_side(bound_line.p1.pos);
                    let mut cut_line = if s1 == my_side {
                        VoronoiLine::new(bound_line.p1.clone(), vpt.clone())
                    } else {
                        VoronoiLine::new(bound_line.p2.clone(), vpt.clone())
                    };
                    cut_line.cells.extend(bound_line.cells.iter());
                    cut_lines.push(cut_line);
                }
                break_points.push(vpt.clone());
                remain_points.push(vpt.clone());
                separated_points.push(vpt.clone());
                if let Some(another_cell_id) = bound_line.another_cell(self.cell_id) {
                    new_neighbors.insert(another_cell_id);
                }
            }
        }
        assert_eq!(2, break_points.len());

        let mut middle_line = VoronoiLine::new(break_points[0].clone(), break_points[1].clone());
        middle_line.cells.insert(self.cell_id);
        middle_line.cells.insert(next_cell_id);

        let my_side = line.point_side(self.point.pos);
        for point in self.bound.points.iter() {
            // log!("bound_points: {}", point.pos);
            if my_side == line.point_side(point.pos) {
                // myCellならば全部残す
                remain_points.push(point.clone());
            } else {
                if !point.on_outbound() || point.is_corner {
                    // newCellならば、外周ではない or 四隅なら残す
                    separated_points.push(point.clone());
                }
            }
        }

        assert!(remain_points.len() >= 3);

        DivideInfo {
            cut_lines,
            remain_points,
            separated_points,
            middle_line,
            new_neighbors: Vec::from_iter(new_neighbors),
        }
    }

    pub fn update(&mut self, divide_info: &DivideInfo) {
        // 分割線より 全部こっち→残す 全部あっち→なくす 半分→分割して残す 分割線→残す
        let split_line = &divide_info.middle_line.line;
        let my_side = split_line.point_side(self.point.pos); // 中央の側 符号

        let mut v_lines = Vec::new();
        for my_line in self.lines.iter() {
            let s1 = split_line.point_side(my_line.p1.pos);
            let s2 = split_line.point_side(my_line.p2.pos);
            if s1 == my_side && s2 == my_side {
                v_lines.push(my_line.clone());
            }
        }

        v_lines.extend_from_slice(&divide_info.cut_lines.clone());
        v_lines.push(divide_info.middle_line.clone());

        self.bound = VoronoiPolygon::from_lines(&self.point, &v_lines);
        self.lines = v_lines;
        //log!("update cell: {}", self);
    }
}

impl Display for VoronoiCell {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = String::new();
        for line in self.lines.iter() {
            s = format!("{}\n{}", s, line);
        }
        write!(
            f,
            "Cell({}) Center={} {}{}",
            self.cell_id, self.point.pos, self.bound, s
        )
    }
}

pub struct DivideInfo {
    pub cut_lines: Vec<VoronoiLine>, // 垂直二等分線がぶつかって、残った線分
    pub remain_points: Vec<VoronoiPoint>, // 垂直二等分線より元のCell側の点 (ぶつかった点含む)
    pub separated_points: Vec<VoronoiPoint>, // 垂直二等分線より新しいCell側の点 (ぶつかった点含む)
    pub middle_line: VoronoiLine,    // 垂直二等分線の線分
    pub new_neighbors: Vec<CellId>,  // 新しいセルが隣接するセル
}

#[inline]
pub fn arg2pi_from_center(center: &Vec2d, point: &Vec2d) -> f64 {
    let diff = *point + center.mul(-1.);
    Vec2d::arg_2pi(Vec2d { x: 1., y: 0. }, diff)
}

pub fn sort_points_around_center(center: &Vec2d, points: &Vec<VoronoiPoint>) -> Vec<VoronoiPoint> {
    let mut point_list: Vec<(f64, &VoronoiPoint)> = Vec::new();
    // let cm = center.mul(-1.);
    let unique_points = VoronoiPoint::uniq_points(points);
    for point in unique_points.iter() {
        // let diff = point.pos + cm;
        // let arg = Vec2d::arg_2pi(Vec2d { x: 1., y: 0. }, diff);
        let arg = arg2pi_from_center(center, &point.pos);
        point_list.push((arg, point));
    }
    point_list.sort_by(|a, b| (a.0).partial_cmp(&(b.0)).unwrap_or(Ordering::Equal));

    let mut new_points: Vec<VoronoiPoint> = Vec::new();
    for (_, vp) in point_list {
        new_points.push(vp.clone());
    }

    new_points
}

#[allow(dead_code)]
pub fn vec_to_s<T: Display>(vec: &Vec<T>, deli: &str) -> String {
    let mut s = String::new();
    for val in vec.iter() {
        if s.is_empty() {
            s = format!("{}", val);
        } else {
            s = format!("{}{}{}", s, deli, val);
        }
    }
    s
}
