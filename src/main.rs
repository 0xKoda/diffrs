use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph, Wrap},
    style::{Style, Color},
    text::{Span, Text},
};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    process::Command,
};
use tempfile::NamedTempFile;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::env;

struct DiffApp {
    left_file: NamedTempFile,
    right_file: NamedTempFile,
    left_diff_result: Text<'static>,
    right_diff_result: Text<'static>,
    original_left_content: Text<'static>,
    original_right_content: Text<'static>,
    display_diff: bool,
}

enum FileSide {
    Left,
    Right,
}

impl DiffApp {
    fn new() -> Self {
        Self {
            left_file: NamedTempFile::new().expect("Failed to create temp file"),
            right_file: NamedTempFile::new().expect("Failed to create temp file"),
            left_diff_result: Text::default(),
            right_diff_result: Text::default(),
            original_left_content: Text::default(),
            original_right_content: Text::default(),
            display_diff: false,
        }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let tui_backend = CrosstermBackend::new(stdout);
    let mut tui_terminal = Terminal::new(tui_backend)?;

    let mut app = DiffApp::new();

    if env::args().any(|arg| arg == "-f") {
        app.original_left_content = read_json(Path::new("./left.json"))?;
        app.original_right_content = read_json(Path::new("./right.json"))?;
        let left_content = std::fs::read_to_string("./left.json")?;
        let right_content = std::fs::read_to_string("./right.json")?;
        std::fs::write(app.left_file.path(), left_content)?;
        std::fs::write(app.right_file.path(), right_content)?;
    }

    let res = run_diff_app(&mut tui_terminal, app);

    disable_raw_mode()?;
    execute!(tui_terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    tui_terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_diff_app<B: Backend>(terminal: &mut Terminal<B>, mut app: DiffApp) -> io::Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('a') => {
                    open_editor(&app, FileSide::Left, terminal)
                        .map_err(|_| io::ErrorKind::BrokenPipe)?;
                    app.original_left_content = read_json(app.left_file.path()).unwrap_or_default();
                }
                KeyCode::Char('b') => {
                    open_editor(&app, FileSide::Right, terminal)
                        .map_err(|_| io::ErrorKind::BrokenPipe)?;
                    app.original_right_content = read_json(app.right_file.path()).unwrap_or_default();
                }
                KeyCode::Char('c') => {
                    app.left_file.as_file().set_len(0)?;
                    app.right_file.as_file().set_len(0)?;
                    app.original_left_content = Text::default();
                    app.original_right_content = Text::default();
                }
                KeyCode::Char('d') => {
                    let (left_diff, right_diff) = compare_json_files(&app).map_err(|_| io::ErrorKind::BrokenPipe)?;
                    app.left_diff_result = left_diff;
                    app.right_diff_result = right_diff;
                    app.display_diff = true;
                }
                KeyCode::Char('q') => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn render_ui(f: &mut Frame, app: &DiffApp) {
    let vertical_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
    let [help_section, content_section] = vertical_layout.areas(f.size());
    let horizontal_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [left_content_area, right_content_area] = horizontal_layout.areas(content_section);

    let help_message = render_help();
    f.render_widget(help_message, help_section);

    let left_content = if app.display_diff {
        app.left_diff_result.clone()
    } else {
        app.original_left_content.clone()
    };
    let left_paragraph = Paragraph::new(left_content)
        .style(Style::default())
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title("Left JSON"));
    f.render_widget(left_paragraph, left_content_area);

    let right_content = if app.display_diff {
        app.right_diff_result.clone()
    } else {
        app.original_right_content.clone()
    };
    let right_paragraph = Paragraph::new(right_content)
        .style(Style::default())
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title("Right JSON"));
    f.render_widget(right_paragraph, right_content_area);
}

fn render_help() -> Paragraph<'static> {
    let (msg, style) = (
        vec![
            "[q]".green().bold(),
            " Quit - ".into(),
            "[a]".green().bold(),
            " edit left - ".into(),
            "[b]".green().bold(),
            " edit right - ".into(),
            "[c]".green().bold(),
            " clear input - ".into(),
            "[d]".green().bold(),
            " diff JSON".into(),
        ],
        Style::default().add_modifier(Modifier::RAPID_BLINK),
    );
    let text = Text::from(Line::from(msg)).patch_style(style);
    Paragraph::new(text)
}

fn open_editor<B: Backend>(
    app: &DiffApp,
    side: FileSide,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    let mut stdout = io::stdout();

    stdout.execute(LeaveAlternateScreen)?;

    disable_raw_mode()?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let path = match side {
        FileSide::Left => app.left_file.path(),
        FileSide::Right => app.right_file.path(),
    };
    Command::new(editor).arg(path).status().expect("failed to edit");

    enable_raw_mode()?;

    stdout.execute(EnterAlternateScreen)?;

    terminal.clear()?;
    terminal.draw(|f| render_ui(f, app))?;
    Ok(())
}

fn compare_json_files(app: &DiffApp) -> Result<(Text<'static>, Text<'static>)> {
    let left_json = parse_json(app.left_file.path())?;
    let right_json = parse_json(app.right_file.path())?;
    Ok(diff_json_values(&left_json, &right_json))
}

fn parse_json(path: &std::path::Path) -> Result<Value> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json_value: Value = serde_json::from_str(&contents)?;
    Ok(json_value)
}

fn read_json(path: &std::path::Path) -> Result<Text<'static>> {
    let json_value = parse_json(path)?;
    let json_string = serde_json::to_string_pretty(&json_value)?;
    Ok(Text::from(json_string))
}

fn diff_json_values(left: &Value, right: &Value) -> (Text<'static>, Text<'static>) {
    let mut left_diff = Text::default();
    let mut right_diff = Text::default();

    if let (Some(left_map), Some(right_map)) = (left.as_object(), right.as_object()) {
        let all_keys: BTreeSet<_> = left_map.keys().chain(right_map.keys()).collect();

        for key in all_keys {
            let left_value = left_map.get(key).cloned().unwrap_or(json!(null));
            let right_value = right_map.get(key).cloned().unwrap_or(json!(null));
            if left_value == right_value {
                let line = format!("{}: {}\n", key, left_value);
                left_diff.extend(vec![Span::styled(line.clone(), Style::default().fg(Color::Green))]);
                right_diff.extend(vec![Span::styled(line, Style::default().fg(Color::Green))]);
            } else {
                let left_line = format!("{}: {}\n", key, left_value);
                let right_line = format!("{}: {}\n", key, right_value);
                left_diff.extend(vec![Span::styled(left_line, Style::default().fg(Color::Green))]);
                right_diff.extend(vec![Span::styled(right_line, Style::default().fg(Color::Red))]);
            }
        }
    } else {
        let left_str = format!("{}", left);
        let right_str = format!("{}", right);
        left_diff.extend(vec![Span::styled(left_str, Style::default().fg(Color::Green))]);
        right_diff.extend(vec![Span::styled(right_str, Style::default().fg(Color::Red))]);
    }

    (left_diff, right_diff)
}
