use clap::{crate_version, App, Arg};
use crossterm::{
    cursor,
    style::{Colorize, PrintStyledContent},
    terminal, QueueableCommand,
};
use game_of_life::{Cell, DeathState, Game, GridInitialization, LivingState};
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

const OPTION_WIDTH: &str = "width";
const OPTION_HEIGHT: &str = "height";
const OPTION_DELAY: &str = "delay";
const OPTION_CELL_PROBABILITY: &str = "cell_probability";
const OPTION_DRAW_META_STATE: &str = "draw_meta_state";
const OPTION_STATISTICS: &str = "statistics";

struct Config {
    width: usize,
    height: usize,
    delay: time::Duration,
    probability: f64,
    draw_meta_state: bool,
    statistics: bool,
}

#[derive(Copy, Clone)]
struct ProfilerData {
    draw: u128,
    tick: u128,
    overall: u128,
}
struct Profiler {
    data: Vec<ProfilerData>,
}
impl Profiler {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    fn add(&mut self, draw: u128, tick: u128, overall: u128) {
        self.data.push(ProfilerData {
            draw,
            tick,
            overall,
        })
    }
    fn get_distribution(&mut self, percentiles: Vec<f64>) -> Vec<(f64, ProfilerData)> {
        self.data.sort_by(|a, b| a.overall.cmp(&b.overall));
        self.data.reverse();
        percentiles
            .into_iter()
            .map(|p| {
                let delay = &self
                    .data
                    .get((p * (self.data.len() as f64)) as usize)
                    .unwrap();
                (p, *delay.to_owned())
            })
            .collect()
    }
}

fn main() -> crossterm::Result<()> {
    let matches = App::new("game-of-life")
        .version(crate_version!())
        .about("Simulate Conway's Game of Life in the terminal")
        .arg(
            Arg::with_name(OPTION_WIDTH)
                .value_name("INTEGER")
                .long("width")
                .short("W")
                .help("Width of the universe grid")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OPTION_HEIGHT)
                .value_name("INTEGER")
                .long("height")
                .short("H")
                .help("Height of the universe grid")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OPTION_DELAY)
                .value_name("DELAY")
                .long("delay")
                .short("d")
                .help("Delay in milliseconds between universe updates")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OPTION_CELL_PROBABILITY)
                .value_name("FLOAT")
                .long("cell-probability")
                .short("p")
                .help("Probability for a given cell to be alive at initialization")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OPTION_DRAW_META_STATE)
                .long("draw-meta")
                .short("m")
                .help(
                    r#"Indicates "meta" state of cells with additional colors:
- "dead" -> "alive":
    - reproduction: yellow
- "alive" -> "dead":
    - overpopulation: red
    - underpopulation: cyan"#,
                ),
        )
        .arg(
            Arg::with_name(OPTION_STATISTICS)
                .long("stats")
                .short("s")
                .help("Shows the distribution of time per tick on program exit"),
        )
        .get_matches();
    let term_geom = terminal::size().unwrap();
    let config = Config {
        width: matches
            .value_of(OPTION_WIDTH)
            .map(|s| s.parse::<usize>().expect("couldn't parse width value"))
            .unwrap_or(term_geom.0 as usize),
        height: matches
            .value_of(OPTION_HEIGHT)
            .map(|s| s.parse::<usize>().expect("couldn't parse height value"))
            .unwrap_or(term_geom.1 as usize),
        delay: matches
            .value_of(OPTION_DELAY)
            .map(|s| s.parse::<u64>().expect("couldn't parse delay value"))
            .map(|u| time::Duration::from_millis(u))
            .unwrap_or(time::Duration::from_millis(100)),
        probability: matches
            .value_of(OPTION_CELL_PROBABILITY)
            .map(|s| {
                s.parse::<f64>()
                    .expect("couldn't parse cell-probability value")
            })
            .unwrap_or(0.5),
        draw_meta_state: matches.is_present(OPTION_DRAW_META_STATE),
        statistics: matches.is_present(OPTION_STATISTICS),
    };

    let mut game = Game::new(
        config.width,
        config.height,
        GridInitialization::Random(config.probability),
    );

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap();

    let mut stdout = stdout();
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?
        .flush()?;

    let mut profiler: Option<Profiler> = if config.statistics {
        Some(Profiler::new())
    } else {
        None
    };
    let term_width = min(config.width, term_geom.0 as usize);
    let term_height = min(config.height, term_geom.1 as usize);
    'outer: loop {
        let t_start = time::SystemTime::now();
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        stdout.queue(cursor::MoveTo(0, 0))?;
        let grid = game.get_grid();
        for i in 0..term_height {
            for j in 0..term_width {
                stdout.queue(cursor::MoveTo(j as u16, i as u16))?;
                if let Some(cell) = grid.get(i * config.width + j) {
                    match cell {
                        Cell::Alive(state) => {
                            if config.draw_meta_state {
                                match state {
                                    LivingState::Remains => {
                                        stdout.queue(PrintStyledContent("◼".white()))?;
                                    }
                                    LivingState::Reproduction => {
                                        stdout.queue(PrintStyledContent("◼".yellow()))?;
                                    }
                                }
                            } else {
                                stdout.queue(PrintStyledContent("◼".white()))?;
                            }
                        }
                        Cell::Dead(state) => {
                            if config.draw_meta_state {
                                match state {
                                    DeathState::Remains => {
                                        stdout.queue(PrintStyledContent(" ".black()))?;
                                    }
                                    DeathState::Overpopulation => {
                                        stdout.queue(PrintStyledContent("x".red()))?;
                                    }
                                    DeathState::Underpopulation => {
                                        stdout.queue(PrintStyledContent("x".cyan()))?;
                                    }
                                }
                            } else {
                                stdout.queue(PrintStyledContent(" ".black()))?;
                            }
                        }
                    }
                }
            }
        }
        let d_draw = t_start.elapsed().unwrap();
        let t_tick = time::SystemTime::now();
        game.tick();
        let d_tick = t_tick.elapsed().unwrap();
        stdout.flush()?;
        thread::sleep(config.delay);
        if let Some(ref mut p) = profiler {
            p.add(
                d_draw.as_millis(),
                d_tick.as_millis(),
                t_start.elapsed().unwrap().as_millis(),
            );
        }
    }

    stdout
        .queue(cursor::Show)?
        .queue(terminal::LeaveAlternateScreen)?
        .flush()?;

    if let Some(mut profiler) = profiler {
        println!(
            "Statistics for [grid size: {} cells] [tick delay: {}ms]",
            game.get_grid().len(),
            config.delay.as_millis()
        );
        println!(
            "{:<10} | {:<12} | {:<9} | {:<9}",
            "percentile", "overall (ms)", "draw (ms)", "tick (ms)"
        );
        profiler
            .get_distribution(vec![0.99, 0.95, 0.70, 0.5, 0.3, 0.05, 0.01])
            .iter()
            .for_each(|(percentile, data)| {
                println!(
                    "{:>10} | {:>12} | {:>9} | {:>9}",
                    percentile * (100 as f64),
                    data.overall,
                    data.draw,
                    data.tick
                )
            })
    }
    Ok(())
}
