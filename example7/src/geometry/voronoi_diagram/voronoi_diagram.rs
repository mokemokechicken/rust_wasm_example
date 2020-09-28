use crate::geometry::line::Line;
use crate::geometry::vec2d::Vec2d;

#[allow(unused_imports)]
use crate::geometry::voronoi_diagram::types::{
    vec_to_s, CellId, VoronoiCell, VoronoiCenterPoint, VoronoiLine, VoronoiPoint, VoronoiPolygon,
};
use crate::log;
use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub type VoronoiCellRef = Rc<RefCell<VoronoiCell>>;

pub struct VoronoiDiagram {
    pub outbound: VoronoiPolygon,
    pub cells: Vec<VoronoiCellRef>,
}

impl VoronoiDiagram {
    pub fn new() -> Self {
        VoronoiDiagram {
            outbound: VoronoiPolygon::new(vec![
                VoronoiPoint::corner(Vec2d { x: 0., y: 0. }, vec![0, 3]),
                VoronoiPoint::corner(Vec2d { x: 1., y: 0. }, vec![0, 1]),
                VoronoiPoint::corner(Vec2d { x: 1., y: 1. }, vec![1, 2]),
                VoronoiPoint::corner(Vec2d { x: 0., y: 1. }, vec![2, 3]),
            ]),
            cells: Vec::new(),
        }
    }

    // https://www.jaist.ac.jp/~uehara/course/2014/i481f/pdf/ppt-6.pdf
    #[allow(dead_code)]
    pub fn calculate_diagram(&mut self, points: &mut Vec<VoronoiCenterPoint>) {
        let mut rng = rand::thread_rng();
        points.shuffle(&mut rng);

        for point in points.iter() {
            self.add_point(point);
        }
    }

    pub fn init_cell(&mut self, point: &VoronoiCenterPoint) {
        //log!("init cell: ({},{})", point.pos.x, point.pos.y);
        let mut lines: Vec<VoronoiLine> = Vec::new();
        let mut around_points = self.outbound.points.clone();
        let mut last_point = around_points.remove(0);
        around_points.push(last_point.clone());
        for pt in around_points.iter() {
            lines.push(VoronoiLine::new(last_point.clone(), pt.clone()));
            last_point = pt.clone();
        }
        self.add_cell(point, lines);
        return;
    }

    // 現状のCellsに新しい点を追加する
    pub fn add_point(&mut self, point: &VoronoiCenterPoint) {
        if self.cells.is_empty() {
            return self.init_cell(point);
        }

        //log!("add_point: ({},{})", point.pos.x, point.pos.y);
        let next_cell_id = self.cells.len() as CellId;
        let mut _current_cell = self
            .find_including_cell(point)
            .expect("Including Cell Not Found");

        if _current_cell.as_ref().borrow().point.pos.eq(&point.pos) {
            log!("can not add same position points");
            return;
        }

        // 既に処理したセル
        let mut checked_cell_set: HashSet<CellId> = HashSet::new();
        // 隣接した既存セル
        let mut neighbor_cells: Vec<CellId> = Vec::new();
        // 新しいcellの線分のリスト
        let mut new_cell_lines: Vec<VoronoiLine> = Vec::new();
        let mut new_cell_points: Vec<VoronoiPoint> = Vec::new();
        loop {
            {
                let mut current_cell = _current_cell.borrow_mut();
                checked_cell_set.insert(current_cell.cell_id);
                //log!("current_cell_id={}", current_cell.cell_id);

                // 垂直二等分線をもとめる -> (2)
                let middle_line = Line::vertical_bisector(point.pos, current_cell.point.pos, 5.);
                //log!("middle: {}", middle_line);
                // (2)が交差する current_cellの辺(どれか) を求める -> (3)
                let divide_info = current_cell.intersect_with_bounds(&middle_line, next_cell_id);

                // 次のセルを探す
                for another_cell_id in divide_info.new_neighbors.iter() {
                    if !checked_cell_set.contains(another_cell_id) {
                        neighbor_cells.push(*another_cell_id);
                    }
                }

                // 新Cellの線分追加
                new_cell_lines.push(divide_info.middle_line.clone());
                for pt in divide_info.separated_points.iter() {
                    if pt.is_corner {
                        new_cell_points.push(pt.clone());
                    }
                }

                // current_cellの更新
                {
                    let mut old_neighbors = current_cell.get_neighbor_cells();
                    current_cell.update(&divide_info);
                    let new_neighbors = current_cell.get_neighbor_cells();
                    old_neighbors.retain(|x| !new_neighbors.contains(x));
                    for neighbor_cell_id in old_neighbors {
                        if !checked_cell_set.contains(&neighbor_cell_id) {
                            neighbor_cells.push(neighbor_cell_id);
                        }
                    }
                }
            }

            if neighbor_cells.is_empty() {
                break;
            }
            neighbor_cells.retain(|c| *c != next_cell_id);
            let next_check_cell_id = neighbor_cells.pop().unwrap();
            _current_cell = Rc::clone(self.get_cell(next_check_cell_id));
            neighbor_cells.retain(|c| *c != next_check_cell_id);
        }

        /////////////////////////////////////////////////
        // 新しいセルを追加
        /////////////////////////////////////////////////
        // 頂点候補 -> new_cell_points
        for line in new_cell_lines.iter() {
            new_cell_points.push(line.p1.clone());
            new_cell_points.push(line.p2.clone());
        }
        // 外周上にある点は、他のとつなげる必要がある
        let mut out_point: Vec<VoronoiPoint> = Vec::new();
        for line in new_cell_lines.iter() {
            if line.p1.on_outbound() {
                out_point.push(line.p1.clone());
            }
            if line.p2.on_outbound() {
                out_point.push(line.p2.clone());
            }
        }
        for point in new_cell_points.iter() {
            if point.is_corner {
                out_point.push(point.clone());
            }
        }

        out_point.reverse();
        //log!("out_point: {}", vec_to_s(&out_point, ","));
        while !out_point.is_empty() {
            let tp = out_point.pop().unwrap();
            new_cell_points.retain(|p| !p.eq(&tp));
            out_point.retain(|p| !p.eq(&tp));
            for point in new_cell_points.clone().iter() {
                if tp.outbounds.intersection(&point.outbounds).count() > 0 {
                    let vl = VoronoiLine::new(tp.clone(), point.clone());
                    //log!("add bound line: {}", vl.line);
                    new_cell_lines.push(vl);
                    break;
                }
            }
        }
        self.add_cell(point, new_cell_lines);
        /////////////////////////////////////////////////
    }

    fn find_including_cell(&self, point: &VoronoiCenterPoint) -> Option<VoronoiCellRef> {
        for cell in self.cells.iter() {
            if cell.borrow().is_include(point) {
                return Some(Rc::clone(cell));
            }
        }
        None
    }

    fn add_cell(&mut self, point: &VoronoiCenterPoint, lines: Vec<VoronoiLine>) -> VoronoiCellRef {
        let cell_id = self.cells.len() as u32;
        let _new_cell = VoronoiCell::new(cell_id, point.clone(), lines);
        let new_cell = Rc::new(RefCell::new(_new_cell));
        for line in new_cell.as_ref().borrow_mut().lines.iter_mut() {
            line.cells.insert(cell_id);
        }

        ///////////////////////
        //log!("=============== Add\n{}", new_cell.as_ref().borrow());
        ///////////////////////

        self.cells.push(Rc::clone(&new_cell));
        new_cell
    }

    fn get_cell(&self, cell_id: CellId) -> &VoronoiCellRef {
        self.cells.get(cell_id as usize).unwrap()
    }
}
