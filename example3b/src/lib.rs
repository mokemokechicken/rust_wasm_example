mod dom_util;
mod vec2d;

use dom_util::*;
use vec2d::Vec2d;

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::log_1;

fn log(s: &String) {
    log_1(&JsValue::from(s));
}

#[derive(Clone)]
struct Ball {
    id: u32,
    pos: Vec2d,
    mov: Vec2d,
    size: f64,
}

impl Ball {
    pub fn weight(&self) -> f64 {
        self.size.powi(3) as f64
    }
}

#[derive(Serialize, Deserialize)]
pub struct MyAppOptions {
    pub canvas_id: String,
    pub initial_n_balls: u32,
    pub wx: Option<u32>,
    pub wy: Option<u32>,
    pub balls_history_size: Option<u32>,
    pub save_history_per_frame: Option<u32>,
}

#[wasm_bindgen]
pub struct MyApp {
    context: web_sys::CanvasRenderingContext2d,
    i: u32,
    //
    wx: u32,
    wy: u32,
    //
    initial_n_balls: u32,
    balls: Vec<Box<Ball>>,
    power_constant: f64,
    //
    balls_history: Vec<Vec<Box<Ball>>>,
    balls_history_size: u32,
    save_history_per_frame: u32,
    //
    fps_counter: u32,
    fps_time: f64,
    last_fps: u32,
}

#[wasm_bindgen]
impl MyApp {
    pub fn new(options: JsValue) -> MyApp {
        let opts: MyAppOptions = options.into_serde().unwrap();
        MyApp {
            i: 0,
            wx: opts.wx.unwrap_or(500),
            wy: opts.wy.unwrap_or(500),
            context: context2d(&opts.canvas_id),
            initial_n_balls: opts.initial_n_balls,
            balls: vec![],
            balls_history_size: opts.balls_history_size.unwrap_or(3),
            balls_history: vec![],
            power_constant: 0.25,
            save_history_per_frame: opts.save_history_per_frame.unwrap_or(5),
            fps_counter: 0,
            fps_time: 0.,
            last_fps: 0,
        }
    }

    pub fn init(&mut self) {
        // 初期化して、Ballを生成する
        let canvas = self.context.canvas().unwrap();
        canvas.set_width(self.wx);
        canvas.set_height(self.wy);
        self.balls = Vec::with_capacity(self.initial_n_balls as usize);
        self.balls_history = Vec::with_capacity(self.balls_history_size as usize);

        let mut random = rand::thread_rng();
        for i in 0..self.initial_n_balls {
            let ball = Box::new(Ball {
                id: i as u32,
                pos: Vec2d {
                    x: random.gen_range(0., self.wx as f64),
                    y: random.gen_range(0., self.wy as f64),
                },
                mov: Vec2d {
                    x: random.gen_range(-0.5, 0.5),
                    y: random.gen_range(-0.5, 0.5),
                },
                size: random.gen_range(1., 3.0),
            });
            self.balls.push(ball);
        }
    }

    pub fn on_animation_frame(&mut self, time: f64) -> bool {
        self.i += 1;
        self.calc_fps(time);
        if self.i % self.save_history_per_frame == 0 {
            self.save_to_history();
        }
        self.conflicts();
        self.forces();
        self.moves();
        self.render();
        true
    }

