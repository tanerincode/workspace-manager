use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{poll, read, Event, KeyCode},
    ExecutableCommand,
};
use std::io::{self};
use std::fs;
use std::path::{Path, PathBuf};
use tui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph, ListItem, List},
    layout::{Layout, Constraint, Direction, Alignment},
    style::{Style, Modifier, Color},
    text::{Span, Spans},
};
use dirs::home_dir;
use sysinfo::{System, SystemExt, ProcessorExt};


struct Project {
    name: String,
    is_dir: bool,
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut current_dir = home_dir().unwrap().join("workspace");
    let mut projects = get_workspace_content(&current_dir)?;
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(5),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .margin(0)
                .split(size);
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                ])
                .margin(1)
                .split(chunks[0]);
        
            let left_text = List::new(vec![
                ListItem::new(Spans::from(vec![
                    Span::styled("Open folder :", Style::default().fg(Color::White)),
                    Span::styled("Enter", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Spans::from(vec![
                    Span::styled("Back folder :", Style::default().fg(Color::White)),
                    Span::styled("Esc OR h", Style::default().fg(Color::White)),
                ])),
                ListItem::new(Spans::from(vec![
                    Span::styled("Quit App :", Style::default().fg(Color::White)),
                    Span::styled("q", Style::default().fg(Color::White)),
                ])),
            ])
            .block(Block::default());

            let middle_text = Spans::from(vec![
            ]);
        
            let mut sys = System::new_all();
            sys.refresh_all();
            let total_memory = sys.total_memory() as f32;
            let used_memory = sys.used_memory() as f32;
            let memory_usage = (used_memory / total_memory) * 100.0;

            let processor_info = sys.global_processor_info();
            let cpu_usage = processor_info.cpu_usage();

            let right_text = vec![
                format!("CPU: {:.1}%", cpu_usage),
                format!("MEM: {:.1}%", memory_usage),
            ].join("\n");

        
            let middle_paragraph = Paragraph::new(middle_text)
                .alignment(Alignment::Center);
            let right_paragraph = Paragraph::new(right_text)
                .block(Block::default())
                .style(Style::default().fg(Color::White));
        
            f.render_widget(left_text, top_chunks[0]);
            f.render_widget(middle_paragraph, top_chunks[1]);
            f.render_widget(right_paragraph, top_chunks[2]);
            

            let header_titles = ["Name", "Type", "Path"];
            let header_cells = header_titles.iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::Blue))
                .height(1);

            let rows: Vec<Row> = projects.iter().map(|proj| {
                let cells = vec![
                    Cell::from(proj.name.as_str()),
                    Cell::from(if proj.is_dir { "Directory" } else { "File" }),
                    Cell::from(proj.path.to_string_lossy().to_string()), // Change here
                ];
                Row::new(cells)
            }).collect();

            let table = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Projects"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::LightBlue))
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .column_spacing(1);

            f.render_stateful_widget(table, chunks[1], &mut table_state);
        })?;

        if poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    },
                    KeyCode::Esc | KeyCode::Char('h') => {
                        if current_dir != home_dir().unwrap().join("workspace") {
                            current_dir.pop(); // Navigate to the parent directory
                            projects = get_workspace_content(&current_dir)?;
                            table_state.select(Some(0));
                        }
                    },
                    KeyCode::Enter => {
                        if let Some(index) = table_state.selected() {
                            let selected_project = &projects[index];
                            if selected_project.is_dir {
                                current_dir = selected_project.path.clone();
                                projects = get_workspace_content(&current_dir)?;
                                table_state.select(Some(0));
                            }
                        }
                    },
                    KeyCode::Down => {
                        let i = match table_state.selected() {
                            Some(i) => if i >= projects.len() - 1 { 0 } else { i + 1 },
                            None => 0,
                        };
                        table_state.select(Some(i));
                    },
                    KeyCode::Up => {
                        let i = match table_state.selected() {
                            Some(i) => if i == 0 { projects.len() - 1 } else { i - 1 },
                            None => 0,
                        };
                        table_state.select(Some(i));
                    },
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn get_workspace_content(path: &Path) -> io::Result<Vec<Project>> {
    let mut items = Vec::new();
    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_name() == ".DS_Store" {
                continue; // Skip .DS_Store files
            }
            let is_dir = path.is_dir();
            items.push(Project {
                name: entry.file_name().into_string().unwrap(),
                is_dir,
                path,
            });
        }
    }
    Ok(items)
}
