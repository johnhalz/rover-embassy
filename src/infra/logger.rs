use crate::types::{LogEntry, LogLevel};
use chrono::Local;
use crossterm::style::Stylize;
use flatbuffers::FlatBufferBuilder;
use foxglove::{Context, RawChannel, Schema, WebSocketServer, WebSocketServerHandle};
use foxglove_flatbuffers::{
    helpers::{create_log_message, log_schema_bytes},
    LogLevel as FoxgloveLogLevel,
};
use mcap::{Writer, records::MessageHeader};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc};

pub struct Logger {
    log_rx: mpsc::Receiver<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    min_level: LogLevel,
    mcap_writer: Option<Writer<File>>,
    schema_id: u16,
    module_channels: HashMap<String, u16>,
    foxglove_context: Option<Arc<Context>>,
    ws_server_handle: Option<WebSocketServerHandle>,
    ws_channels: HashMap<String, Arc<RawChannel>>,
    message_count: u64,
}

impl Logger {
    pub fn new(log_rx: mpsc::Receiver<LogEntry>, shutdown_rx: broadcast::Receiver<()>) -> Self {
        // Create MCAP file with human-readable timestamp
        let timestamp = Local::now().format("%y%m%d_%H%M%S");
        let filename = format!("log_{}.mcap", timestamp);

        let (mcap_writer, schema_id) = match Self::create_mcap_writer(&filename) {
            Ok(writer_info) => {
                println!(
                    "{} Created MCAP log file: {}",
                    "[Logger]".dark_grey(),
                    filename.magenta().bold()
                );
                println!(
                    "{} Press 'q' to quit gracefully for proper file indexing!",
                    "[Logger]".dark_grey()
                );
                writer_info
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to create MCAP file: {}. Logging disabled.",
                    "[Logger]".dark_grey(),
                    e
                );
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
            foxglove_context: None,
            ws_server_handle: None,
            ws_channels: HashMap::new(),
            message_count: 0,
        }
    }

    fn create_mcap_writer(
        filename: &str,
    ) -> Result<(Option<Writer<File>>, u16), Box<dyn std::error::Error>> {
        let file = File::create(filename)?;

        // Use default options which enable chunking and indexing
        let mut writer = Writer::new(file)?;

        // Read the Foxglove Log FlatBuffer binary schema from foxglove-flatbuffers
        let schema_data = log_schema_bytes();

        // Add schema to MCAP file
        let schema_id = writer.add_schema("foxglove.Log", "flatbuffer", schema_data)?;

        Ok((Some(writer), schema_id))
    }

    async fn create_websocket_server()
    -> Result<(Arc<Context>, WebSocketServerHandle, String), Box<dyn std::error::Error>> {
        let context = Context::new();
        let host = "127.0.0.1";
        let port = 8765;

        let server = WebSocketServer::new()
            .name("RoverOS Logger")
            .bind(host, port)
            .context(&context)
            .start()
            .await?;

        let addr = format!("{}:{}", host, port);

        Ok((context, server, addr))
    }

