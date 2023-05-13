use std::{
    io,
    time::{Duration, Instant},
};

use crate::db::prelude::*;

pub enum PrimaryMode {
    Search,
    ViewArticle,
    CreateArticle,
}

pub struct AppState {
    db: ReassociateDb,
    mode: PrimaryMode,
    quit: bool,
}

impl AppState {
    fn new(db: ReassociateDb) -> Self {
        Self {
            db,
            mode: PrimaryMode::Search,
            quit: false,
        }
    }
}

fn run<B: tui::backend::Backend>(
    terminal: &mut tui::Terminal<B>,
    mut state: AppState,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => state.quit = true,
                    _ => {}
                }
            }
        }

        if state.quit {
            return Ok(());
        }
    }
}

pub fn start(db: ReassociateDb, tick_rate: Duration) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = AppState::new(db);

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    let run_result = run(&mut terminal, state, tick_rate);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
