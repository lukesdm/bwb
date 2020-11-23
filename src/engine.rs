use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{Duration, Instant};

use crate::game_logic::{move_cannon, try_fire, update_world, LevelState};
use crate::geometry::Direction;
use crate::levels;
use crate::render::Renderer;
use crate::text;
use crate::world;

const MAX_FPS: u32 = 60; // Max FPS. Set this low to observe effects.

type LevelId = i32;

/// Wrapper of SDL event systems, which allows cleaner event handling.
struct Events {
    // EventPump.poll_iter consumes some events that aren't relevant at the time.
    // These can then be requeued using EventSubsystem.push_event, to be handled by the more appropriate handler later.
    event_pump: sdl2::EventPump,
    event_subsystem: sdl2::EventSubsystem,
}

impl Events {
    pub fn new(event_pump: sdl2::EventPump, event_subsystem: sdl2::EventSubsystem) -> Self {
        Events {
            event_pump,
            event_subsystem,
        }
    }
    pub fn poll_iter(&mut self) -> sdl2::event::EventPollIterator {
        self.event_pump.poll_iter()
    }

    pub fn push_event(&mut self, event: Event) -> Result<(), String> {
        self.event_subsystem.push_event(event)
    }
}

enum GameState {
    ShowingTitleScreen,
    StartingLevel(LevelId),
    PlayingLevel(
        world::World,
        world::ObjectFactory,
        Instant, /* last fire time */
        LevelId,
    ),
    AdvancingLevel(LevelId),
    GameOvering, // TODO: handling
    Quitting,    // TODO: handling
}

fn title_screen(renderer: &mut Renderer, events: &mut Events) -> GameState {
    renderer.draw_text_n(
        &vec![
            ("bwb", text::Size::Large),
            ("Baddies, Walls & Bullets", text::Size::Medium),
            ("Press any key to begin...", text::Size::Small),
        ],
        text::Position::CenterScreen,
    );

    for event in events.poll_iter() {
        match event {
            Event::KeyDown {
                keycode: Some(_), ..
            } => return GameState::StartingLevel(1),
            _ => {
                // re-queue event for subsequent handlers
                events.push_event(event).unwrap();
                break;
            }
        }
    }
    GameState::ShowingTitleScreen
}

fn print_framerate(frame_time: i32) {
    let frame_rate = 1.0 / (frame_time as f32 / 1000.0);
    println!("{}", frame_rate);
}

fn init_level(curr_level: i32) -> GameState {
    let (world, obj_factory) = levels::init(curr_level);
    GameState::PlayingLevel(
        world,
        obj_factory,
        Instant::now() - Duration::from_secs(1),
        curr_level,
    )
}

fn play_level(
    renderer: &mut Renderer,
    events: &mut Events,
    frame_time: i32,
    current_time: Instant,
    mut world: world::World,
    obj_factory: world::ObjectFactory,
    mut prev_fire_time: Instant,
    curr_level: i32,
) -> GameState {
    let (world_temp, level_state) = update_world(world, frame_time);
    world = world_temp;

    match level_state {
        LevelState::Complete => return GameState::AdvancingLevel(curr_level),
        LevelState::GameOver => return GameState::GameOvering,
        _ => false,
    };

    renderer.render(&world.0, &world.2, &world.3);

    for event in events.poll_iter() {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                prev_fire_time = try_fire(
                    current_time,
                    prev_fire_time,
                    &mut world,
                    Direction::Left,
                    &obj_factory,
                )
            }
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                prev_fire_time = try_fire(
                    current_time,
                    prev_fire_time,
                    &mut world,
                    Direction::Right,
                    &obj_factory,
                )
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => move_cannon(&mut world, Direction::Up),
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => move_cannon(&mut world, Direction::Down),
            _ => {
                // re-queue event for subsequent handlers
                events.push_event(event).unwrap();
                break;
            }
        }
    }

    GameState::PlayingLevel(world, obj_factory, prev_fire_time, curr_level)
}

pub fn run() {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut renderer = Renderer::new(&sdl_context, text::load_font(&ttf_context));

    let mut events = Events::new(
        sdl_context.event_pump().unwrap(),
        sdl_context.event().unwrap(),
    );

    let mut game_state = GameState::ShowingTitleScreen;
    let mut current_time = Instant::now();

    'running: loop {
        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_millis() as i32;
        current_time = new_time;

        game_state = match game_state {
            GameState::ShowingTitleScreen => title_screen(&mut renderer, &mut events),
            GameState::StartingLevel(curr_level) => init_level(curr_level),
            GameState::PlayingLevel(world, obj_factory, prev_fire_time, curr_level) => play_level(
                &mut renderer,
                &mut events,
                frame_time,
                current_time,
                world,
                obj_factory,
                prev_fire_time,
                curr_level,
            ),
            GameState::AdvancingLevel(curr_level) => GameState::StartingLevel(curr_level + 1), // TODO: level complete screen; last level?
            GameState::GameOvering => GameState::Quitting, // TODO: game over screen
            GameState::Quitting => break 'running,
        };

        for event in events.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => print_framerate(frame_time),
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        renderer.present();
        // Cap rendering rate. COULDDO: try and calculate more accurately i.e. account for render-time
        let frame_time = Duration::new(0, 1_000_000_000u32 / MAX_FPS);
        ::std::thread::sleep(frame_time);
    }
}
