use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};
use std::{error::Error, io};

// Struktur untuk daftar tugas dengan stateful
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn new(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct App {
    tabs: Vec<String>,
    active_tab: usize,
    task_lists: Vec<StatefulList<String>>,
    show_details: Vec<bool>,
}

impl App {
    fn new() -> Self {
        Self {
            tabs: vec![
                "Work".to_string(),
                "Personal".to_string(),
                "Hobbies".to_string(),
            ],
            active_tab: 0,
            task_lists: vec![
                StatefulList::new(vec![
                    "Finish project report".to_string(),
                    "Email manager".to_string(),
                ]),
                StatefulList::new(vec![
                    "Buy groceries".to_string(),
                    "Call family".to_string(),
                ]),
                StatefulList::new(vec![
                    "Practice guitar".to_string(),
                    "Read a book".to_string(),
                ]),
            ],
            show_details: vec![false, false, false], // Semua detail tersembunyi
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
                .split(size);

            // Render Tabs
            let tabs: Vec<_> = app.tabs.iter().map(String::as_str).collect();
            let tabs_widget = Tabs::new(tabs)
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.active_tab);
            f.render_widget(tabs_widget, chunks[0]);

            // Render Task List for the Active Tab
            let task_items: Vec<ListItem> = app.task_lists[app.active_tab]
                .items
                .iter()
                .map(|task| ListItem::new(task.clone()).style(Style::default().fg(Color::White)))
                .collect();
            let task_list = List::new(task_items)
                .block(Block::default().borders(Borders::ALL).title("Tasks"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(
                task_list,
                chunks[1],
                &mut app.task_lists[app.active_tab].state,
            );

            // Render Detail (Jika ditampilkan)
            if app.show_details[app.active_tab] {
                let detail = Paragraph::new("Detail for selected task...")
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .style(Style::default().fg(Color::Gray));
                f.render_widget(detail, chunks[1]);
            }

            // Render Instructions
            let instructions = Paragraph::new(
                "Use 1/2/3 to switch tabs, ↑/↓ to navigate, Enter to toggle details, q to quit.",
            )
            .style(Style::default().fg(Color::Gray));
            f.render_widget(instructions, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('1') => app.active_tab = 0,
                KeyCode::Char('2') => app.active_tab = 1,
                KeyCode::Char('3') => app.active_tab = 2,
                KeyCode::Enter => {
                    // Toggle detail visibility
                    app.show_details[app.active_tab] = !app.show_details[app.active_tab];
                }
                KeyCode::Down => app.task_lists[app.active_tab].next(),
                KeyCode::Up => app.task_lists[app.active_tab].previous(),
                _ => {}
            }
        }
    }
}

