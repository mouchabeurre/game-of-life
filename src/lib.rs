use rand::Rng;
use rayon::prelude::*;
use std::convert::TryInto;

pub enum GridInitialization {
    Random(f64),
    Custom(Grid),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LivingState {
    Remains,
    Reproduction,
}
#[derive(Clone, Debug, PartialEq)]
pub enum DeathState {
    Remains,
    Overpopulation,
    Underpopulation,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Alive(LivingState),
    Dead(DeathState),
}
type Grid = Vec<Cell>;

pub struct Game {
    height: usize,
    width: usize,
    grid: Grid,
}
impl Game {
    fn init_rand(width: usize, height: usize, probability: f64) -> Grid {
        (0..width * height)
            .into_par_iter()
            .map(|_| {
                if rand::thread_rng().gen_bool(probability) {
                    Cell::Alive(LivingState::Remains)
                } else {
                    Cell::Dead(DeathState::Remains)
                }
            })
            .collect()
    }
    fn live_neighbour_count1(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        if col > 0 {
            if row > 0 {
                // nw
                if let Some(&Cell::Alive(_)) = self.grid.get((row - 1) * self.width + col - 1) {
                    count += 1
                }
            }
            // w
            if let Some(&Cell::Alive(_)) = self.grid.get(row * self.width + (col - 1)) {
                count += 1
            }
            if row < self.height - 1 {
                // sw
                if let Some(&Cell::Alive(_)) = self.grid.get((row + 1) * self.width + (col - 1)) {
                    count += 1
                }
            }
        }
        if col < self.width - 1 {
            if row > 0 {
                // ne
                if let Some(&Cell::Alive(_)) = self.grid.get((row - 1) * self.width + (col + 1)) {
                    count += 1
                }
            }
            // e
            if let Some(&Cell::Alive(_)) = self.grid.get(row * self.width + (col + 1)) {
                count += 1
            }
            if row < self.height - 1 {
                // se
                if let Some(&Cell::Alive(_)) = self.grid.get((row + 1) * self.width + (col + 1)) {
                    count += 1
                }
            }
        }
        if row < self.height - 1 {
            // s
            if let Some(&Cell::Alive(_)) = self.grid.get((row + 1) * self.width + col) {
                count += 1
            }
        }
        if row > 0 {
            // n
            if let Some(&Cell::Alive(_)) = self.grid.get((row - 1) * self.width + col) {
                count += 1
            }
        }
        count
    }
    fn _live_neighbour_count2(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        for i in [-1, 0, 1].iter() {
            if let Ok(n_row) = TryInto::<usize>::try_into((row as i32) + i) {
                for j in [-1, 0, 1].iter() {
                    if *i == 0 && *j == 0 {
                        continue;
                    }
                    if let Ok(n_col) = TryInto::<usize>::try_into((col as i32) + j) {
                        if let Some(&Cell::Alive(_)) = self.grid.get(n_row * self.width + n_col) {
                            count += 1
                        }
                    } else {
                        continue;
                    }
                }
            } else {
                continue;
            }
        }
        count
    }
    fn compute_next(&self) -> Grid {
        let next_grid = self.grid.clone();
        next_grid
            .into_par_iter()
            .enumerate()
            .map(|(x, cell)| {
                let i = x / self.width;
                let j = x % self.width;
                let neighbour_count = self.live_neighbour_count1(i, j);
                let alive = if let Cell::Alive(_) = cell {
                    true
                } else {
                    false
                };
                match neighbour_count {
                    0..=1 if alive => Cell::Dead(DeathState::Underpopulation),
                    4..=8 if alive => Cell::Dead(DeathState::Overpopulation),
                    3 if !alive => Cell::Alive(LivingState::Reproduction),
                    _ => match cell {
                        Cell::Alive(LivingState::Reproduction) => Cell::Alive(LivingState::Remains),
                        Cell::Dead(DeathState::Overpopulation)
                        | Cell::Dead(DeathState::Underpopulation) => {
                            Cell::Dead(DeathState::Remains)
                        }
                        _ => cell,
                    },
                }
            })
            .collect()
    }
    pub fn new(width: usize, height: usize, init: GridInitialization) -> Self {
        let grid = match init {
            GridInitialization::Random(probability) => Self::init_rand(width, height, probability),
            GridInitialization::Custom(grid) => {
                assert_eq!(
                    width * height,
                    grid.len(),
                    "custom grid has expected dimensions"
                );
                grid
            }
        };
        Self {
            width,
            height,
            grid,
        }
    }
    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }
    pub fn tick(&mut self) {
        let next_grid = self.compute_next();
        self.grid = next_grid;
    }
}

#[test]
fn test_game_rules() {
    let width: usize = 3;
    let height: usize = 4;
    /* "x" is "dead"
    xox
    xxo
    ooo
    xxx
    */
    let start_grid: Grid = (0..width * height)
        .enumerate()
        .map(|(x, _)| {
            let xy = (x % width, x / width);
            if xy == (1, 0) || xy == (2, 1) || xy == (0, 2) || xy == (1, 2) || xy == (2, 2) {
                Cell::Alive(LivingState::Remains)
            } else {
                Cell::Dead(DeathState::Remains)
            }
        })
        .collect();
    /*
    xxx
    oxo
    xoo
    xox
    */
    let next_grid: Grid = (0..width * height)
        .enumerate()
        .map(|(x, _)| {
            let xy = (x % width, x / width);
            if xy == (2, 1) || xy == (1, 2) || xy == (2, 2) {
                Cell::Alive(LivingState::Remains)
            } else if xy == (0, 1) || xy == (1, 3) {
                Cell::Alive(LivingState::Reproduction)
            } else if xy == (1, 0) || xy == (0, 2) {
                Cell::Dead(DeathState::Underpopulation)
            } else {
                Cell::Dead(DeathState::Remains)
            }
        })
        .collect();
    let mut game = Game::new(width, height, GridInitialization::Custom(start_grid));
    game.tick();
    assert_eq!(&next_grid, game.get_grid());
}
