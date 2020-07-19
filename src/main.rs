use ws_connector::ws::{Client, ClientConfig};
use serde::{Deserialize};
use::std::sync::{Mutex, Arc};
use std::{thread, time::Duration};

mod pattern;
use pattern::{Runner, PatternManager, MovingRainbow};

fn main() {
    let client_config = ClientConfig {
        token: "test-token".to_string(),
        device: "test-device".to_string(),
        app: "rust-app".to_string(),
        url: "ws://10.8.0.14:8000".to_string()
    };

    let mut client = Client::new(client_config).unwrap();

    #[derive(Deserialize)]
    struct Value { value: i32 }
    client.on("do something".to_string(), |val: Value| {
        println!("Doing something");
        Some("Test Response".to_string())
    });

    let mut manager = PatternManager::new(18, 240);
    let arc_manager = Arc::new(Mutex::new(manager));
    let mut runner = Runner::new();
    let runner_arc = Arc::clone(&arc_manager);
    runner.start(runner_arc);

    let thread_manager = Arc::clone(&arc_manager);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(3000));
        let test_pattern = MovingRainbow::new(1, 0.9, 0.9, 0.9);
        thread_manager.lock().unwrap().add_pattern("test_pattern".to_string(), Box::new(test_pattern));
    });

    client.start();
}
