use crate::types::{LogEntry, LogLevel};
use crate::infra::foxglove::{Log, LogArgs, LogLevel as FoxgloveLogLevel, Time, TimeArgs};
use tokio::sync::{broadcast, mpsc};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::collections::{BTreeMap, HashMap};
use mcap::{Writer, records::MessageHeader};
use flatbuffers::FlatBufferBuilder;
use chrono::Local;
use crossterm::style::Stylize;

pub struct Logger {
    log_rx: mpsc::Receiver<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    min_level: LogLevel,
    mcap_writer: Option<Writer<File>>,
    schema_id: u16,
    module_channels: HashMap<String, u16>,
    message_count: u64,
}

impl Logger {
    pub fn new(
        log_rx: mpsc::Receiver<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        // Create MCAP file with human-readable timestamp
        let timestamp = Local::now().format("%y%m%d_%H%M%S");
        let filename = format!("log_{}.mcap", timestamp);

        let (mcap_writer, schema_id) = match Self::create_mcap_writer(&filename) {
            Ok(writer_info) => {
                println!("{} Created MCAP log file: {}", "[Logger]".dark_grey(), filename.magenta().bold());
                println!("{} Press 'q' to quit gracefully for proper file indexing!", "[Logger]".dark_grey());
                writer_info
            }
            Err(e) => {
                eprintln!("{} Failed to create MCAP file: {}. Logging disabled.", "[Logger]".dark_grey(), e);
                (None, 0)
            }
        };

        Self {
            log_rx,
            shutdown_rx,
            min_level: LogLevel::Debug,
            mcap_writer,
            schema_id,
            module_channels: HashMap::new(),
            message_count: 0,
        }
    }

    fn create_mcap_writer(filename: &str) -> Result<(Option<Writer<File>>, u16), Box<dyn std::error::Error>> {
        let file = File::create(filename)?;

        // Use default options which enable chunking and indexing
        let mut writer = Writer::new(file)?;

        // Read the Foxglove Log FlatBuffer binary schema
        let schema_data = include_bytes!("../../schemas/Log.bfbs");

        // Add schema to MCAP file
        let schema_id = writer.add_schema(
            "foxglove.Log",
            "flatbuffer",
            schema_data,
        )?;

        Ok((Some(writer), schema_id))
    }

    pub async fn run(mut self) {
        println!("{} Starting logger module", "[Logger]".dark_grey());

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    println!("{} Shutdown signal received", "[Logger]".dark_grey());
                    break;
                }
                Some(entry) = self.log_rx.recv() => {
                    if entry.level >= self.min_level {
                        self.log_entry(&entry);
                    }
                }
            }
        }

        // Finalize MCAP file - write summary section and footer
        if let Some(mut writer) = self.mcap_writer.take() {
            println!("{} Finalizing MCAP file with {} messages...", "[Logger]".dark_grey(), self.message_count);
            if let Err(e) = writer.finish() {
                eprintln!("{} Error finishing MCAP file: {}", "[Logger]".dark_grey(), e);
            } else {
                println!("{} MCAP file finalized successfully (indexed)", "[Logger]".dark_grey());
            }
        }

        println!("{} Stopped", "[Logger]".dark_grey());
    }

    fn log_entry(&mut self, entry: &LogEntry) {
        // Write to MCAP file
        if self.mcap_writer.is_none() {
            return;
        }

        // Get or create channel for this module
        let channel_id = match self.module_channels.get(&entry.module) {
            Some(&channel_id) => channel_id,
            None => {
                // Need to create a new channel
                let topic = format!("roverOS/{}", entry.module);
                let writer = self.mcap_writer.as_mut().unwrap();
                match writer.add_channel(
                    self.schema_id,
                    &topic,
                    "flatbuffer",
                    &BTreeMap::new(),
                ) {
                    Ok(channel_id) => {
                        self.module_channels.insert(entry.module.clone(), channel_id);
                        channel_id
                    }
                    Err(e) => {
                        eprintln!("{} Failed to create channel for module {}: {}", "[Logger]".dark_grey(), entry.module, e);
                        return;
                    }
                }
            }
        };

        // Convert SystemTime to nanoseconds since UNIX_EPOCH for MCAP
        let duration = entry.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp_nanos = duration.as_nanos() as u64;

        // Build FlatBuffer message
        let mut builder = FlatBufferBuilder::new();

        // Create Time message
        let time = Time::create(&mut builder, &TimeArgs {
            sec: duration.as_secs() as u32,
            nsec: duration.subsec_nanos(),
        });

        // Convert log level
        let fb_level = match entry.level {
            LogLevel::Debug => FoxgloveLogLevel::DEBUG,
            LogLevel::Info => FoxgloveLogLevel::INFO,
            LogLevel::Warn => FoxgloveLogLevel::WARNING,
            LogLevel::Error => FoxgloveLogLevel::ERROR,
        };

        // Create strings
        let message_str = builder.create_string(&entry.message);
        let name_str = builder.create_string(&entry.module);

        // Create Log message
        let log = Log::create(&mut builder, &LogArgs {
            timestamp: Some(time),
            level: fb_level,
            message: Some(message_str),
            name: Some(name_str),
            file: None,
            line: 0,
        });

        builder.finish(log, None);
        let message_data = builder.finished_data();

        let header = MessageHeader {
            channel_id,
            sequence: 0,
            log_time: timestamp_nanos,
            publish_time: timestamp_nanos,
        };

        let writer = self.mcap_writer.as_mut().unwrap();
        if let Err(e) = writer.write_to_known_channel(&header, message_data) {
            eprintln!("{} Error writing to MCAP: {}", "[Logger]".dark_grey(), e);
        }

        self.message_count += 1;
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Ensure MCAP file is properly finalized when logger is dropped
        if let Some(mut writer) = self.mcap_writer.take() {
            if let Err(e) = writer.finish() {
                eprintln!("{} Error finishing MCAP file in Drop: {}", "[Logger]".dark_grey(), e);
            }
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
