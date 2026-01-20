use crate::types::{LogEntry, LogLevel};
use tokio::sync::{broadcast, mpsc};
use std::time::SystemTime;

pub struct Logger {
    log_rx: mpsc::Receiver<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    min_level: LogLevel,
}

impl Logger {
    pub fn new(
        log_rx: mpsc::Receiver<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            log_rx,
            shutdown_rx,
            min_level: LogLevel::Debug,
        }
    }

    pub async fn run(mut self) {
        println!("[Logger] Starting logger module");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    println!("[Logger] Shutdown signal received");
                    break;
                }
                Some(entry) = self.log_rx.recv() => {
                    if entry.level >= self.min_level {
                        self.print_log(&entry);
                    }
                }
            }
        }

        println!("[Logger] Stopped");
    }

    fn print_log(&self, entry: &LogEntry) {
        let level_str = match entry.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warn => "WARN ",
            LogLevel::Error => "ERROR",
        };

        println!(
            "[{:5}] [{:20}] {}",
            level_str,
            self.truncate_module(&entry.module),
            entry.message
        );
    }

    fn truncate_module(&self, module: &str) -> String {
        if module.len() > 20 {
            format!("{}...", &module[..17])
        } else {
            module.to_string()
        }
    }
}

// Helper function to create log entries easily
pub fn create_log(module: &str, level: LogLevel, message: String) -> LogEntry {
    LogEntry {
        timestamp: SystemTime::now(),
        level,
        module: module.to_string(),
        message,
    }
}
