use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// ============================================================================
// Sensor Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub timestamp: SystemTime,
    pub distance_sensors: Vec<f32>, // Distance readings in meters
    pub imu: ImuData,
    pub gps: GpsData,
    pub battery_level: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImuData {
    pub acceleration: [f32; 3], // x, y, z in m/s^2
    pub gyroscope: [f32; 3],    // roll, pitch, yaw in rad/s
    pub orientation: [f32; 4],  // quaternion [w, x, y, z]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub accuracy: f32, // meters
}

// ============================================================================
// User Input Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserCommand {
    ManualControl(ManualControl),
    MissionCommand(MissionCommand),
    SystemCommand(SystemCommand),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManualControl {
    MoveForward(f32),  // Speed 0.0 to 1.0
    MoveBackward(f32),
    TurnLeft(f32),     // Angular velocity
    TurnRight(f32),
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionCommand {
    GoToWaypoint { lat: f64, lon: f64 },
    FollowPath(Vec<Waypoint>),
    Patrol { waypoints: Vec<Waypoint>, loops: u32 },
    ReturnHome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub lat: f64,
    pub lon: f64,
    pub tolerance: f32, // meters
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemCommand {
    Pause,
    Resume,
    EmergencyStop,
    Calibrate,
}

// ============================================================================
// Robot State Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotPose {
    pub position: [f32; 3],    // x, y, z in meters
    pub orientation: [f32; 4], // quaternion [w, x, y, z]
    pub velocity: [f32; 3],    // linear velocity
    pub angular_velocity: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RobotState {
    Idle,
    ManualControl,
    ExecutingMission,
    Paused,
    EmergencyStop,
    Error(String),
}

// ============================================================================
// Environment Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub obstacles: Vec<Obstacle>,
    pub terrain_type: TerrainType,
    pub confidence: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    pub position: [f32; 3],
    pub size: [f32; 3], // width, height, depth
    pub obstacle_type: ObstacleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObstacleType {
    Static,
    Dynamic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainType {
    Flat,
    Rough,
    Steep,
    Unknown,
}

// ============================================================================
// Mission/Task Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: u64,
    pub name: String,
    pub tasks: Vec<Task>,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Navigate(Waypoint),
    Scan,
    Wait(u64), // milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

// ============================================================================
// Planning Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub target_pose: RobotPose,
    pub goal_type: GoalType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    ReachPosition,
    OrientTowards,
    FollowTrajectory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub waypoints: Vec<RobotPose>,
    pub total_distance: f32,
    pub estimated_time: f32, // seconds
}

// ============================================================================
// Stance/Posture Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StanceConfig {
    pub stance_type: StanceType,
    pub stability: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StanceType {
    Normal,
    LowProfile,    // For obstacles
    HighClearance, // For rough terrain
    TiltCompensation(f32), // Angle in radians
}

// ============================================================================
// Behavior Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorCommand {
    pub timestamp: SystemTime,
    pub behavior: Behavior,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Behavior {
    Idle,
    MoveTowards { target: [f32; 3], speed: f32 },
    AvoidObstacle { direction: [f32; 3] },
    AdjustStance(StanceConfig),
    EmergencyStop,
}

// ============================================================================
// Hardware Control Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorCommand {
    pub left_speed: f32,  // -1.0 to 1.0
    pub right_speed: f32, // -1.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatus {
    pub timestamp: SystemTime,
    pub battery_voltage: f32,
    pub motor_temps: Vec<f32>,
    pub health: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning(String),
    Critical(String),
}

// ============================================================================
// Calibration Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub wheel_diameter: f32,      // meters
    pub wheel_base: f32,          // meters between wheels
    pub max_speed: f32,           // m/s
    pub max_angular_velocity: f32, // rad/s
    pub sensor_offsets: Vec<[f32; 3]>,
}

// ============================================================================
// Logging Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// ============================================================================
// Communication Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub timestamp: SystemTime,
    pub state: RobotState,
    pub pose: RobotPose,
    pub current_mission: Option<String>,
    pub battery_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub message: String,
    pub feedback_type: FeedbackType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    Status,
    Warning,
    Error,
    Success,
}
