use crate::text::Font;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render;

use std::collections::HashMap;

use crate::entity::EntityKind;
use crate::geometry::Vertex;
use crate::text;
use crate::world::{Entities, Geometries, Healths, GRID_HEIGHT, GRID_WIDTH, PLAYER_HEALTH_MAX};

// Screen coordinate bounds.
const WIN_WIDTH: u32 = 600;
const WIN_HEIGHT: u32 = 600;

// TODO: Parameterize
const TEXT_COLOR: Color = Color::RGBA(255, 80, 255, 255);
const TEXT_LINE_PADDING: u32 = 30;

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

/// Draws a health bar with a border, in a fixed position
fn draw_health_bar(canvas: &mut render::WindowCanvas, health: u32) {
    let x = 20;
    let max_width = 100;
    let x_increment = max_width / PLAYER_HEALTH_MAX as u32;
    let y = 20;
    let height = 20;
    let bar_color = Color::GREEN;
    let border_color = Color::GREY;
    canvas.set_draw_color(bar_color);
    canvas
        .draw_rect(Rect::new(x, y, health * x_increment, height))
        .unwrap();
    canvas.set_draw_color(border_color);
    canvas
        .draw_rect(Rect::new(x - 1, y - 1, max_width + 1, height + 2))
        .unwrap();
}

// Calculates the x coordinate of the left edge of the centered rectangle
fn h_center(width: u32) -> i32 {
    // Will be negative if width > screen_width. COULDDO: clamp to 0 and use u32.
    WIN_WIDTH as i32 / 2 - width as i32 / 2
}

// Calculates the y coordinate of the top edge of the centered rectangle
fn v_center(height: u32) -> i32 {
    // Will be negative if width > screen_width. COULDDO: clamp to 0 and use u32.
    WIN_HEIGHT as i32 / 2 - height as i32 / 2
}

pub struct Renderer<'ttf_context> {
    canvas: Canvas,
    font: Font<'ttf_context>,
}

impl<'ttf_context> Renderer<'ttf_context> {
    pub fn new(sdl_context: &sdl2::Sdl, font: Font<'ttf_context>) -> Renderer<'ttf_context> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Baddies, Walls and Bullets", WIN_WIDTH, WIN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        Renderer {
            canvas: window.into_canvas().build().unwrap(),
            font,
        }
    }

    /// Render the scene described by the objects.
    pub fn render(&mut self, entities: &Entities, geometries: &Geometries, healths: &Healths) {
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
        let health = healths.iter().last();
        if let Some((_, health)) = health {
            draw_health_bar(&mut self.canvas, *health as u32);
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn draw_text_n(&mut self, lines: &Vec<text::Line>, _position: text::Position) {
        // Would be good to extract this, but we can't reference it, as the return type is private.
        // Also holds references captured by closure.
        // TODO: handling of position param
        let texture_creator = self.canvas.texture_creator();
        let mut textures: Vec<(sdl2::render::Texture, u32, u32)> = vec![];
        for (text, size) in lines {
            let surface = self
                .font
                .get(size)
                .unwrap()
                .render(text)
                .blended(TEXT_COLOR)
                .unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .unwrap();
            let render::TextureQuery { width, height, .. } = texture.query();
            textures.push((texture, width, height));
        }

        let total_height: u32 = textures.iter().map(|(_, _, height)| height).sum::<u32>()
            + TEXT_LINE_PADDING * textures.len() as u32
            - 1;

        let mut curr_y = v_center(total_height) as u32; // TODO: choose based on parameter

        for (texture, width, height) in textures {
            let x = h_center(width); // TODO: choose based on parameter
            let y = curr_y;
            curr_y += height + TEXT_LINE_PADDING;
            let target = Rect::new(x, y as i32, width, height);
            &self.canvas.copy(&texture, None, Some(target)).unwrap();
        }
    }
}