    pub async fn run(mut self) {
        println!("{} Starting logger module", "[Logger]".dark_grey());

        // Initialize WebSocket server asynchronously
        match Self::create_websocket_server().await {
            Ok((context, server_handle, addr)) => {
                println!(
                    "{} {} {}",
                    "[Logger]".dark_grey(),
                    "Foxglove WebSocket server listening on".green(),
                    format!("ws://{}", addr).cyan().bold()
                );
                self.foxglove_context = Some(context);
                self.ws_server_handle = Some(server_handle);
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to start WebSocket server: {}. Livestreaming disabled.",
                    "[Logger]".dark_grey(),
                    e
                );
            }
        }

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
            println!(
                "{} Finalizing MCAP file with {} messages...",
                "[Logger]".dark_grey(),
                self.message_count
            );
            if let Err(e) = writer.finish() {
                eprintln!(
                    "{} Error finishing MCAP file: {}",
                    "[Logger]".dark_grey(),
                    e
                );
            } else {
                println!(
                    "{} MCAP file finalized successfully (indexed)",
                    "[Logger]".dark_grey()
                );
            }
        }

        println!("{} Stopped", "[Logger]".dark_grey());
    }

    fn log_entry(&mut self, entry: &LogEntry) {
        // Convert SystemTime to nanoseconds since UNIX_EPOCH
        let duration = entry
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp_nanos = duration.as_nanos() as u64;

        // Convert log level
        let fb_level = match entry.level {
            LogLevel::Debug => FoxgloveLogLevel::DEBUG,
            LogLevel::Info => FoxgloveLogLevel::INFO,
            LogLevel::Warn => FoxgloveLogLevel::WARNING,
            LogLevel::Error => FoxgloveLogLevel::ERROR,
        };

        // Build FlatBuffer message using helper function from foxglove-flatbuffers
        let mut builder = FlatBufferBuilder::new();
        let message_data_copy = create_log_message(
            &mut builder,
            entry.timestamp,
            fb_level,
            &entry.message,
            &entry.module,
            None,
            0,
        );

        // Publish to WebSocket FIRST (before MCAP to ensure real-time delivery)
        if let Some(context) = &self.foxglove_context {
            // Get or create WebSocket channel for this module
            if !self.ws_channels.contains_key(&entry.module) {
                // Read the Foxglove Log FlatBuffer binary schema from foxglove-flatbuffers
                let schema_data = log_schema_bytes();

                // Create a new WebSocket channel
                let topic = format!("roverOS/{}", entry.module);
                let schema = Schema::new("foxglove.Log", "flatbuffer", schema_data);
                match context
                    .channel_builder(topic)
                    .schema(schema)
                    .message_encoding("flatbuffer")
                    .build_raw()
                {
                    Ok(channel) => {
                        self.ws_channels.insert(entry.module.clone(), channel);
                    }
                    Err(e) => {
                        eprintln!(
                            "{} Failed to create WebSocket channel for module {}: {}",
                            "[Logger]".dark_grey(),
                            entry.module,
                            e
                        );
                        return;
                    }
                }
            }

            // Publish message to WebSocket
            if let Some(ws_channel) = self.ws_channels.get(&entry.module) {
                ws_channel.log(&message_data_copy);
            }
        }

        // Write to MCAP file
        if let Some(writer) = &mut self.mcap_writer {
            // Get or create MCAP channel for this module
            let channel_id = match self.module_channels.get(&entry.module) {
                Some(&channel_id) => channel_id,
                None => {
                    // Need to create a new channel
                    let topic = format!("roverOS/{}", entry.module);
                    match writer.add_channel(self.schema_id, &topic, "flatbuffer", &BTreeMap::new())
                    {
                        Ok(channel_id) => {
                            self.module_channels
                                .insert(entry.module.clone(), channel_id);
                            channel_id
                        }
                        Err(e) => {
                            eprintln!(
                                "{} Failed to create MCAP channel for module {}: {}",
                                "[Logger]".dark_grey(),
                                entry.module,
                                e
                            );
                            return;
                        }
                    }
                }
            };

            let header = MessageHeader {
                channel_id,
                sequence: 0,
                log_time: timestamp_nanos,
                publish_time: timestamp_nanos,
            };

            if let Err(e) = writer.write_to_known_channel(&header, &message_data_copy) {
                eprintln!("{} Error writing to MCAP: {}", "[Logger]".dark_grey(), e);
            }
        }

        self.message_count += 1;
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Ensure MCAP file is properly finalized when logger is dropped
        if let Some(mut writer) = self.mcap_writer.take() {
            if let Err(e) = writer.finish() {
                eprintln!(
                    "{} Error finishing MCAP file in Drop: {}",
                    "[Logger]".dark_grey(),
                    e
                );
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