    pub fn dummy(&mut self) {
        // error
        log(&format!("i={}", &self.i));
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub fn start_animation(app: MyApp) -> Result<(), JsValue> {
    let closure_owner_captured = Rc::new(RefCell::new(None));
    let closure_owner = closure_owner_captured.clone();
    let app_holder_captured = Rc::new(RefCell::new(app));

    *closure_owner.borrow_mut() = Some(Closure::wrap(Box::new(move |time: f64| {
        app_holder_captured.borrow_mut().on_animation_frame(time);
        request_animation_frame(closure_owner_captured.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));
    request_animation_frame(closure_owner.borrow().as_ref().unwrap());
    Ok(())
}

impl MyApp {
    fn calc_fps(&mut self, time: f64) {
        if self.fps_time + 1000. < time {
            self.last_fps = self.fps_counter;
            self.fps_counter = 0;
            self.fps_time = time;
        }
        self.fps_counter += 1;
    }

    fn save_to_history(&mut self) {
        // ballの軌跡を描くために位置を保存。ball別に位置をVecで保存するほうがセンスが良さそう...
        let mut history: Vec<Box<Ball>> = Vec::with_capacity(self.balls.len());
        if self.balls_history.len() == self.balls_history_size as usize {
            self.balls_history.remove(0);
        }
        for ball in self.balls.iter() {
            history.push(Box::new(ball.as_ref().clone()));
        }
        self.balls_history.push(history);
    }

    fn conflicts(&mut self) {
        //一定以上近づいたら衝突して１つのballにする
        let mut balls: Vec<Box<Ball>> = self.balls.clone();
        let mut new_balls: Vec<Box<Ball>>;
        'conflict_check: loop {
            for b1 in balls.iter() {
                for b2 in balls.iter() {
                    if b1.id == b2.id {
                        continue;
                    }
                    let distance = b1.pos.distance_from(b2.pos);
                    if distance < (b1.size + b2.size) as f64 {
                        // log(&format!("Conflict: {} and {}", b1.id, b2.id));
                        // conflict!
                        new_balls = Vec::with_capacity(balls.len() - 1);
                        for bb in balls.iter() {
                            if bb.id != b1.id && bb.id != b2.id {
                                new_balls.push(bb.clone());
                            }
                        }
                        new_balls.push(Box::new(Ball {
                            id: b1.id,
                            pos: (b1.pos.mul(b1.weight()) + b2.pos.mul(b2.weight()))
                                .mul(1. / (b1.weight() + b2.weight())), // 加重平均
                            mov: (b1.mov.mul(b1.weight()) + b2.mov.mul(b2.weight()))
                                .mul(1. / (b1.weight() + b2.weight())), // 加重平均
                            size: (b1.weight() + b2.weight()).powf(1. / 3.),
                        }));
                        balls = new_balls;
                        continue 'conflict_check;
                    }
                }
            }
            break;
        }
        if balls.len() != self.balls.len() {
            self.balls = balls;
        }
    }

    fn forces(&mut self) {
        // 全てのball同士に反発力を
        let n_balls = self.balls.len();
        let mut forces: Vec<Vec2d> = Vec::with_capacity(n_balls);
        for _ in 0..n_balls {
            forces.push(Vec2d { x: 0., y: 0. });
        }

        for (idx, b1) in self.balls.iter().enumerate() {
            for b2 in self.balls.iter() {
                if b1.id == b2.id {
                    continue;
                }
                let d = (b2.pos - b1.pos).mul(-1.); // -1 を 1 にすると引力になる
                let distance_sq = (d.x * d.x + d.y * d.y).max(1.);
                if distance_sq > 0. {
                    let power =
                        self.power_constant * (b1.weight() * b2.weight()) as f64 / distance_sq;
                    let j = d.normalize().mul(power);
                    forces[idx] = forces[idx] + j;
                }
            }
        }

        for (idx, ball) in self.balls.iter_mut().enumerate() {
            ball.mov = ball.mov + forces[idx].mul(1.0 / ball.weight());
        }
    }

    fn moves(&mut self) {
        // mx, my に従って移動、境界を超えたら反転する
        for ball in self.balls.iter_mut() {
            ball.pos = ball.pos + ball.mov;

            if ball.pos.x < 0. || ball.pos.x > self.wx as f64 {
                ball.pos.x -= ball.mov.x;
                ball.mov.x = -ball.mov.x;
            }
            if ball.pos.y < 0. || ball.pos.y > self.wy as f64 {
                ball.pos.y -= ball.mov.y;
                ball.mov.y = -ball.mov.y;
            }
        }
    }

    fn render(&self) {
        // contextにレンダリングする
        self.context.save();
        //
        self.context
            .set_fill_style(&JsValue::from("rgb(0, 0, 0, 1)"));
        self.context
            .fill_rect(0., 0., self.wx as f64, self.wy as f64);

        self.context
            .set_fill_style(&JsValue::from(format!("rgb(255, 0, 0, 1)")));
        for (i, balls) in self.balls_history.iter().enumerate() {
            let rate = (i + self.balls_history_size as usize) as f64
                / (self.balls_history_size as f64 * 2.);
            self.render_balls(balls, rate);
        }
        self.render_balls(&self.balls, 1.);
        //
        self.context
            .set_fill_style(&JsValue::from(format!("rgb(0, 0, 255, 1)")));
        self.context
            .fill_text(
                &format!("fps {} ball_num {}", self.last_fps, self.balls.len()),
                10.,
                10.,
            )
            .unwrap();
        //
        self.context.restore();
    }

    fn render_balls(&self, balls: &Vec<Box<Ball>>, rate: f64) {
        for (_, ball) in balls.iter().enumerate() {
            self.context.begin_path();
            self.context
                .arc(
                    ball.pos.x.into(),
                    ball.pos.y.into(),
                    (ball.size as f64) * rate,
                    0.,
                    std::f64::consts::PI * 2.0,
                )
                .unwrap();
            self.context.fill();
        }
    }
}
