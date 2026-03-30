use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    emulators: Vec<Emulator>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Emulator {
    name: String,
    path: String,
    #[serde(default)]
    args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            emulators: vec![
                Emulator {
                    name: "RetroArch".into(),
                    path: "/usr/bin/retroarch".into(),
                    args: vec![],
                },
                Emulator {
                    name: "PCSX2".into(),
                    path: "/usr/bin/pcsx2".into(),
                    args: vec![],
                },
                Emulator {
                    name: "mGBA".into(),
                    path: "/usr/bin/mgba".into(),
                    args: vec![],
                },
                Emulator {
                    name: "Cemu".into(),
                    path: "/usr/bin/cemu".into(),
                    args: vec![],
                },
                Emulator {
                    name: "Dolphin".into(),
                    path: "/usr/bin/dolphin-emu".into(),
                    args: vec![],
                },
            ],
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("emu-launcher")
        .join("config.toml")
}

fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).expect("failed to read config");
        toml::from_str(&content).expect("failed to parse config")
    } else {
        let config = Config::default();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let content = toml::to_string_pretty(&config).expect("failed to serialize config");
        fs::write(&path, &content).ok();
        eprintln!(
            "created default config at {:?} — edit it with your emulator paths!",
            path
        );
        config
    }
}

fn save_config(config: &Config) {
    let path = config_path();
    let content = toml::to_string_pretty(config).expect("failed to serialize config");
    fs::write(&path, content).expect("failed to write config");
}

const HEADER: &str = concat!(
    "      . * . * . * . * . * . * . * . * . * .\r\n",
    "\r\n",
    "      .----------------------------------------.\r\n",
    "      |   * . o   E M U  L A U N C H E R       |\r\n",
    "      |       o . *   pick ur adventure~       |\r\n",
    "      '----------------------------------------'\r\n",
    "\r\n",
    "               /\\_/\\    press enter to launch!\r\n",
    "              ( ^w^ )\r\n",
    "               > - <\r\n",
    "\r\n",
    "      . * . * . * . * . * . * . * . * . * .\r\n",
    "\r\n",
);

fn draw(stdout: &mut io::Stdout, emulators: &[Emulator], selected: usize) -> io::Result<()> {
    queue!(
        stdout,
        Clear(ClearType::Purge),
        MoveTo(0, 0),
        Clear(ClearType::All)
    )?;

    // header
    queue!(
        stdout,
        SetForegroundColor(Color::Magenta),
        SetAttribute(Attribute::Bold),
        Print(HEADER),
        SetAttribute(Attribute::Reset)
    )?;

    // nav hint
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("  up/down or j/k to move  •  enter to launch  •  q to quit\r\n\r\n"),
        ResetColor
    )?;

    // emulator list
    for (i, emu) in emulators.iter().enumerate() {
        if i == selected {
            queue!(
                stdout,
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print(format!("   >  {}\r\n", emu.name)),
                SetAttribute(Attribute::Reset),
                ResetColor
            )?;
        } else {
            queue!(
                stdout,
                SetForegroundColor(Color::White),
                Print(format!("      {}\r\n", emu.name)),
                ResetColor
            )?;
        }
    }

    // add emulator option at the bottom
    if selected == emulators.len() {
        queue!(
            stdout,
            SetForegroundColor(Color::Green),
            SetAttribute(Attribute::Bold),
            Print("   >  + add emulator\r\n"),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
    } else {
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print("      + add emulator\r\n"),
            ResetColor
        )?;
    }

    stdout.flush()
}

// shows a text input prompt and returns what the user typed, or None if they cancelled
fn read_input(stdout: &mut io::Stdout, prompt: &str) -> io::Result<Option<String>> {
    let mut buf = String::new();

    loop {
        queue!(
            stdout,
            MoveTo(0, 14),
            Clear(ClearType::FromCursorDown),
            SetForegroundColor(Color::White),
            Print(format!("  {}: {}_\r\n\r\n", prompt, buf)),
            SetForegroundColor(Color::DarkGrey),
            Print("  enter to confirm  •  esc to cancel\r\n"),
            ResetColor
        )?;
        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => buf.push(c),
                KeyCode::Backspace => { buf.pop(); }
                KeyCode::Enter => return Ok(Some(buf)),
                KeyCode::Esc => return Ok(None),
                _ => {}
            }
        }
    }
}

fn draw_add_screen(stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(
        stdout,
        Clear(ClearType::Purge),
        MoveTo(0, 0),
        Clear(ClearType::All),
        SetForegroundColor(Color::Magenta),
        SetAttribute(Attribute::Bold),
        Print("\r\n  .---------------------------------------.\r\n"),
        Print("  |         ~ add emulator ~             |\r\n"),
        Print("  '---------------------------------------'\r\n\r\n"),
        SetAttribute(Attribute::Reset),
        ResetColor
    )?;
    stdout.flush()
}

fn main() -> io::Result<()> {
    let mut config = load_config();

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut selected = 0usize;

    'outer: loop {
        let total = config.emulators.len() + 1; // +1 for the add option

        if selected >= total {
            selected = total - 1;
        }

        draw(&mut stdout, &config.emulators, selected)?;

        let choice = loop {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                        draw(&mut stdout, &config.emulators, selected)?;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < total - 1 {
                            selected += 1;
                        }
                        draw(&mut stdout, &config.emulators, selected)?;
                    }
                    KeyCode::Enter => break Some(selected),
                    KeyCode::Char('q') | KeyCode::Esc => break None,
                    _ => {}
                }
            }
        };

        let Some(idx) = choice else {
            break 'outer;
        };

        // "add emulator" was selected
        if idx == config.emulators.len() {
            draw_add_screen(&mut stdout)?;

            let name = read_input(&mut stdout, "name")?;
            let Some(name) = name else { continue; };
            if name.is_empty() { continue; }

            draw_add_screen(&mut stdout)?;
            queue!(
                stdout,
                MoveTo(0, 8),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  name: {}\r\n\r\n", name)),
                ResetColor
            )?;
            stdout.flush()?;

            let path = read_input(&mut stdout, "path")?;
            let Some(path) = path else { continue; };
            if path.is_empty() { continue; }

            config.emulators.push(Emulator { name, path, args: vec![] });
            save_config(&config);
            selected = config.emulators.len() - 1;
            continue;
        }

        let emu = &config.emulators[idx];

        // show a waiting screen while the emulator is open
        queue!(
            stdout,
            Clear(ClearType::All),
            MoveTo(0, 0),
            SetForegroundColor(Color::Magenta),
            SetAttribute(Attribute::Bold),
            Print(format!("\r\n  launching {}...\r\n\r\n", emu.name)),
            SetAttribute(Attribute::Reset),
            SetForegroundColor(Color::DarkGrey),
            Print("  close the emulator window to return to the menu\r\n"),
            ResetColor
        )?;
        stdout.flush()?;

        match Command::new(&emu.path)
            .args(&emu.args)
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(e) => {
                queue!(
                    stdout,
                    SetForegroundColor(Color::Red),
                    Print(format!("\r\n  error: {}\r\n", e)),
                    ResetColor
                )?;
                stdout.flush()?;
                let _ = event::read();
            }
        }

        // drain any buffered keypresses before returning to menu
        while event::poll(std::time::Duration::from_millis(0))? {
            let _ = event::read();
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
