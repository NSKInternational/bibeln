use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{SetForegroundColor, ResetColor, Color},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{stdout, Write, Result};
use std::process::Command;
use std::time::Duration;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

const ASCII_ART: &str = r#"
| |   (_) |        | |      
| |__  _| |__   ___| |_ __  
| '_ \| | '_ \ / _ \ | '_ \ 
| |_) | | |_) |  __/ | | | |
|_.__/|_|_.__/ \___|_|_| |_|
"#;

fn draw(stdout: &mut std::io::Stdout, cols: u16, rows: u16) -> Result<()> {
    stdout.execute(terminal::Clear(ClearType::All))?;

    if cols < MIN_WIDTH || rows < MIN_HEIGHT {
        let message = format!(
            "Window too small.\nMinimum: {}x{}\nCurrent: {}x{}",
            MIN_WIDTH, MIN_HEIGHT, cols, rows
        );
        let lines: Vec<&str> = message.lines().collect();
        let start_row = (rows / 2).saturating_sub(lines.len() as u16 / 2);
        for (i, line) in lines.iter().enumerate() {
            let col = (cols.saturating_sub(line.len() as u16)) / 2;
            stdout.execute(cursor::MoveTo(col, start_row + i as u16))?;
            writeln!(stdout, "{}", line)?;
        }
    } else {
        let lines: Vec<&str> = ASCII_ART.trim_matches('\n').lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let col = (cols.saturating_sub(line.len() as u16)) / 2;
            stdout.execute(cursor::MoveTo(col, 2 + i as u16))?;
            writeln!(stdout, "{}", line)?;
        }
        let footer = "[q]uit - [c]heck";
        let col = (cols.saturating_sub(footer.len() as u16)) / 2;
        let row = 2 + lines.len() as u16 + 1;
        stdout.execute(cursor::MoveTo(col, row))?;
        stdout.execute(SetForegroundColor(Color::Green))?;
        writeln!(stdout, "{}", footer)?;
        stdout.execute(ResetColor)?;
    }

    stdout.flush()?;
    Ok(())
}

fn check_git_status() -> String {
    let _ = Command::new("git")
        .args(&["fetch", "--quiet"])
        .output();

    let ahead = Command::new("git")
        .args(&["rev-list", "--count", "origin/main..HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);

    let behind = Command::new("git")
        .args(&["rev-list", "--count", "HEAD..origin/main"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);

    if ahead > 0 {
        format!("[info] You are ahead by {} commit(s)", ahead)
    } else if behind > 0 {
        format!("[info] You are behind by {} commit(s)", behind)
    } else {
        "[info] You are up to date with upstream".to_string()
    }
}

fn main() -> Result<()> {
    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    stdout.execute(Hide)?;

    let mut last_size = terminal::size()?;
    draw(&mut stdout, last_size.0, last_size.1)?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.code == KeyCode::Char('q') => break,
                Event::Key(key_event) if key_event.code == KeyCode::Char('c') => {
                    let size = terminal::size()?;
                    let status = check_git_status();
                    let col = (size.0.saturating_sub(status.len() as u16)) / 2;
                    let row = 2 + ASCII_ART.trim_matches('\n').lines().count() as u16 + 2;
                    stdout.execute(cursor::MoveTo(col, row))?;
                    stdout.execute(SetForegroundColor(Color::Green))?;
                    writeln!(stdout, "{}", status)?;
                    stdout.execute(ResetColor)?;
                    stdout.flush()?;
                }
                Event::Resize(cols, rows) => {
                    last_size = (cols, rows);
                    draw(&mut stdout, cols, rows)?;
                }
                _ => {}
            }
        } else {
            let size = terminal::size()?;
            if size != last_size {
                last_size = size;
                draw(&mut stdout, size.0, size.1)?;
            }
        }
    }

    terminal::disable_raw_mode()?;
    stdout.execute(Show)?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
