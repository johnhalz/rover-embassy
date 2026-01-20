use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rover_embassy::RoverSystem;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

async fn input_listener(shutdown_tx: broadcast::Sender<()>) {
    // Enable raw mode to read key presses without Enter
    if let Err(e) = enable_raw_mode() {
        eprintln!("[Input] Failed to enable raw mode: {}", e);
        return;
    }

    loop {
        // Poll for events in a non-blocking way
        if let Ok(true) = crossterm::event::poll(Duration::from_millis(100)) {
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Char('Q'),
                kind: KeyEventKind::Press,
                ..
            })) = crossterm::event::read()
            {
                // Disable raw mode before shutting down
                let _ = disable_raw_mode();
                println!("\n[Main] Shutdown requested, terminating all modules...");
                let _ = shutdown_tx.send(());
                break;
            }
        }
    }

    // Ensure raw mode is disabled on exit
    let _ = disable_raw_mode();
}

#[tokio::main]
async fn main() {
    // Create and initialize the rover system
    let mut rover = RoverSystem::new();

    // Get shutdown transmitter before initializing
    let shutdown_tx = rover.shutdown_tx();

    // Initialize and start all modules
    rover.initialize_and_run().await;

    // Spawn input listener
    tokio::spawn(input_listener(shutdown_tx.clone()));

    // Wait for shutdown signal
    let mut shutdown_rx = shutdown_tx.subscribe();
    let _ = shutdown_rx.recv().await;

    // Give tasks a moment to clean up
    sleep(Duration::from_millis(200)).await;

    // Wait for all tasks to complete
    rover.wait_for_completion().await;

    println!("\n[Main] All modules stopped. Goodbye!");
}
