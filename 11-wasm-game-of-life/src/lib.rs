#![allow(dead_code)]

mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    let message = format!("Hello, {name}!");
    alert(&message);
}

// Important to have repr to represent the value as a single byte
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    fn get_flat_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn alive_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                };
                let i = (row + delta_row) % self.height;
                let j = (column + delta_col) % self.width;
                let idx = self.get_flat_index(i, j);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for i in 0..self.height {
            for j in 0..self.width {
                let alive_neighbors = self.alive_neighbor_count(i, j);
                let idx = self.get_flat_index(i, j);

                let new_cell = match (self.cells[idx], alive_neighbors) {
                    // Alive cells only stay alive with 2-3 alive neighbors
                    (Cell::Alive, n) if n == 2 || n == 3 => Cell::Alive,
                    // Dead cells become alive with exactly 3 alive neighbors
                    (Cell::Dead, 3) => Cell::Alive,
                    // Everything else is dead :(
                    _ => Cell::Dead,
                };
                next[idx] = new_cell;
            }
        }

        self.cells = next;
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Matrix = [[u8; 6]; 6];

    const UNIVERSE: Matrix = [
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 1, 0, 0],
        [0, 0, 0, 1, 0, 0],
        [0, 0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
    ];
    const UNIVERSE_TICK: Matrix = [
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
    ];

    const UNIVERSE_2: Matrix = [
        [0, 0, 0, 1, 0, 0],
        [1, 0, 0, 0, 0, 0],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 1],
        [0, 0, 1, 0, 0, 0],
    ];

    fn get_universe(u: &Matrix) -> Universe {
        let cells: Vec<Cell> = u
            .iter()
            .flat_map(|v| v.map(|c| if c == 0 { Cell::Dead } else { Cell::Alive }))
            .collect();
        Universe {
            width: 6,
            height: 6,
            cells,
        }
    }

    #[test]
    fn test_neighbor_count() {
        let u1 = get_universe(&UNIVERSE);
        assert_eq!(u1.alive_neighbor_count(2, 3), 2);
        assert_eq!(u1.alive_neighbor_count(1, 3), 1);
        assert_eq!(u1.alive_neighbor_count(0, 0), 0);
        assert_eq!(u1.alive_neighbor_count(5, 5), 0);

        let u2 = get_universe(&UNIVERSE_2);
        assert_eq!(u2.alive_neighbor_count(1, 0), 2);
        assert_eq!(u2.alive_neighbor_count(2, 0), 4);
        assert_eq!(u2.alive_neighbor_count(5, 3), 2);
        assert_eq!(u2.alive_neighbor_count(5, 5), 1);
    }

    #[test]
    fn test_ticks_periodic_universe() {
        let mut u = get_universe(&UNIVERSE);
        let u_orig = u.clone();
        let u_alt = get_universe(&UNIVERSE_TICK);

        u.tick();
        assert_eq!(u, u_alt);
        u.tick();
        assert_eq!(u, u_orig);
    }
}
