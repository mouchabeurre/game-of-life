use crossterm::{
    cursor,
    style::{Colorize, PrintStyledContent},
    terminal, QueueableCommand,
};
use game_of_life::{Cell, DeathState, Game, LivingState};
use std::{
    cmp::min,
    env,
    io::{stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

fn main() -> crossterm::Result<()> {
    const MAIN_LOOP_TIMEOUT: u64 = 10;
    const DEFAULT_DELAY: u64 = 100;
    let term_geom = terminal::size().unwrap();
    let default_width = term_geom.0 as usize;
    let default_height = term_geom.1 as usize;

    let args: Vec<String> = env::args().collect();
    let width: usize = match args.get(1) {
        Some(d) => d.parse().unwrap_or(default_width),
        None => default_width,
    };
    let height: usize = match args.get(2) {
        Some(d) => d.parse().unwrap_or(default_height),
        None => default_height,
    };
    let delay: u64 = match args.get(3) {
        Some(d) => d.parse().unwrap_or(DEFAULT_DELAY),
        None => DEFAULT_DELAY,
    };
    let timeout = time::Duration::from_millis(MAIN_LOOP_TIMEOUT);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let mut game = Game::new(width, height);

    let mut stdout = stdout();
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?
        .flush()?;

    let mut counter = delay / MAIN_LOOP_TIMEOUT + 1;
    let mut profiler: Vec<u128> = Vec::new();
    let term_width = min(width, default_width);
    let term_height = min(height, default_height);
    'outer: loop {
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        if counter >= delay / MAIN_LOOP_TIMEOUT {
            let now = time::SystemTime::now();
            stdout.queue(cursor::MoveTo(0, 0))?;
            let grid = game.get_grid();
            for i in 0..term_height {
                for j in 0..term_width {
                    stdout.queue(cursor::MoveTo(j as u16, i as u16))?;
                    if let Some(cell) = grid.get(i * width + j) {
                        match cell {
                            Cell::Alive(state) => match state {
                                LivingState::Remains => {
                                    stdout.queue(PrintStyledContent("◼".white()))?;
                                }
                                LivingState::Reproduction => {
                                    stdout.queue(PrintStyledContent("◼".yellow()))?;
                                }
                            },
                            Cell::Dead(state) => match state {
                                DeathState::Remains => {
                                    stdout.queue(PrintStyledContent(" ".black()))?;
                                }
                                DeathState::Overpopulation => {
                                    stdout.queue(PrintStyledContent("x".red()))?;
                                }
                                DeathState::Underpopulation => {
                                    stdout.queue(PrintStyledContent("x".cyan()))?;
                                }
                            },
                        }
                    }
                }
            }
            game.tick();
            counter = 0;
            let elapsed = now.elapsed().unwrap();
            profiler.push(elapsed.as_millis());
        } else {
            counter += 1;
        }
        stdout.flush()?;
        thread::sleep(timeout);
    }

    stdout
        .queue(cursor::Show)?
        .queue(terminal::LeaveAlternateScreen)?
        .flush()?;

    profiler.sort();
    profiler.reverse();

    let results: Vec<_> = vec![0.99, 0.95, 0.5, 0.05, 0.01]
        .into_iter()
        .map(|p| {
            let delay = profiler
                .get((p * (profiler.len() as f64)) as usize)
                .unwrap();
            (p, delay)
        })
        .collect();
    println!("{:?}", results);
    Ok(())
}
