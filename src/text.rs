use sdl2::ttf;
use std::collections::HashMap;

const FONT_PATH: &str = "./LeroyLetteringLightBeta01.ttf";

// From load_font bindings
//pub type Font<'ttf_context> = ttf::Font<'ttf_context,'static>;
pub type Font<'ttf_context> = HashMap<Size, ttf::Font<'ttf_context,'static>>;

#[derive(Hash, PartialEq, Eq)]
pub enum Size {
    Small,
    Medium,
    Large
}

pub type Line<'a> = (&'a str, Size);

pub enum Position {
    CenterScreen
}

pub fn load_font(ttf_context: &ttf::Sdl2TtfContext) -> Font {
    let mut fs = Font::new();
    fs.insert(Size::Small, ttf_context.load_font(FONT_PATH, 18).unwrap());
    fs.insert(Size::Medium, ttf_context.load_font(FONT_PATH, 36).unwrap());
    fs.insert(Size::Large, ttf_context.load_font(FONT_PATH, 64).unwrap());

    fs
}

