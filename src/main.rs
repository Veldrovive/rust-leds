use ws_connector::ws::{Client, ClientConfig};
use serde::{Deserialize};
use serde_json::{Value, json};
use::std::sync::{Mutex, Arc};
use std::{thread, time::Duration};

mod pattern;
use pattern::{Runner, PatternManager, MovingRainbow, SolidTimeVaryingRainbow, GradientPattern, SolidPattern, FadePattern, FadingCrawl};

fn main() {
    let client_config = ClientConfig {
        token: "test-token".to_string(),
        device: "test-device".to_string(),
        app: "room-lights".to_string(),
        url: "ws://108.174.195.143:8000".to_string()
    };

    let mut client = Client::new(client_config).unwrap();

    let mut manager = PatternManager::new(18, 240);
    let arc_manager = Arc::new(Mutex::new(manager));
    let mut runner = Runner::new();
    let runner_arc = Arc::clone(&arc_manager);
    runner.start(runner_arc);

    #[derive(Deserialize)]
    struct Blank {}

    #[derive(Deserialize)]
    struct AddPattern { pattern: String, name: String, args: Value }
    let add_pattern_manager = Arc::clone(&arc_manager);
    client.on("add_pattern".to_string(), move |val: AddPattern| {
        let mut p_manager = add_pattern_manager.lock().unwrap();
        if val.pattern == "moving_rainbow" {
            #[derive(Deserialize)]
            struct MRArgs {tick_rate: u128, saturation: f64, lightness: f64, brightness: f64}
            let args: MRArgs = serde_json::from_value(val.args).unwrap();
            p_manager.add_pattern(val.name, Box::new(MovingRainbow::new(args.tick_rate, args.saturation, args.lightness, args.brightness)));
        } else if val.pattern == "solid_rainbow" {
            #[derive(Deserialize)]
            struct MRArgs {tick_rate: u128, saturation: f64, lightness: f64, brightness: f64}
            let args: MRArgs = serde_json::from_value(val.args).unwrap();
            p_manager.add_pattern(val.name, Box::new(SolidTimeVaryingRainbow::new(args.tick_rate, args.saturation, args.lightness, args.brightness)));
        } else if val.pattern == "gradient" {
            #[derive(Deserialize)]
            struct MRArgs {start_color: [u32; 4], end_color: [u32; 4]}
            let args: MRArgs = serde_json::from_value(val.args).unwrap();
            p_manager.add_pattern(val.name, Box::new(GradientPattern::new(args.start_color, args.end_color)));
        } else if val.pattern == "solid" {
            #[derive(Deserialize)]
            struct MRArgs {color: [u8; 4]}
            let args: MRArgs = serde_json::from_value(val.args).unwrap();
            p_manager.add_pattern(val.name, Box::new(SolidPattern::new(args.color)));
        } else if val.pattern == "fade" {
            #[derive(Deserialize)]
            struct MRArgs {tick_rate: u128, color: [u8; 4], num_dots: usize}
            let args: MRArgs = serde_json::from_value(val.args).unwrap();
            p_manager.add_pattern(val.name, Box::new(FadePattern::new(args.tick_rate, args.color, args.num_dots)));
        } else if val.pattern == "fade_crawl" {
            #[derive(Deserialize)]
            struct MRArgs {tick_rate: u128, tail_len: u128, color: [u8; 4], start_pos: u128}
            let args: MRArgs = serde_json::from_value(val.args).unwrap();
            p_manager.add_pattern(val.name, Box::new(FadingCrawl::new(args.tick_rate, args.tail_len, args.color, args.start_pos)));
        }
        Some(json!(p_manager.get_patterns()).to_string())
    });

    #[derive(Deserialize)]
    struct AddPatterns { patterns: Vec<AddPattern> }
    let add_patterns_manager = Arc::clone(&arc_manager);
    client.on("add_patterns".to_string(), move |val: AddPatterns| {
        println!("Manager Locked to Add Patterns");
        let mut p_manager = add_patterns_manager.lock().unwrap();
        for n_pattern in val.patterns.iter() {
            if n_pattern.pattern == "moving_rainbow" {
                #[derive(Deserialize)]
                struct MRArgs {tick_rate: u128, saturation: f64, lightness: f64, brightness: f64}
                let args: MRArgs = serde_json::from_value(n_pattern.args.clone()).unwrap();
                p_manager.add_pattern(n_pattern.name.clone(), Box::new(MovingRainbow::new(args.tick_rate, args.saturation, args.lightness, args.brightness)));
            } else if n_pattern.pattern == "solid_rainbow" {
                #[derive(Deserialize)]
                struct MRArgs {tick_rate: u128, saturation: f64, lightness: f64, brightness: f64}
                let args: MRArgs = serde_json::from_value(n_pattern.args.clone()).unwrap();
                p_manager.add_pattern(n_pattern.name.clone(), Box::new(SolidTimeVaryingRainbow::new(args.tick_rate, args.saturation, args.lightness, args.brightness)));
            } else if n_pattern.pattern == "gradient" {
                #[derive(Deserialize)]
                struct MRArgs {start_color: [u32; 4], end_color: [u32; 4]}
                let args: MRArgs = serde_json::from_value(n_pattern.args.clone()).unwrap();
                p_manager.add_pattern(n_pattern.name.clone(), Box::new(GradientPattern::new(args.start_color, args.end_color)));
            } else if n_pattern.pattern == "solid" {
                #[derive(Deserialize)]
                struct MRArgs {color: [u8; 4]}
                let args: MRArgs = serde_json::from_value(n_pattern.args.clone()).unwrap();
                p_manager.add_pattern(n_pattern.name.clone(), Box::new(SolidPattern::new(args.color)));
            } else if n_pattern.pattern == "fade" {
                #[derive(Deserialize)]
                struct MRArgs {tick_rate: u128, color: [u8; 4], num_dots: usize}
                let args: MRArgs = serde_json::from_value(n_pattern.args.clone()).unwrap();
                p_manager.add_pattern(n_pattern.name.clone(), Box::new(FadePattern::new(args.tick_rate, args.color, args.num_dots)));
            } else if n_pattern.pattern == "fade_crawl" {
                #[derive(Deserialize)]
                struct MRArgs {tick_rate: u128, tail_len: u128, color: [u8; 4], start_pos: u128}
                let args: MRArgs = serde_json::from_value(n_pattern.args.clone()).unwrap();
                p_manager.add_pattern(n_pattern.name.clone(), Box::new(FadingCrawl::new(args.tick_rate, args.tail_len, args.color, args.start_pos)));
            }
        };
        let ret_val = Some(json!(p_manager.get_patterns()).to_string());
        drop(p_manager);
        println!("Manager to add pattern unlocked");
        return ret_val;
    });

    let clear_patterns_manager = Arc::clone(&arc_manager);
    client.on("clear_patterns".to_string(), move |_: Blank| {
        println!("Manage locked to clear patterns");
        clear_patterns_manager.lock().unwrap().clear();
        Some("Cleared".to_string())
    });

    #[derive(Deserialize)]
    struct SetBrightness { brightness: f64 }
    let brightness_patterns_manager = Arc::clone(&arc_manager);
    client.on("set_brightness".to_string(), move |val: SetBrightness| {
        let brightness = (val.brightness * 255.0) as u8;
        brightness_patterns_manager.lock().unwrap().set_brightness(brightness);
        Some(format!("Set brightness to: {} ({})", brightness, val.brightness).to_string())
    });

    client.start();
}
