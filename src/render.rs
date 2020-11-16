use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render;

use std::collections::HashMap;

use crate::entity::EntityKind;
use crate::geometry::Vertex;
use crate::world::{Entities, Geometries, GRID_HEIGHT, GRID_WIDTH};

// Screen coordinate bounds.
const WIN_WIDTH: u32 = 600;
const WIN_HEIGHT: u32 = 600;

type Canvas = sdl2::render::Canvas<sdl2::video::Window>;

fn world_to_screen(coords: &(i32, i32)) -> (i32, i32) {
    let sf_x = WIN_WIDTH as f32 / GRID_WIDTH as f32;
    let sf_y = WIN_HEIGHT as f32 / GRID_HEIGHT as f32;

    // Assume common origin, so just need to multiply
    let (wx, wy) = *coords;
    let sx = wx as f32 * sf_x;
    let sy = wy as f32 * sf_y;
    (sx as i32, sy as i32)
}

fn render_box(canvas: &mut render::WindowCanvas, box_geometry: &[Vertex], color: Color) {
    // COULDDO: Way to avoid reallocating here? (E.g. re-use existing render vec)
    let points: Vec<Point> = box_geometry
        .iter()
        .map(|p| world_to_screen(p))
        .map(|p| Point::new(p.0, p.1))
        .collect();

    canvas.set_draw_color(color);

    canvas.draw_lines(&points[..]).unwrap();
}


pub struct Renderer {
    canvas: Canvas
}

impl Renderer {
    pub fn new(sdl_context: &sdl2::Sdl) -> Renderer {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Bullets, Walls and Baddies", WIN_WIDTH, WIN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        Renderer {
            canvas: window.into_canvas().build().unwrap()
        }
    }

    pub fn render(
        &mut self,
        entities: &Entities,
        geometries: &Geometries,
    ) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
    
        let colors: HashMap<EntityKind, Color> = [
            (EntityKind::Bullet, Color::RGB(74, 143, 255)),
            (EntityKind::Wall, Color::RGB(232, 225, 81)),
            (EntityKind::Baddie, Color::RGB(235, 33, 35)),
            (EntityKind::Cannon, Color::RGB(69, 247, 105)),
        ]
        .iter()
        .cloned()
        .collect();
        for entity in entities {
            render_box(
                &mut self.canvas,
                geometries.get(&entity.get_id()).unwrap(),
                *colors.get(entity.get_kind()).unwrap(),
            );
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}