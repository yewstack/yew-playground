use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ansi_to_tui::IntoText as _;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tracing::{debug, info, warn};

fn copy_to_clipboard(text: &str) -> bool {
    use base64::Engine as _;

    // Try wl-copy (Wayland), then xclip (X11), then xsel (X11)
    let commands: &[(&str, &[&str])] = &[
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];
    for (cmd, args) in commands {
        if let Ok(mut child) = Command::new(cmd)
            .args(*args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                use io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            child.stdin.take(); // close stdin
            if let Ok(status) = child.wait()
                && status.success()
            {
                info!("copied via {cmd}");
                return true;
            }
        }
    }

    // Fallback: OSC 52 escape sequence which sets clipboard via the terminal emulator.
    // Supported by most modern terminals (kitty, alacritty, wezterm, foot, ghostty, etc.)
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    // \x1b]52;c;<base64>\x07  — OSC 52, clipboard selection "c", ST with BEL
    let osc = format!("\x1b]52;c;{encoded}\x07");
    use io::Write;
    if let Ok(mut tty) = std::fs::OpenOptions::new().write(true).open("/dev/tty")
        && tty.write_all(osc.as_bytes()).is_ok()
        && tty.flush().is_ok()
    {
        info!("copied via OSC 52");
        return true;
    }

    false
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until we hit a letter (end of escape sequence)
            for c2 in chars.by_ref() {
                if c2.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

const MAX_LINES: usize = 1000;

fn find_open_port(start: u16) -> u16 {
    (start..start + 100)
        .find(|&port| TcpListener::bind(("127.0.0.1", port)).is_ok())
        .unwrap_or_else(|| panic!("no open port found starting from {start}"))
}

struct Service {
    name: &'static str,
    lines: VecDeque<String>,
    child: Option<Child>,
    scroll: u16,
    port: u16,
    copied_flash: u8, // countdown frames to show "copied!" feedback
}

impl Service {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            lines: VecDeque::new(),
            child: None,
            scroll: 0,
            port: 0,
            copied_flash: 0,
        }
    }

    fn push_line(&mut self, line: String) {
        self.lines.push_back(line);
        if self.lines.len() > MAX_LINES {
            self.lines.pop_front();
        }
    }

    fn kill(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
    }

    fn styled_content(&self) -> Text<'static> {
        let mut s = String::new();
        for line in &self.lines {
            s.push_str(line);
            s.push('\n');
        }
        s.as_bytes().into_text().unwrap_or_else(|_| Text::raw(s))
    }

    fn plain_content(&self) -> String {
        let mut s = String::new();
        for line in &self.lines {
            s.push_str(&strip_ansi(line));
            s.push('\n');
        }
        s
    }

    fn tick_flash(&mut self) {
        self.copied_flash = self.copied_flash.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll = self.lines.len().saturating_sub(1) as u16;
    }
}

enum OutputMsg {
    Line(usize, String),
    Exited(usize, Option<i32>),
}

