use colored::Colorize;

pub enum Level {
    Info,
    Warn,
    Error,
}

pub fn log(level: Level, message: &str) {
    let level_msg = match level {
        Level::Info => format!("{}", "[INFO]".cyan()),
        Level::Warn => format!("{}", "[WARNING]".truecolor(214, 143, 0)),
        Level::Error => format!("{}", "[ERROR]".red()),
    };
    println!("{} {}", level_msg, message);
}
