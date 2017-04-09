extern crate common;
extern crate rand;

use common::*;
use common::Cell::*;

use std::collections::HashMap;

use rand::{StdRng, SeedableRng, Rand, Rng};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
// #[cfg(debug_assertions)]
// #[no_mangle]
// pub fn new_state(size: Size) -> State {
//     //skip the title screen
//     println!("debug {}",
//              if cfg!(debug_assertions) { "on" } else { "off" });
//
//     let seed: &[_] = &[42];
//     let rng: StdRng = SeedableRng::from_seed(seed);
//
//     next_level(size, (8, 8), _000011, rng)
// }
//
// #[cfg(not(debug_assertions))]
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
    }
}

const START_POS: (i32, i32) = (7, 3);

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    if state.title_screen {
        for event in events {
            match *event {
                Event::KeyPressed { key: KeyCode::R, ctrl: true, shift: _ } => {
                    println!("reset");
                    *state = new_state((platform.size)());
                }
                Event::Close |
                Event::KeyPressed { key: KeyCode::Escape, ctrl: _, shift: _ } => return true,
                _ => (),
            }
        }

        if state.player_pos == START_POS {
            println!("START");
            // *state = next_level((platform.size)(), state.rng);
        }

        print_tuple(platform, START_POS, "S");

        draw(platform, state);

        false
    } else {
        game_update_and_render(platform, state, events)
    }
}

fn print_tuple(platform: &Platform, (x, y): (i32, i32), text: &str) {
    if x >= 0 && y >= 0 {
        (platform.print_xy)(x, y, text);
    }
}


pub fn game_update_and_render(platform: &Platform,
                              state: &mut State,
                              events: &mut Vec<Event>)
                              -> bool {
    draw(platform, state);

    false
}

fn draw(platform: &Platform, state: &State) {
    for (&coords, &cell) in state.cells.iter() {
        print_cell(platform, coords, cell);
    }

    print_tuple(platform, state.player_pos, "@");
}

fn print_cell(platform: &Platform, coords: (i32, i32), cell: Cell) {
    // with_layer!(platform, CELL_LAYER, {
    print_tuple(platform, coords, &cell.to_string())
    // })
}
