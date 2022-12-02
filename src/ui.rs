use std::io;

use arboard::Clipboard;
use chem_eq::{balance::EquationBalancer, Equation, error::{EquationError, BalanceError}};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame, Terminal,
};

#[derive(Debug, Default)]
struct App {
    pub input_mode: InputMode,
    pub input: String,
    pub output: Option<Result<Equation, Error>>,
}

impl App {
    pub fn input_body(&self) -> impl Widget + '_ {
        let (text, text_colour) = if self.input.is_empty() {
            ("Input equation...", Color::DarkGray)
        } else {
            (self.input.as_str(), Color::Yellow)
        };
        let (text_style, border_style) = if let InputMode::Editing = self.input_mode {
            (
                Style::default().fg(text_colour),
                Style::default().fg(Color::Yellow),
            )
        } else {
            (
                Style::default().fg(Color::DarkGray),
                Style::default().fg(Color::DarkGray),
            )
        };
        Paragraph::new(Span::styled(format!(" {}", text), text_style))
            .style(border_style)
            .block(Block::default().borders(Borders::ALL))
    }

    pub fn output_body(&self) -> impl Widget + '_ {
        let text = self.output.as_ref().map_or_else(
            || "Waiting for equation...".to_string(),
            |r| {
                let res = r
                    .as_ref()
                    .map(Equation::equation);
                match res {
                    Ok(s) => s.to_string(),
                    Err(Error::Eq(EquationError::ParsingError(_))) => "Couldn't parse equation".to_string(),
                    Err(Error::Eq(EquationError::IncorrectEquation)) => "Equation was not valid".to_string(),
                    Err(Error::Balance(e)) => e.to_string(),
                }
            },
        );
        let style = match &self.output {
            Some(Ok(_)) => Style::default().fg(Color::Green),
            Some(Err(_)) => Style::default().fg(Color::Red),
            None => Style::default().fg(Color::DarkGray),
        };
        Paragraph::new(format!(" {}", text))
            .style(style)
            .block(Block::default().borders(Borders::ALL))
    }

    pub fn update_eq(&mut self) {
        if self.input.is_empty() {
            self.output = None;
            return;
        }
        let res = Equation::new(self.input.as_str());
        let Ok(eq) = res else {
            self.output = Some(res.map_err(Into::into));
            return;
        };
        let balancer = EquationBalancer::new(&eq);
        let eq = balancer.balance().map_err(Into::into);
        self.output = Some(eq);
    }
}

#[derive(Debug, Default)]
enum InputMode {
    Editing,
    #[default]
    Normal,
}

impl InputMode {
    pub const fn to_help(&self) -> &'static str {
        match self {
            Self::Normal => " i or e          to edit\n q or esc        to quit\n y               to copy balanced equation",
            Self::Editing => " esc or ctrl-[   leave editing mode\n",
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    Eq(EquationError),
    Balance(BalanceError),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::Eq(EquationError::ParsingError(_)) => "Equation could not be balanced".to_string(),
            Self::Eq(EquationError::IncorrectEquation) => "Equation is not valid".to_string(),
            Self::Balance(e) => e.to_string(),
        }
    }
}

impl From<EquationError> for Error {
    fn from(e: EquationError) -> Self {
        Self::Eq(e)
    }
}

impl From<BalanceError> for Error {
    fn from(e: BalanceError) -> Self {
        Self::Balance(e)
    }
}

/// Enable the tui, allowing a user to solve the equation
pub fn tui() -> color_eyre::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // app state
    let mut app = App::default();
    let mut clipboard = Clipboard::new()?;

    loop {
        terminal.draw(|f| ui(f, &app))?;
        if let Event::Key(key) = event::read()? {
            match (&app.input_mode, key.code) {
                (_, KeyCode::Char('c')) if key.modifiers == KeyModifiers::CONTROL => break,
                (InputMode::Normal, KeyCode::Char('q') | KeyCode::Esc) => break,
                (InputMode::Normal, KeyCode::Char('i' | 'e')) => {
                    app.input_mode = InputMode::Editing;
                }
                (InputMode::Normal, KeyCode::Char('y')) => {
                    if let Some(Ok(ref eq)) = app.output {
                        clipboard.set_text(eq.to_string())?;
                    }
                }
                (InputMode::Editing, KeyCode::Esc) => app.input_mode = InputMode::Normal,
                (InputMode::Editing, KeyCode::Char('['))
                    if key.modifiers == KeyModifiers::CONTROL =>
                {
                    app.input_mode = InputMode::Normal;
                }
                (InputMode::Editing, KeyCode::Char(c)) => {
                    app.input.push(c);
                    app.update_eq();
                }
                (InputMode::Editing, KeyCode::Backspace) => {
                    app.input.pop();
                    app.update_eq();
                }
                _ => {}
            }
        }
    }

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
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    // title
    let title = Paragraph::new("Chemical Equation Balancer")
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, chunks[0]);

    // input area
    let input_body = app.input_body();
    f.render_widget(input_body, chunks[1]);

    // output area
    let output = app.output_body();
    f.render_widget(output, chunks[2]);

    // help area
    let help_body = Paragraph::new(app.input_mode.to_help())
        .block(Block::default().title("Help").borders(Borders::ALL));
    f.render_widget(help_body, chunks[3]);

    // cursor
    match app.input_mode {
        InputMode::Editing => {
            f.set_cursor(chunks[1].x + app.input.len() as u16 + 2, chunks[1].y + 1);
        }
        InputMode::Normal => {}
    }
}
