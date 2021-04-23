use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
enum Cell {
    Alive,
    Dead,
}
impl Distribution<Cell> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cell {
        if rng.gen_bool(0.5) {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }
}

type Grid = Vec<Cell>;

pub struct Game {
    height: usize,
    width: usize,
    grid: Grid,
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in 0..self.height {
            for y in 0..self.width {
                let cell = self.grid.get(x + y * self.height).unwrap();
                let symbol = if cell == &Cell::Alive { "◼" } else { " " };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
impl Game {
    fn init(width: usize, height: usize) -> Grid {
        let mut grid: Grid = Vec::with_capacity(width * height);
        let mut rng = rand::thread_rng();
        for _ in 0..width {
            for _ in 0..height {
                let cell = rng.gen::<Cell>();
                grid.push(cell);
            }
        }
        grid
    }
    fn compute_next(&self) -> Grid {
        let mut next_grid = self.grid.clone();
        for x in 0..self.height {
            for y in 0..self.width {
                let mut neighbours = 0;
                if x > 0 && y > 0 {
                    if let Some(c) = self.grid.get(x - 1 + (y - 1) * self.height) {
                        if c == &Cell::Alive {
                            neighbours += 1
                        }
                    }
                }
                if y > 0 {
                    if let Some(c) = self.grid.get(x + (y - 1) * self.height) {
                        if c == &Cell::Alive {
                            neighbours += 1
                        }
                    }
                    if let Some(c) = self.grid.get(x + 1 + (y - 1) * self.height) {
                        if c == &Cell::Alive {
                            neighbours += 1
                        }
                    }
                }
                if let Some(c) = self.grid.get(x + 1 + y * self.height) {
                    if c == &Cell::Alive {
                        neighbours += 1
                    }
                }
                if let Some(c) = self.grid.get(x + 1 + (y + 1) * self.height) {
                    if c == &Cell::Alive {
                        neighbours += 1
                    }
                }
                if let Some(c) = self.grid.get(x + (y + 1) * self.height) {
                    if c == &Cell::Alive {
                        neighbours += 1
                    }
                }
                if x > 0 {
                    if let Some(c) = self.grid.get(x - 1 + (y + 1) * self.height) {
                        if c == &Cell::Alive {
                            neighbours += 1
                        }
                    }
                    if let Some(c) = self.grid.get(x - 1 + y * self.height) {
                        if c == &Cell::Alive {
                            neighbours += 1
                        }
                    }
                }

                let cell = next_grid.get_mut(x + y * self.height).unwrap();
                let alive = match cell {
                    &mut Cell::Alive => true,
                    _ => false,
                };
                match neighbours {
                    0..=1 if alive => *cell = Cell::Dead,
                    4..=8 if alive => *cell = Cell::Dead,
                    3 if !alive => *cell = Cell::Alive,
                    _ => (),
                }
            }
        }
        next_grid
    }

    pub fn new(width: usize, height: usize) -> Self {
        let grid = Self::init(width, height);
        Self {
            width,
            height,
            grid,
        }
    }
    pub fn tick(&mut self) {
        let next_grid = self.compute_next();
        self.grid = next_grid;
    }
}
