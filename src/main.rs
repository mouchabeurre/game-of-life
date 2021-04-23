use crossterm::{cursor, style::Print, terminal, QueueableCommand};
use game_of_life::Game;
use std::{
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
    'outer: loop {
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        if counter >= delay / MAIN_LOOP_TIMEOUT {
            stdout
                .queue(terminal::Clear(terminal::ClearType::All))?
                .queue(cursor::MoveTo(0, 0))?
                .queue(Print(&game))?
                .flush()?;
            game.tick();
            counter = 0
        } else {
            counter += 1;
        }
        thread::sleep(timeout);
    }

    stdout
        .queue(cursor::Show)?
        .queue(terminal::LeaveAlternateScreen)?
        .flush()?;
    Ok(())
}