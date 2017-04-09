extern crate common;
extern crate rand;

use common::*;
use common::Cell::*;
use common::Motion::*;

use std::collections::HashMap;

use rand::{StdRng, SeedableRng, Rand, Rng};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
#[cfg(debug_assertions)]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //skip the title screen
    println!("debug {}",
             if cfg!(debug_assertions) { "on" } else { "off" });

    let seed: &[_] = &[42];
    let rng: StdRng = SeedableRng::from_seed(seed);

    next_level(size, rng)
}

#[cfg(not(debug_assertions))]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //show the title screen
    let seed: &[_] = &[42];
    let rng: StdRng = SeedableRng::from_seed(seed);

    let mut cells = HashMap::new();

    cells.insert((5, 2), Wall);
    cells.insert((4, 3), Wall);
    cells.insert((6, 3), Wall);

    cells.insert((4, 6), Wall);
    cells.insert((5, 7), Wall);

    cells.insert((10, 7), Wall);
    cells.insert((11, 6), Wall);

    cells.insert((11, 3), Wall);
    cells.insert((10, 2), Wall);

    State {
        player_pos: (5, 3),
        cells: cells,
        rng: rng,
        title_screen: true,
        frame_count: 0,
        motion: Stopped,
    }
}

const START_POS: (i32, i32) = (7, 3);

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    state.frame_count = state.frame_count.overflowing_add(1).0;

    if state.title_screen {
        for event in events {
            cross_mode_event_handling(platform, state, event);

            match *event {
                Event::Close |
                Event::KeyPressed { key: KeyCode::Escape, ctrl: _, shift: _ } => return true,
                _ => (),
            }
        }

        if state.player_pos == START_POS {
            *state = next_level((platform.size)(), state.rng);
        } else {
            move_player((platform.size)(), state);
        }

        print_tuple(platform, START_POS, goal_string(state.frame_count));

        draw(platform, state);

        draw_button(platform,
                    6,
                    8,
                    3,
                    3,
                    "↑",
                    (platform.key_pressed)(KeyCode::Up));
        draw_button(platform,
                    3,
                    11,
                    3,
                    3,
                    "←",
                    (platform.key_pressed)(KeyCode::Left));
        draw_button(platform,
                    6,
                    11,
                    3,
                    3,
                    "↓",
                    (platform.key_pressed)(KeyCode::Down));
        draw_button(platform,
                    9,
                    11,
                    3,
                    3,
                    "→",
                    (platform.key_pressed)(KeyCode::Right));

        false
    } else {
        game_update_and_render(platform, state, events)
    }
}

fn draw_button(platform: &Platform, x: i32, y: i32, w: i32, h: i32, label: &str, pressed: bool) {

    if pressed {
        draw_pressed_button_rect(platform, x, y, w, h);
    } else {
        draw_unpressed_button_rect(platform, x, y, w, h);
    }

    (platform.print_xy)(x + 1, y + 1, label);
}

pub fn game_update_and_render(platform: &Platform,
                              state: &mut State,
                              events: &mut Vec<Event>)
                              -> bool {
    for event in events {
        cross_mode_event_handling(platform, state, event);

        match *event {
            Event::Close |
            Event::KeyPressed { key: KeyCode::Escape, ctrl: _, shift: _ } => return true,
            _ => (),
        }
    }

    move_player((platform.size)(), state);

    if let Some(&Goal) = state.cells.get(&state.player_pos) {
        *state = next_level((platform.size)(), state.rng);
    }

    draw(platform, state);

    false
}

fn move_player(size: Size, state: &mut State) {
    match state.motion {
        Stopped => {}
        Up => {
            let target = add(state.player_pos, (0, -1));
            if can_go(size, &state.cells, target) {
                state.player_pos = target;
            } else {
                state.motion = Stopped;
            }
        }
        Right => {
            let target = add(state.player_pos, (1, 0));
            if can_go(size, &state.cells, target) {
                state.player_pos = target;
            } else {
                state.motion = Stopped;
            }
        }
        Down => {
            let target = add(state.player_pos, (0, 1));
            if can_go(size, &state.cells, target) {
                state.player_pos = target;
            } else {
                state.motion = Stopped;
            }
        }
        Left => {
            let target = add(state.player_pos, (-1, 0));
            if can_go(size, &state.cells, target) {
                state.player_pos = target;
            } else {
                state.motion = Stopped;
            }
        }
    }
}

