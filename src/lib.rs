use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
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

fn compute_next(grid: Grid, width: usize, height: usize, start: usize, end: usize) -> Grid {
    let mut next_grid = grid.clone();
    for x in start..end {
        let i = x / width;
        let j = x % width;
        let mut neighbours = 0;
        if j > 0 {
            if i > 0 {
                // nw
                if let Some(c) = grid.get((i - 1) * width + j - 1) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
            }
            // w
            if let Some(c) = grid.get(i * width + (j - 1)) {
                if let &Cell::Alive(_) = c {
                    neighbours += 1
                }
            }
            if i < height - 1 {
                // sw
                if let Some(c) = grid.get((i + 1) * width + (j - 1)) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
            }
        }
        if j < width - 1 {
            if i > 0 {
                // ne
                if let Some(c) = grid.get((i - 1) * width + (j + 1)) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
            }
            // e
            if let Some(c) = grid.get(i * width + (j + 1)) {
                if let &Cell::Alive(_) = c {
                    neighbours += 1
                }
            }
            if i < height - 1 {
                // se
                if let Some(c) = grid.get((i + 1) * width + (j + 1)) {
                    if let &Cell::Alive(_) = c {
                        neighbours += 1
                    }
                }
            }
        }
        if i < height - 1 {
            // s
            if let Some(c) = grid.get((i + 1) * width + j) {
                if let &Cell::Alive(_) = c {
                    neighbours += 1
                }
            }
        }
        if i > 0 {
            // n
            if let Some(c) = grid.get((i - 1) * width + j) {
                if let &Cell::Alive(_) = c {
                    neighbours += 1
                }
            }
        }

        let cell = next_grid.get_mut(i * width + j).unwrap();
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
    next_grid.drain(start..end).collect()
}
pub struct Game {
    height: usize,
    width: usize,
    thread_count: usize,
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
    fn compute_next_multi(&self) -> Grid {
        let (tx, rx) = mpsc::channel::<(usize, Grid)>();
        let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(self.thread_count);
        for i in 0..self.thread_count {
            let tx = tx.clone();
            let grid = self.grid.clone();
            let thread_count = self.thread_count.clone();
            let width = self.width.clone();
            let height = self.height.clone();
            let handle = thread::spawn(move || {
                let start: usize = i * grid.len() / thread_count;
                let end: usize = (i + 1) * grid.len() / thread_count;
                let grid = compute_next(grid, width, height, start, end);
                tx.send((i, grid)).unwrap();
            });
            handles.push(handle);
        }
        drop(tx);
        let mut grids: Vec<(usize, Grid)> = Vec::with_capacity(self.thread_count);
        for sub_grid in rx {
            grids.push(sub_grid);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        grids.sort_by(|a, b| a.0.cmp(&b.0));
        grids.into_iter().flat_map(|x| x.1).collect()
    }
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Self::init(width, height);
        let mut thread_count: usize = num_cpus::get();
        while ((width * height) as u64 / thread_count as u64) < 1 {
            thread_count -= 1;
        }
        println!("{}", thread_count);
        assert!(thread_count >= 1);
        Self {
            width,
            height,
            thread_count,
            grid,
        }
    }
    pub fn get_grid(&self) -> Grid {
        self.grid.clone()
    }
    pub fn tick(&mut self) {
        let next_grid = self.compute_next_multi();
        self.grid = next_grid;
    }
}
