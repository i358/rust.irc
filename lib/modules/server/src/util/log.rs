use chrono::Local;
use crossterm::{
    execute,
    style::{self, Color},
};
use std::io::{self};

fn write(t: &str, color: Color, bold: bool) {
    let bs = "\x1b[1m"; 
    let be = "\x1b[22m"; 
    execute!(io::stdout(), style::SetForegroundColor(color)).unwrap();
    match bold {
        true => print!("{bs}{t}{be} "),
        false =>   print!("{t} ")
    }
    execute!(io::stdout(), style::ResetColor).unwrap();
}

pub fn log(message: String, t: LogType) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    write("⭕", Color::Green, true);
    write(&timestamp, Color::DarkGrey, false);
    match t {
        LogType::INFO => write(
            "[LOG]",
            Color::Rgb {
                r: 150,
                g: 150,
                b: 150,
            },
            true
        ),
        LogType::WARN => write(
            "[WARN]",
            Color::Rgb {
                r: 255,
                g: 70,
                b: 0,
            },
            true
        ),
        LogType::OK => write("[OK]", Color::Green, true),
        LogType::STATUS => write("[STATUS]", Color::Cyan, true),
        LogType::ERROR => write("[ERROR]", Color::Red, true),
    };
    print!("{message}");
  println!("");
}

pub enum LogType {
    INFO,
    WARN,
    ERROR,
    OK,
    STATUS,
}
