mod boid;
mod config;
mod simulation;
mod ui;

use crate::ui::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Get terminal size
    let terminal_size = terminal.size()?;
    let app = App::new(terminal_size);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        let target_fps = if app.high_fps { 60 } else { 30 };
        let frame_duration = Duration::from_millis(1000 / target_fps);
        let start_time = Instant::now();

        if event::poll(frame_duration)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(' ') => app.toggle_pause(),
                    KeyCode::Char('f') => app.toggle_fps(),
                    KeyCode::Char('r') => app.reset(),
                    _ => {}
                }
            }
        }

        if !app.paused {
            app.update();
        }

        let elapsed = start_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}