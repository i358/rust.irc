use colored::*;
use chrono::Local;

pub fn log(message: String, t:LogType) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let t_formatted = match t {
        LogType::INFO =>  "[LOG]".bold().truecolor(150, 150, 150),
        LogType::WARN =>  "[WARN]".bold().truecolor(255, 70, 0),
        LogType::OK =>    "[OK]".bold().green(),
        LogType::STATUS => "[STATUS]".bold().cyan(),
        LogType::ERROR => "[ERROR]".bold().red()
    };
    let icon = match t {
        LogType::INFO => "⭕".bold().bright_black(),
        LogType::WARN => "⭕".bold().truecolor(255, 70, 0),
        LogType::ERROR => "⭕".bold().red(),
        LogType::STATUS => "⭕".bold().cyan(),
        LogType::OK => "⭕".bold().green()
    };

    let gray_timestamp = timestamp.dimmed();

    println!("{icon} {} {t_formatted} {}", gray_timestamp, message);
}

pub enum LogType {
    INFO,
    WARN,
    ERROR,
    OK,
    STATUS,
}