fn cross_mode_event_handling(platform: &Platform, state: &mut State, event: &Event) {
    match *event {
        Event::KeyPressed { key: KeyCode::W, ctrl: _, shift: _ } |
        Event::KeyPressed { key: KeyCode::Up, ctrl: _, shift: _ } => {
            if state.motion == Stopped {
                state.motion = Up;
            }
        }
        Event::KeyPressed { key: KeyCode::D, ctrl: _, shift: _ } |
        Event::KeyPressed { key: KeyCode::Right, ctrl: _, shift: _ } => {
            if state.motion == Stopped {
                state.motion = Right;
            }
        }
        Event::KeyPressed { key: KeyCode::S, ctrl: _, shift: _ } |
        Event::KeyPressed { key: KeyCode::Down, ctrl: _, shift: _ } => {
            if state.motion == Stopped {
                state.motion = Down;
            }
        }
        Event::KeyPressed { key: KeyCode::A, ctrl: _, shift: _ } |
        Event::KeyPressed { key: KeyCode::Left, ctrl: _, shift: _ } => {
            if state.motion == Stopped {
                state.motion = Left;
            }
        }
        Event::KeyPressed { key: KeyCode::R, ctrl: true, shift: _ } => {
            println!("reset");
            *state = new_state((platform.size)());
        }
        _ => (),
    }
}

fn can_go(size: Size, cells: &Cells, (x, y): (i32, i32)) -> bool {
    if x >= 0 && y >= 0 && x < size.width && y < size.height {

        match cells.get(&(x, y)) {
            None => true,
            Some(&Goal) => true,
            Some(&Wall) => false,
        }
    } else {
        false
    }
}

fn goal_string(frame_count: u32) -> &'static str {
    match frame_count & 31 {
        1 => "\u{E010}",
        2 => "\u{E011}",
        3 => "\u{E011}",
        4 => "\u{E012}",
        5 => "\u{E012}",
        6 => "\u{E013}",
        7 => "\u{E013}",
        8 => "\u{E014}",
        9 => "\u{E014}",
        10 => "\u{E015}",
        11 => "\u{E015}",
        12 => "\u{E016}",
        13 => "\u{E016}",
        14 => "\u{E017}",
        15 => "\u{E017}",
        16 => "\u{E018}",
        17 => "\u{E017}",
        18 => "\u{E017}",
        19 => "\u{E016}",
        20 => "\u{E016}",
        21 => "\u{E015}",
        22 => "\u{E015}",
        23 => "\u{E014}",
        24 => "\u{E014}",
        25 => "\u{E013}",
        26 => "\u{E013}",
        27 => "\u{E012}",
        28 => "\u{E012}",
        29 => "\u{E011}",
        30 => "\u{E011}",
        31 => "\u{E010}",
        _ => "\u{E010}",
    }
}

fn print_tuple(platform: &Platform, (x, y): (i32, i32), text: &str) {
    if x >= 0 && y >= 0 {
        (platform.print_xy)(x, y, text);
    }
}

fn draw(platform: &Platform, state: &State) {
    for (&coords, &cell) in state.cells.iter() {
        print_cell(platform, coords, cell, state.frame_count);
    }

    print_tuple(platform, state.player_pos, "@");
}

fn draw_unpressed_button_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(platform,
                   x,
                   y,
                   w,
                   h,
                   ["┌", "─", "╖", "│", "║", "╘", "═", "╝"]);
}

fn draw_pressed_button_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(platform,
                   x,
                   y,
                   w,
                   h,
                   ["╔", "═", "╕", "║", "│", "╙", "─", "┘"]);
}

fn draw_rect_with(platform: &Platform, x: i32, y: i32, w: i32, h: i32, edges: [&str; 8]) {
    (platform.clear)(Some(Rect::from_values(x, y, w, h)));

    let right = x + w - 1;
    let bottom = y + h - 1;
    // top
    (platform.print_xy)(x, y, edges[0]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, y, edges[1]);
    }
    (platform.print_xy)(right, y, edges[2]);

    // sides
    for i in (y + 1)..bottom {
        (platform.print_xy)(x, i, edges[3]);
        (platform.print_xy)(right, i, edges[4]);
    }

    //bottom
    (platform.print_xy)(x, bottom, edges[5]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, bottom, edges[6]);
    }
    (platform.print_xy)(right, bottom, edges[7]);
}

fn print_cell(platform: &Platform, coords: (i32, i32), cell: Cell, frame_count: u32) {
    match cell {
        Goal => print_tuple(platform, coords, goal_string(frame_count)),
        _ => print_tuple(platform, coords, &cell.to_string()),
    }
    // with_layer!(platform, CELL_LAYER, {
    // })
}

fn next_level(size: Size, mut rng: StdRng) -> State {
    let mut cells = HashMap::new();

    cells.insert((0, 0), Goal);

    State {
        player_pos: (7, 3),
        cells: cells,
        rng: rng,
        title_screen: false,
        frame_count: 0,
        motion: Stopped,
    }
}

use std::ops::Add;
fn add<T: Add<Output = T>>((x1, y1): (T, T), (x2, y2): (T, T)) -> (T, T) {
    (x1 + x2, y1 + y2)
}

use std::ops::Sub;
fn sub<T: Sub<Output = T>>((x1, y1): (T, T), (x2, y2): (T, T)) -> (T, T) {
    (x1 - x2, y1 - y2)
}