fn pipe_reader(idx: usize, reader: impl io::Read + Send + 'static, tx: mpsc::Sender<OutputMsg>) {
    thread::spawn(move || {
        use io::BufRead;
        let buf = io::BufReader::new(reader);
        for line in buf.lines() {
            match line {
                Ok(l) => {
                    if tx.send(OutputMsg::Line(idx, l)).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn spawn_service(
    cmd: &str,
    args: &[&str],
    env: &[(&str, String)],
    dir: Option<&str>,
    idx: usize,
    tx: &mpsc::Sender<OutputMsg>,
) -> Child {
    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("CLICOLOR_FORCE", "1")
        .env("FORCE_COLOR", "1")
        .env("CARGO_TERM_COLOR", "always");
    for (k, v) in env {
        command.env(k, v);
    }
    if let Some(d) = dir {
        command.current_dir(d);
    }

    let mut child = command.spawn().unwrap_or_else(|e| {
        panic!("failed to spawn {cmd}: {e}");
    });

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    pipe_reader(idx, stdout, tx.clone());
    pipe_reader(idx, stderr, tx.clone());

    let tx2 = tx.clone();
    let pid = child.id();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            let status = Command::new("kill")
                .args(["-0", &pid.to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if status.map(|s| !s.success()).unwrap_or(true) {
                let _ = tx2.send(OutputMsg::Exited(idx, None));
                break;
            }
        }
    });

    child
}

const IDX_BACKEND: usize = 0;
const IDX_FRONTEND: usize = 1;

struct App {
    services: [Service; 2],
    focused: usize,
    rx: mpsc::Receiver<OutputMsg>,
    tx: mpsc::Sender<OutputMsg>,
    project_root: String,
    panel_rects: [Rect; 2],
    bottom_rects: [Rect; 2],
}

impl App {
    fn new(project_root: String) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            services: [Service::new("backend"), Service::new("frontend")],
            focused: 0,
            rx,
            tx,
            project_root,
            panel_rects: [Rect::default(); 2],
            bottom_rects: [Rect::default(); 2],
        }
    }

    fn start_all(&mut self) {
        let backend_port = find_open_port(3000);
        let frontend_port = find_open_port(8080);

        self.services[IDX_BACKEND].port = backend_port;
        self.services[IDX_FRONTEND].port = frontend_port;

        // Backend (merged with compiler)
        self.services[IDX_BACKEND]
            .push_line(format!("--- starting backend on port {backend_port} ---"));
        let app_dir_stable = format!("{}/app", self.project_root);
        let app_dir_next = format!("{}/app-next", self.project_root);
        let child = spawn_service(
            "cargo",
            &[
                "run",
                "--package",
                "backend",
                "--features",
                "simulate-delay",
            ],
            &[
                ("PORT", backend_port.to_string()),
                ("APP_DIR_STABLE", app_dir_stable),
                ("APP_DIR_NEXT", app_dir_next),
                ("SIMULATE_DELAY_SECS", "3".to_string()),
            ],
            Some(&self.project_root),
            IDX_BACKEND,
            &self.tx,
        );
        self.services[IDX_BACKEND].child = Some(child);

        // Frontend: ensure tailwind CSS exists, write dev config, then trunk serve
        self.services[IDX_FRONTEND]
            .push_line(format!("--- starting frontend on port {frontend_port} ---"));
        let frontend_dir = format!("{}/frontend", self.project_root);
        self.services[IDX_FRONTEND].push_line("--- running npm run build:tailwind ---".into());
        let _ = Command::new("npm")
            .args(["run", "build:tailwind"])
            .current_dir(&frontend_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        // Write a dev Trunk config with the correct backend port
        let dev_trunk_toml = format!("{frontend_dir}/Trunk-dev.toml");
        let _ = std::fs::write(
            &dev_trunk_toml,
            format!(
                "[[proxy]]\nbackend = \"http://localhost:{backend_port}/api\"\n\n\
                 [[hook]]\nstage = \"build\"\ncommand = \"npm\"\n\
                 command_arguments = [\"run\", \"build:tailwind\"]\n"
            ),
        );
        let child = spawn_service(
            "trunk",
            &[
                "--color=always",
                "--config",
                &dev_trunk_toml,
                "serve",
                "--port",
                &frontend_port.to_string(),
            ],
            &[],
            Some(&frontend_dir),
            IDX_FRONTEND,
            &self.tx,
        );
        self.services[IDX_FRONTEND].child = Some(child);
    }

    fn kill_all(&mut self) {
        for svc in &mut self.services {
            svc.kill();
        }
    }

    fn drain_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                OutputMsg::Line(idx, line) => {
                    self.services[idx].push_line(line);
                    // Auto-scroll to bottom if already near bottom
                    let svc = &mut self.services[idx];
                    let line_count = svc.lines.len() as u16;
                    if svc.scroll >= line_count.saturating_sub(5) {
                        svc.scroll_to_bottom();
                    }
                }
                OutputMsg::Exited(idx, code) => {
                    self.services[idx]
                        .push_line(format!("--- process exited (code: {code:?}) ---"));
                    self.services[idx].child = None;
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let outer =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(frame.area());
        let panels = Layout::vertical([Constraint::Ratio(1, 2); 2]).split(outer[0]);

        for i in 0..2 {
            self.panel_rects[i] = panels[i];
            // Top border row where copy button lives (right side)
            self.bottom_rects[i] = Rect {
                x: panels[i].x + panels[i].width / 2,
                y: panels[i].y,
                width: panels[i].width / 2,
                height: 1,
            };
        }

        for (i, svc) in self.services.iter().enumerate() {
            let is_focused = i == self.focused;
            let status = if svc.child.is_some() {
                Span::raw(" running ").green()
            } else {
                Span::raw(" stopped ").red()
            };
            let title_left = Line::from(vec![
                Span::raw(format!(" {} ", svc.name)),
                Span::raw(format!(":{} ", svc.port)).dim(),
                status,
            ]);
            let copy_key = i + 1;
            let title_right = if svc.copied_flash > 0 {
                Line::from(" copied! ".green().bold()).right_aligned()
            } else {
                Line::from(vec![
                    Span::raw(format!("{copy_key}")).bold(),
                    Span::raw(" copy ").dim(),
                ])
                .right_aligned()
            };
            let border_style = if is_focused {
                ratatui::style::Style::new().cyan()
            } else {
                ratatui::style::Style::new().dark_gray()
            };
            let block = Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(border_style)
                .title_top(title_left)
                .title_top(title_right);

            let content = svc.styled_content();
            let paragraph = Paragraph::new(content)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((svc.scroll, 0));
            frame.render_widget(paragraph, panels[i]);
        }

        // Status bar
        let help = Line::from(vec![
            " Tab".bold(),
            " switch ".into(),
            " q".bold(),
            " quit ".into(),
            " r".bold(),
            " restart focused ".into(),
            " R".bold(),
            " restart all ".into(),
            " Up/Down".bold(),
            " scroll ".into(),
            " End".bold(),
            " bottom ".into(),
            " 1/2".bold(),
            " copy ".into(),
            " Shift+drag".bold(),
            " select ".into(),
        ])
        .dim();
        frame.render_widget(help, outer[1]);
    }

    fn restart_service(&mut self, idx: usize) {
        self.services[idx].kill();
        self.services[idx].lines.clear();
        self.services[idx].scroll = 0;

        match idx {
            IDX_BACKEND => {
                let port = find_open_port(3000);
                self.services[idx].port = port;
                let app_dir_stable = format!("{}/app", self.project_root);
                let app_dir_next = format!("{}/app-next", self.project_root);
                self.services[idx].push_line(format!("--- restarting backend on port {port} ---"));
                let child = spawn_service(
                    "cargo",
                    &[
                        "run",
                        "--package",
                        "backend",
                        "--features",
                        "simulate-delay",
                    ],
                    &[
                        ("PORT", port.to_string()),
                        ("APP_DIR_STABLE", app_dir_stable),
                        ("APP_DIR_NEXT", app_dir_next),
                        ("SIMULATE_DELAY_SECS", "10".to_string()),
                    ],
                    Some(&self.project_root),
                    idx,
                    &self.tx,
                );
                self.services[idx].child = Some(child);
            }
            IDX_FRONTEND => {
                let backend_port = self.services[IDX_BACKEND].port;
                let port = find_open_port(8080);
                self.services[idx].port = port;
                let frontend_dir = format!("{}/frontend", self.project_root);
                self.services[idx].push_line(format!("--- restarting frontend on port {port} ---"));
                self.services[idx].push_line("--- running npm run build:tailwind ---".into());
                let _ = Command::new("npm")
                    .args(["run", "build:tailwind"])
                    .current_dir(&frontend_dir)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let dev_trunk_toml = format!("{frontend_dir}/Trunk-dev.toml");
                let _ = std::fs::write(
                    &dev_trunk_toml,
                    format!(
                        "[[proxy]]\nbackend = \"http://localhost:{backend_port}/api\"\n\n\
                         [[hook]]\nstage = \"build\"\ncommand = \"npm\"\n\
                         command_arguments = [\"run\", \"build:tailwind\"]\n"
                    ),
                );
                let child = spawn_service(
                    "trunk",
                    &[
                        "--color=always",
                        "--config",
                        &dev_trunk_toml,
                        "serve",
                        "--port",
                        &port.to_string(),
                    ],
                    &[],
                    Some(&frontend_dir),
                    idx,
                    &self.tx,
                );
                self.services[idx].child = Some(child);
            }
            _ => {}
        }
    }

    fn restart_all(&mut self) {
        self.kill_all();
        for svc in &mut self.services {
            svc.lines.clear();
            svc.scroll = 0;
        }
        self.start_all();
    }
}

fn init_tracing() {
    let log_file = File::create("devtool.log").expect("failed to create devtool.log");
    tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "devtool=debug".parse().unwrap()),
        )
        .init();
}

fn main() -> io::Result<()> {
    init_tracing();
    info!("devtool starting");

    let project_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| format!("{d}/.."))
        .unwrap_or_else(|_| ".".to_string());
    let project_root = std::fs::canonicalize(project_root)?
        .to_string_lossy()
        .to_string();

    let mut app = App::new(project_root);
    app.start_all();

    crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;

    let result = ratatui::run(|terminal: &mut ratatui::DefaultTerminal| {
        loop {
            app.drain_messages();
            for svc in &mut app.services {
                svc.tick_flash();
            }
            terminal.draw(|frame| app.draw(frame))?;

            if event::poll(Duration::from_millis(50))? {
                let ev = event::read()?;

                // Handle mouse events
                if let event::Event::Mouse(mouse) = &ev {
                    match mouse.kind {
                        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            let col = mouse.column;
                            let row = mouse.row;
                            debug!(col, row, "mouse click");
                            // Copy button click
                            for i in 0..2 {
                                let r = app.bottom_rects[i];
                                if row == r.y && col >= r.x && col < r.x + r.width {
                                    let text = app.services[i].plain_content();
                                    info!(panel = i, "copy via mouse click");
                                    if copy_to_clipboard(&text) {
                                        app.services[i].copied_flash = 20;
                                    }
                                    break;
                                }
                            }
                        }
                        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                            let row = mouse.row;
                            for i in 0..2 {
                                let r = app.panel_rects[i];
                                if row >= r.y && row < r.y + r.height {
                                    app.focused = i;
                                    break;
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            app.services[app.focused].scroll_up();
                        }
                        MouseEventKind::ScrollDown => {
                            app.services[app.focused].scroll_down();
                        }
                        _ => {}
                    }
                }

                if let event::Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    modifiers,
                    ..
                }) = ev
                {
                    debug!(?code, ?modifiers, "key press");
                    match code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Tab => {
                            app.focused = (app.focused + 1) % 2;
                        }
                        KeyCode::BackTab => {
                            app.focused = (app.focused + 1) % 2;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.services[app.focused].scroll_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.services[app.focused].scroll_down();
                        }
                        KeyCode::End => {
                            app.services[app.focused].scroll_to_bottom();
                        }
                        KeyCode::Char('r') => {
                            app.restart_service(app.focused);
                        }
                        KeyCode::Char('R') => {
                            app.restart_all();
                        }
                        KeyCode::Char(c @ '1'..='2') => {
                            let idx = (c as usize) - ('1' as usize);
                            let text = app.services[idx].plain_content();
                            let len = text.len();
                            info!(panel = idx, char = ?c, text_len = len, "copy requested");
                            if copy_to_clipboard(&text) {
                                info!(panel = idx, "copy succeeded");
                                app.services[idx].copied_flash = 20;
                            } else {
                                warn!(panel = idx, "copy failed - no clipboard command available");
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    });

    let _ = crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture);
    app.kill_all();
    result
}
