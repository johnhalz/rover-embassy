use crate::types::{CalibrationData, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc, RwLock};
use std::sync::Arc;

pub struct ModelCalibrationStorage {
    calibration_data: Arc<RwLock<CalibrationData>>,
    request_rx: mpsc::Receiver<CalibrationRequest>,
    response_tx: mpsc::Sender<CalibrationData>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

#[derive(Debug, Clone)]
pub enum CalibrationRequest {
    Get,
    Update(CalibrationData),
}

impl ModelCalibrationStorage {
    pub fn new(
        request_rx: mpsc::Receiver<CalibrationRequest>,
        response_tx: mpsc::Sender<CalibrationData>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        let default_calibration = CalibrationData {
            wheel_diameter: 0.15,      // 15cm
            wheel_base: 0.30,          // 30cm
            max_speed: 2.0,            // 2 m/s
            max_angular_velocity: 1.5, // 1.5 rad/s
            sensor_offsets: vec![
                [0.20, 0.0, 0.10],   // Front sensor
                [0.0, 0.15, 0.10],   // Left sensor
                [0.0, -0.15, 0.10],  // Right sensor
                [-0.20, 0.0, 0.10],  // Back sensor
            ],
        };

        Self {
            calibration_data: Arc::new(RwLock::new(default_calibration)),
            request_rx,
            response_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "CalibrationStorage",
            LogLevel::Info,
            "Starting calibration storage".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "CalibrationStorage",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(request) = self.request_rx.recv() => {
                    self.handle_request(request).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "CalibrationStorage",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn handle_request(&mut self, request: CalibrationRequest) {
        match request {
            CalibrationRequest::Get => {
                let data = self.calibration_data.read().await.clone();
                let _ = self.response_tx.send(data).await;
            }
            CalibrationRequest::Update(new_data) => {
                let mut data = self.calibration_data.write().await;
                *data = new_data;
                drop(data); // Release lock before await
                let _ = self.log_tx.send(create_log(
                    "CalibrationStorage",
                    LogLevel::Info,
                    "Calibration data updated".to_string()
                )).await;
            }
        }
    }

    pub async fn get_calibration_data(&self) -> CalibrationData {
        self.calibration_data.read().await.clone()
    }
}
