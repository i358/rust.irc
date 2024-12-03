use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use serde_json::json;
use std::sync::Arc;
use textwrap::wrap;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::select;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser)]
#[command(name = "rust-irc client")]
#[command(author = "i358")]
#[command(version = "1.0")]
#[command(about = "Rust IRC İstemcisi")]
struct Args {
    #[arg(short = 'u', long = "username", default_value = "unknown_user")]
    username: String,
    #[arg(short = 'H', long = "hostname", default_value = "default")]
    host: String,
    #[arg(short = 'p', long = "port", default_value = "0")]
    port: u16,
}

struct App {
    list_state: ListState,
    items: Vec<String>,
    input: String,
    max_wrap_lines: usize,
    width: usize,
}

impl App {
    fn new(items: Vec<String>, width: usize) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        App {
            list_state: state,
            items,
            input: String::new(),
            max_wrap_lines: 4,
            width,
        }
    }

    fn insert(&mut self, item: &str) {
        let wrapped_lines = wrap(item, self.width);
        for line in wrapped_lines {
            self.items.push(line.to_string());
        }
        self.list_state.select(Some(self.items.len() - 1));
    }

    async fn handle_input(
        &mut self,
        key: KeyCode,
        writer_clone: &Arc<Mutex<OwnedWriteHalf>>,
        username: &String,
    ) {
        let wrapped_input = wrap(&self.input, self.width);

        match key {
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.items.len() - 1 {
                        self.list_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Char(c) => {
                if wrapped_input.len() < self.max_wrap_lines {
                    self.input.push(c);
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    let data = json!({
                        "uuid": username,
                        "content": format!("{}", self.input.clone())
                    })
                    .to_string();
                    let mut writer = writer_clone.lock().await;
                    writer
                        .write_all(format!("FN<>::Message {data}\r\n").as_bytes())
                        .await
                        .unwrap();
                    self.input.clear();
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let messages = vec![];

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size()?;
    let width = size.width as usize - 4;

    let mut app = App::new(messages.clone(), width);

    let args = Args::parse();
    let Args {
        mut host,
        mut port,
        username,
    } = args;

    if host == "default" || port == 0 {
        app.insert("log: Herhangi bir sunucu ve port belirtmediniz, varsayılan değerler olan '127.0.0.1:33363' kullanılacak.");
        host = String::from("127.0.0.1");
        port = 33363;
    }
    let addr = format!("{host}:{port}");
    app.insert(format!("log: {addr} sunucusuna bağlanılıyor...").as_str());
    let stream = TcpStream::connect(&addr).await?;
    app.insert(format!("log: {addr} ile bağlantı kuruldu. Sunucu yanıtı bekleniyor..").as_str());
    let (reader, writer) = stream.into_split();

    let reader = BufReader::new(reader);
    let writer = Arc::new(Mutex::new(writer));

    let writer_clone = Arc::clone(&writer);
    let username_clone = username.clone();
    let mut lines = String::new();
    let mut reader = reader;

    loop {
        select! {
            result = reader.read_line(&mut lines) => {
                match result {
                    Ok(0) => {
                        break;
                    },
                    Ok(_) => {
                        if let Some((header, body)) = lines.split_once("::") {
                            let body = body.trim();

                            match header {
                                "MSG" => {
                                    app.insert("log: Sunucudan yanıt alındı. Kimlik doğrulama için başvuru yapılıyor.. Sunucu kimliğiniz doğrulandıktan sonra işleme devam edilecek.");
                                    let data = json!({
                                        "username": username_clone,
                                        "pem": "x"
                                    })
                                    .to_string();
                                    let mut writer = writer_clone.lock().await;
                                    writer
                                        .write_all(format!("FN<>::Identify {data}\r\n").as_bytes())
                                        .await?;
                                    writer.flush().await?;
                                }
                                "OK" => {
                                   
                                    if body.contains("Connection Established") {
                                        app.insert("log: Sunucu tarafından kimlik doğrulama işlemi onaylandı. Bağlantı kuruldu, sunucu tarafından kullanıcı ID'si atanması bekleniyor.");
                                    }
                                }
                                "UUID" => {
                                    app.insert(format!("log: Sunucu tarafından kullanıcı ID'si atandı: {}", body).as_str());
                                    app.insert(format!("log: Artık mesajlaşmaya hazırsın, {}!", username).as_str());
                                }
                                "UMSG" => {
                                    if body.starts_with("JOIN"){
                                        app.insert(format!("join: {}", &body[6..]).as_str());
                                    } else {
                                        app.insert(format!("{body}").as_str());
                                    }
                                }
                                _ => {}
                            }
                        }
                        lines.clear();
                    },
                    Err(e) => {
                        app.insert(format!("err: Sunucu kaynaklı bir hatadan dolayı bağlantı koptu: {e}").as_str());
                        break;
                    }
                }
            },
            _ = tokio::task::spawn_blocking(move || event::poll(std::time::Duration::from_millis(100))) => {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        if key.code == KeyCode::Esc {
                            break;
                        } else {
                            app.handle_input(key.code, &writer_clone, &username).await;
                        }
                    }
                }
            },
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(size);

            let wrapped_items: Vec<ListItem> = app
                .items
                .iter()
                .map(|item| {
                    let parts: Vec<&str> = item.splitn(2, ": ").collect();
                    let spans = if parts.len() == 2 {
                        Spans::from(vec![
                            Span::styled(
                                parts[0],
                                Style::default()
                                    .fg(match parts[0] {
                                        "log" => Color::LightYellow,
                                        "error" => Color::LightRed,
                                        "join" => Color::LightGreen,
                                        _ => {
                                            if parts[0] == username {
                                                Color::LightCyan
                                            } else {
                                                Color::LightRed
                                            }
                                        },
                                    })
                                    .add_modifier(tui::style::Modifier::BOLD),
                            ),
                            Span::raw(": "),
                            Span::raw(parts[1]),
                        ])
                    } else {
                        Spans::from(vec![Span::raw(item)])
                    };

                    ListItem::new(vec![spans])
                })
                .collect();

            let list = List::new(wrapped_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("-• Mesajlar ~ [{addr}] •-"))
                        .title_alignment(tui::layout::Alignment::Center)
                        .border_type(tui::widgets::BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(150, 200, 255))),
                )
                .highlight_style(Style::default().bg(Color::Rgb(0, 50, 100)))
                .highlight_symbol(" ");

            f.render_stateful_widget(list, chunks[0], &mut app.list_state);

            let wrapped_input = wrap(&app.input, app.width);
            let input_paragraph = Paragraph::new(
                wrapped_input
                    .iter()
                    .map(|line| Spans::from(Span::raw(line.to_string())))
                    .collect::<Vec<_>>(),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("-• Bir mesaj yaz ~ {username}@[{addr}] •-"))
                    .title_alignment(tui::layout::Alignment::Left)
                    .border_type(tui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(150, 200, 255))),
            );

            f.render_widget(input_paragraph, chunks[1]);
        })?;
    }

    Ok(())
}
