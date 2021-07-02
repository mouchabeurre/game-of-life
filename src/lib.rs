use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

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
        for i in 0..self.height {
            for j in 0..self.width {
                let mut neighbours = 0;
                if i > 0 && j > 0 {
                    if let Some(c) = self.grid.get((i - 1) * self.width + j - 1) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                }
                if j > 0 {
                    if let Some(c) = self.grid.get(i * self.width + (j - 1)) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                    if let Some(c) = self.grid.get((i + 1) * self.width + (j - 1)) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                }
                if let Some(c) = self.grid.get((i + 1) * self.width + j) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
                if let Some(c) = self.grid.get((i + 1) * self.width + (j + 1)) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
                if let Some(c) = self.grid.get(i * self.width + (j + 1)) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
                if i > 0 {
                    if let Some(c) = self.grid.get((i - 1) * self.width + (j + 1)) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                    if let Some(c) = self.grid.get((i - 1) * self.width + j) {
                        if let &Cell::Alive(_) = c {
                            neighbours += 1
                        }
                    }
                }

                let cell = next_grid.get_mut(i * self.width + j).unwrap();
                let alive = match cell {
                    &mut Cell::Alive(_) => true,
                    _ => false,
                };
                match neighbours {
                    0..=1 if alive => *cell = Cell::Dead(DeathState::Underpopulation),
                    4..=8 if alive => *cell = Cell::Dead(DeathState::Overpopulation),
                    3 if !alive => *cell = Cell::Alive(LivingState::Reproduction),
                    _ => match cell {
                        &mut Cell::Alive(LivingState::Reproduction) => {
                            *cell = Cell::Alive(LivingState::Remains)
                        }
                        &mut Cell::Dead(DeathState::Overpopulation)
                        | &mut Cell::Dead(DeathState::Underpopulation) => {
                            *cell = Cell::Dead(DeathState::Remains)
                        }
                        _ => (),
                    },
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
    pub fn get_grid(&self) -> Grid {
        self.grid.clone()
    }
    pub fn tick(&mut self) {
        let next_grid = self.compute_next();
        self.grid = next_grid;
    }
}
