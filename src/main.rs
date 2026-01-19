use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, sleep};

async fn producer(tx: mpsc::Sender<String>, mut shutdown_rx: broadcast::Receiver<()>) {
    let mut counter = 0;
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                println!("[Producer] Shutdown signal received, stopping");
                break;
            }
            _ = sleep(Duration::from_secs(1)) => {
                let message = format!("Message {}", counter);
                println!("[Producer] Sending: {}", message);

                if tx.send(message).await.is_err() {
                    println!("[Producer] Receiver dropped, stopping");
                    break;
                }

                counter += 1;
            }
        }
    }
}

async fn consumer(
    mut rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<String>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                println!("[Consumer] Shutdown signal received, stopping");
                break;
            }
            message = rx.recv() => {
                match message {
                    Some(msg) => {
                        println!("[Consumer] Received: {}", msg);

                        // Process and send to processor
                        let processed = format!("Processed: {}", msg);
                        let _ = tx.send(processed).await;
                    }
                    None => {
                        println!("[Consumer] Channel closed, stopping");
                        break;
                    }
                }
            }
        }
    }
}

async fn processor(mut rx: mpsc::Receiver<String>, mut shutdown_rx: broadcast::Receiver<()>) {
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                println!("[Processor] Shutdown signal received, stopping");
                break;
            }
            message = rx.recv() => {
                match message {
                    Some(msg) => {
                        println!("[Processor] Final: {}", msg);
                    }
                    None => {
                        println!("[Processor] Channel closed, stopping");
                        break;
                    }
                }
            }
        }
    }
}

async fn heartbeat(mut shutdown_rx: broadcast::Receiver<()>) {
    let mut count = 0;
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                println!("[Heartbeat] Shutdown signal received, stopping");
                break;
            }
            _ = sleep(Duration::from_secs(3)) => {
                count += 1;
                println!("[Heartbeat] Alive for {} cycles", count);
            }
        }
    }
}

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
                println!("\n[Input] Shutdown requested, terminating all tasks...");
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
    println!("Tokio async runtime started!");
    println!("Platform: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    println!("Press 'q' and Enter to quit");
    println!();

    // Create channels for inter-task communication
    let (tx1, rx1) = mpsc::channel(32);
    let (tx2, rx2) = mpsc::channel(32);

    // Create shutdown broadcast channel
    let (shutdown_tx, _) = broadcast::channel(1);

    // Spawn tasks with shutdown receivers
    tokio::spawn(producer(tx1, shutdown_tx.subscribe()));
    tokio::spawn(consumer(rx1, tx2, shutdown_tx.subscribe()));
    tokio::spawn(processor(rx2, shutdown_tx.subscribe()));
    tokio::spawn(heartbeat(shutdown_tx.subscribe()));

    // Spawn input listener task
    tokio::spawn(input_listener(shutdown_tx.clone()));

    // Wait for shutdown signal
    let mut shutdown_rx = shutdown_tx.subscribe();
    let _ = shutdown_rx.recv().await;

    // Give tasks a moment to clean up
    sleep(Duration::from_millis(100)).await;
    println!("[Main] All tasks stopped. Goodbye!");
}
