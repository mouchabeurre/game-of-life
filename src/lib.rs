use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rayon::prelude::*;

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
impl Distribution<Cell> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cell {
        if rng.gen_bool(0.5) {
            Cell::Alive(LivingState::Remains)
        } else {
            Cell::Dead(DeathState::Remains)
        }
    }
}

type Grid = Vec<Cell>;

pub struct Game {
    height: usize,
    width: usize,
    grid: Grid,
}
impl Game {
    fn init(width: usize, height: usize) -> Grid {
        let mut grid: Grid = Vec::with_capacity(width * height);
        let mut rng = rand::thread_rng();
        for _ in 0..width * height {
            let cell: Cell = rng.gen();
            grid.push(cell);
        }
        grid
    }
    fn compute_next(&self) -> Grid {
        let next_grid = self.grid.clone();
        next_grid
            .into_par_iter()
            .enumerate()
            .map(|(x, cell)| {
                let i = x / self.width;
                let j = x % self.width;
                let mut neighbours = 0;
                if j > 0 {
                    if i > 0 {
                        // nw
                        if let Some(c) = self.grid.get((i - 1) * self.width + j - 1) {
                            if let &Cell::Alive(_) = c {
                                neighbours += 1
                            }
                        }
                    }
                    // w
                    if let Some(c) = self.grid.get(i * self.width + (j - 1)) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                    if i < self.height - 1 {
                        // sw
                        if let Some(c) = self.grid.get((i + 1) * self.width + (j - 1)) {
                            if let &Cell::Alive(_) = c {
                                neighbours += 1
                            }
                        }
                    }
                }
                if j < self.width - 1 {
                    if i > 0 {
                        // ne
                        if let Some(c) = self.grid.get((i - 1) * self.width + (j + 1)) {
                            if let &Cell::Alive(_) = c {
                                neighbours += 1
                            }
                        }
                    }
                    // e
                    if let Some(c) = self.grid.get(i * self.width + (j + 1)) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                    if i < self.height - 1 {
                        // se
                        if let Some(c) = self.grid.get((i + 1) * self.width + (j + 1)) {
                            if let &Cell::Alive(_) = c {
                                neighbours += 1
                            }
                        }
                    }
                }
                if i < self.height - 1 {
                    // s
                    if let Some(c) = self.grid.get((i + 1) * self.width + j) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                }
                if i > 0 {
                    // n
                    if let Some(c) = self.grid.get((i - 1) * self.width + j) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                }

                let alive = match cell {
                    Cell::Alive(_) => true,
                    _ => false,
                };
                match neighbours {
                    0..=1 if alive => Cell::Dead(DeathState::Underpopulation),
                    4..=8 if alive => Cell::Dead(DeathState::Overpopulation),
                    3 if !alive => Cell::Alive(LivingState::Reproduction),
                    _ => match cell {
                        Cell::Alive(LivingState::Reproduction) => Cell::Alive(LivingState::Remains),
                        Cell::Dead(DeathState::Overpopulation)
                        | Cell::Dead(DeathState::Underpopulation) => {
                            Cell::Dead(DeathState::Remains)
                        }
                        _ => cell.clone(),
                    },
                }
            })
            .collect()
    }
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Self::init(width, height);
        Self {
            width,
            height,
            grid,
        }
    }
    pub fn get_grid(&self) -> Grid {
        self.grid.clone()
    }
    pub fn tick(&mut self) {
        let next_grid = self.compute_next();
        self.grid = next_grid;
    }
}
