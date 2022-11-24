use std::{io, thread, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Frame, Terminal,
};

/// Enable the tui, allowing a user to solve the equation
pub fn tui() -> color_eyre::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(ui)?;

    thread::sleep(Duration::from_secs(5));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Draw tui ui
fn ui<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();
    let block = Block::default()
        .title("Balance Chemical Equation")
        .borders(Borders::ALL);
    f.render_widget(block, size)
}
