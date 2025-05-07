use chrono::{DateTime, Local};
use const_format::formatcp;

use std::{
    io::Write,
    path::MAIN_SEPARATOR,
    sync::{LazyLock, Mutex},
    time::Instant,
};

use crate::context::Context;

pub fn init(args: &Context, start_time: Instant) {
    let logger = Logger::new(args.log_format, start_time);

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(args.log_level.to_level_filter());
    log::info!("System time {}", DateTimeFormat(Local::now()));
}

pub fn init_for_test() {
    static ONCE: LazyLock<()> = LazyLock::new(|| {
        let logger = Logger::new(LogFormat::Complete, Instant::now());

        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
    });
    let _ = &*ONCE;
}

struct Logger {
    format: LogFormat,
    start_time: Instant,
    last_time: Mutex<Instant>,
}

impl Logger {
    fn new(format: LogFormat, start_time: Instant) -> Self {
        Self {
            format,
            start_time,
            last_time: Mutex::new(start_time),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        false
    }

    fn log(&self, record: &log::Record) {
        if self.format.is_simple() {
            println!("{}", record.args());
            return;
        }

        let current_time = Instant::now();
        let since_last;
        {
            let mut last_time = self.last_time.lock().unwrap();
            since_last = current_time.duration_since(*last_time);
            *last_time = current_time;
        }
        let since_last = since_last.as_micros() as f32 / 1000.0;
        let elapsed_time =
            DurationFormat(current_time.duration_since(self.start_time).as_micros() as u64);
        let log_level = format_level(record.level());
        let source_file = SourceFileFormat::new(
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
        );
        let message = format!(
            "{elapsed_time} {log_level} {} {source_file} {since_last:.3}ms\n",
            record.args()
        );

        print!("{}", message);
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum LogFormat {
    Simple,
    Complete,
}

impl LogFormat {
    fn is_simple(&self) -> bool {
        matches!(self, LogFormat::Simple)
    }
    fn is_complete(&self) -> bool {
        matches!(self, LogFormat::Complete)
    }
}

fn format_level(level: log::Level) -> char {
    match level {
        log::Level::Error => 'E',
        log::Level::Warn => 'W',
        log::Level::Info => 'I',
        log::Level::Debug => 'D',
        log::Level::Trace => 'T',
    }
}

struct DurationFormat(u64);

impl std::fmt::Display for DurationFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let millis = self.0 / 1000;
        let second = millis / 1000;
        let minute = second / 60;
        let hour = minute / 60;
        write!(
            f,
            "{}:{:02}:{:02}.{:03},{:03}",
            hour,
            minute % 60,
            second % 60,
            millis % 1000,
            self.0 % 1000
        )
    }
}

struct DateTimeFormat(DateTime<Local>);

impl std::fmt::Display for DateTimeFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let micros = self.0.timestamp_subsec_micros() % 1000_000;
        let millis = micros / 1000;
        write!(
            f,
            "{}.{:03},{:03}",
            self.0.format("%Y-%m-%d %H:%M:%S"),
            millis,
            micros % 1000
        )
    }
}

struct SourceFileFormat<'a> {
    file: &'a str,
    line: u32,
}

impl<'a> SourceFileFormat<'a> {
    fn new(file: &'a str, line: u32) -> Self {
        Self { file, line }
    }
}

impl<'a> std::fmt::Display for SourceFileFormat<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.file;
        let mod_file = formatcp!("{}mod.rs", MAIN_SEPARATOR);
        if name.ends_with(mod_file) {
            name = &name[..name.len() - mod_file.len()];
        }
        if let Some(pos) = name.rfind(MAIN_SEPARATOR) {
            name = &name[pos + 1..];
        }
        write!(f, "{}:{}", name, self.line)
    }
}
