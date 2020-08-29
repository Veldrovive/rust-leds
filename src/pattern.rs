use::rs_ws281x::{ChannelBuilder, StripType, ControllerBuilder, Controller};
use::std::{time, thread};
use::std::sync::{Mutex, Arc};
use hsl::HSL;
use std::collections::HashMap;

extern crate rand;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::time::SystemTime;

pub struct MovingRainbow {
    pub tick_rate: u128,
    pub tick_cycle: Option<u128>,
    pub saturation: f64,
    pub lightness: f64,
    pub brightness: f64,
    color: HSL,
}

impl MovingRainbow {
    pub fn new(tick_rate: u128, saturation: f64, lightness: f64, brightness: f64) -> MovingRainbow {
        MovingRainbow {
            brightness,
            tick_rate,
            tick_cycle: None,
            saturation,
            lightness,
            color: HSL{
                h: 0.0,
                s: saturation,
                l: lightness
            },
        }
    }
}

impl Pattern for MovingRainbow {
    fn tick_rate(&self) -> u128 { self.tick_rate }
    fn tick_cycle(&self) -> Option<u128> { self.tick_cycle }
    fn tick(&mut self, tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        for i in 0..leds.len() {
            self.color.h = ( (tick as usize + i) % leds.len() * 360 / leds.len() ) as f64;
            let rgb = self.color.to_rgb();
            let r_val = (rgb.0 as f64 * self.brightness) as u8;
            let g_val = (rgb.1 as f64 * self.brightness) as u8;
            let b_val = (rgb.2 as f64 * self.brightness) as u8;
            leds[i as usize] = [b_val, g_val, r_val, 0];
        }
        true
    }
}

pub struct SolidTimeVaryingRainbow {
    pub tick_rate: u128,
    pub tick_cycle: Option<u128>,
    pub saturation: f64,
    pub lightness: f64,
    pub brightness: f64,
    color: HSL,
    logged: bool
}

impl SolidTimeVaryingRainbow {
    pub fn new(tick_rate: u128, saturation: f64, lightness: f64, brightness: f64) -> SolidTimeVaryingRainbow {
        SolidTimeVaryingRainbow{
            brightness,
            tick_rate,
            tick_cycle: Some(359),
            saturation,
            lightness,
            color: HSL{
                h: 0.0,
                s: saturation,
                l: lightness
            },
            logged: false
        }
    }
}

impl Pattern for SolidTimeVaryingRainbow {
    fn tick_rate(&self) -> u128 { self.tick_rate }
    fn tick_cycle(&self) -> Option<u128> { self.tick_cycle }
    fn tick(&mut self, tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        if !self.logged {
            self.logged = true;
        }
        self.color.h = tick as f64;
        let rgb = self.color.to_rgb();
        let r_val = (rgb.0 as f64 * self.brightness) as u8;
        let g_val = (rgb.1 as f64 * self.brightness) as u8;
        let b_val = (rgb.2 as f64 * self.brightness) as u8;
        for i in 0..leds.len() {
            leds[i as usize] = [b_val, g_val, r_val, 0];
        }
        true
    }
}

pub struct GradientPattern {
    pub tick_rate: u128,
    pub tick_cycle: Option<u128>,
    pub start_color: [u32; 4],
    pub end_color: [u32; 4],
    rerender: bool
}

impl GradientPattern {
    pub fn new(start_color: [u32; 4], end_color: [u32; 4]) -> GradientPattern {
        GradientPattern {
            tick_rate: 60,
            tick_cycle: None,
            start_color,
            end_color,
            rerender: false
        }
    }
}

impl Pattern for GradientPattern {
    fn tick_rate(&self) -> u128 { self.tick_rate }
    fn tick_cycle(&self) -> Option<u128> { self.tick_cycle }
    fn tick(&mut self, _tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        if !self.rerender {
            for i in 0..leds.len() {
                for j in 0..3 {
                    leds[i as usize][j] = ((self.start_color[j] * (leds.len()-i) as u32 + self.end_color[j] * i as u32) / leds.len() as u32) as u8
                }
            }
            return true
        }
        true
    }
}

pub struct SolidPattern {
    pub tick_rate: u128,
    pub tick_cycle: Option<u128>,
    pub color: [u8; 4],
    rerender: bool
}

impl SolidPattern {
    pub fn new(color: [u8; 4]) -> SolidPattern {
        SolidPattern {
            tick_rate: 60, 
            color,
            tick_cycle: None,
            rerender: false
        }
    }

    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
        self.rerender = false;
    }
}

impl Pattern for SolidPattern {
    fn tick_rate(&self) -> u128 { self.tick_rate }
    fn tick_cycle(&self) -> Option<u128> { self.tick_cycle }
    fn tick(&mut self, _tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        if !self.rerender {
            for i in 0..leds.len() {
                leds[i as usize] = self.color;
            }
            return true
        }
        true
    }
}

pub struct FadePattern {
    pub tick_rate: u128,
    pub tick_cycle: Option<u128>,
    color: [u8; 4],
    lights: Vec<(usize, f32)>,
    num_dots: usize,
    rand: StdRng
}

#[allow(dead_code)]
impl FadePattern {
    pub fn new(tick_rate: u128, color: [u8; 4], num_dots: usize) -> FadePattern {
        FadePattern {
            tick_rate,
            tick_cycle: None,
            color,
            num_dots,
            rand: StdRng::from_entropy(),
            lights: Vec::new()
        }
    }
}

impl Pattern for FadePattern {
    fn tick_rate(&self) -> u128 { self.tick_rate }
    fn tick_cycle(&self) -> Option<u128> { self.tick_cycle }
    fn tick(&mut self, _tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        if self.lights.len() < self.num_dots {
            let pos: usize = self.rand.gen_range(0, leds.len());
            let initial_brightness: f32 = self.rand.gen_range(0.1, 1.0);
            self.lights.push((pos, initial_brightness));
        }

        for (pos, brightness) in self.lights.iter_mut() {
            *brightness -= 0.05;
            if *brightness > 0.0 {
                let b_val: u8 = (self.color[0] as f32 * *brightness) as u8;
                let g_val: u8 = (self.color[1] as f32 * *brightness) as u8;
                let r_val: u8 = (self.color[2] as f32 * *brightness) as u8;
                leds[*pos] = [b_val, g_val, r_val, 0];
            } else {
                leds[*pos] = [0, 0, 0, 0];
            }
        }

        self.lights.retain(|elem| elem.1 > 0.0);

        true
    }
}

pub struct FadingCrawl {
    tick_rate: u128,
    tick_cycle: Option<u128>,
    tail_len: u128,
    color: [u8; 4],
    start_pos: u128
}

#[allow(dead_code)]
impl FadingCrawl {
    pub fn new(tick_rate: u128, tail_len: u128, color: [u8; 4], start_pos: u128) -> FadingCrawl {
        FadingCrawl {
            tick_rate,
            tick_cycle: None,
            tail_len,
            color,
            start_pos
        }
    }
}

impl Pattern for FadingCrawl {
    fn tick_rate(&self) -> u128 { self.tick_rate }
    fn tick_cycle(&self) -> Option<u128> { self.tick_cycle }
    fn tick(&mut self, tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        let calc_tick = tick + self.tail_len + self.start_pos;

        for i in 0..self.tail_len {
            let led = (calc_tick - i) % leds.len() as u128;
            let b_brightness = self.color[0] / (i + 1) as u8;
            let g_brightness = self.color[1] / (i + 1) as u8;
            let r_brightness = self.color[2] / (i + 1) as u8;
            
            leds[led as usize] = [b_brightness, g_brightness, r_brightness, 0];
        }
        let led = (calc_tick - self.tail_len) % leds.len() as u128;
        leds[led as usize] = [0, 0, 0, 0];

        true
    }
}

pub trait Pattern: Send {
    fn tick_rate(&self) -> u128;
    fn tick_cycle(&self) -> Option<u128>;
    fn start_tick(&mut self, raw_tick: u128, leds: &mut Vec<[u8; 4]>) -> bool {
        match self.tick_cycle() {
            Some(cycle) => self.tick(raw_tick % cycle, leds),
            None => self.tick(raw_tick, leds)
        }
    }
    fn tick(&mut self, tick: u128, leds: &mut Vec<[u8; 4]>) -> bool;  // Returns whether to re-render this one
    fn elapsed_to_raw_tick(&self, elapsed: u128) -> u128 {
        // Gets the current tick based on the time elapsed in milliseconds
        (self.tick_rate() * elapsed) / 1000
    }
}

pub struct Runner {
}

impl Runner {
    pub fn new() -> Runner {
        let runner = Runner {
            
        };
        runner
    }

    pub fn start(&mut self, threaded_manager: Arc<Mutex<PatternManager>>) {
        thread::spawn(move || {
            // let mut lock_count = 0;
            let mut fail_count = 0;
            let manager = threaded_manager.lock().unwrap();
            let mut sleep_time = manager.sleep_time;
            drop(manager);
            loop {
                let mut lock = threaded_manager.try_lock();
                if let Ok(ref mut mutex) = lock {
                    // println!("Got manager: {}", lock_count);
                    // lock_count += 1;
                    if (*mutex).sleep_time != sleep_time {
                        sleep_time = (*mutex).sleep_time;
                    }
                    (*mutex).increment_ticks();
                    drop(mutex);
                } else {
                    // println!("Manager was locked: {}", fail_count);
                    fail_count += 1;
                }
                thread::sleep(sleep_time);
            }
        });
    }
}

struct PatternStore {
    pattern: Box<dyn Pattern>,
    leds: Vec<[u8; 4]>,
    curr_tick: u128,
    start_time: time::Instant
}

pub struct PatternManager {
    creation_time: time::Instant,
    sleep_time: time::Duration,
    patterns: HashMap<String, PatternStore>,
    controller: Controller,
    num_leds: i32,
}

impl PatternManager {
    pub fn new(pin: i32, led_count: i32) -> PatternManager {
        let channel = ChannelBuilder::new()
            .pin(pin)
            .count(led_count)
            .strip_type(StripType::Ws2811Grb)
            .brightness(255)
            .build();

        let controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(0, channel)
            .build()
            .expect("Failed to create controllerBuilder");

        let pattern_manager = PatternManager {
            creation_time: time::Instant::now(),
            sleep_time: time::Duration::from_micros(1_000),
            patterns: HashMap::new(),
            controller: controller,
            num_leds: led_count,
        };
        pattern_manager
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        self.controller.set_brightness(0, brightness);
    }

    pub fn add_pattern(&mut self, name: String, pattern: Box<dyn Pattern>) {
        let store = PatternStore {
            pattern: pattern,
            leds: vec![[0, 0, 0, 0]; self.num_leds as usize],
            curr_tick: 0,
            start_time: time::Instant::now(),
        };
        self.patterns.insert(name, store);
    }

    pub fn remove_pattern(&mut self, name: String) -> bool {
        if self.patterns.contains_key(&name) {
            self.patterns.remove(&name);
            return true;
        } else {
            return false;
        }
    }

    pub fn clear(&mut self) {
        self.patterns.clear();
        self.tick();
    }

    pub fn get_patterns(&mut self) -> Vec<String> {
        self.patterns.keys().map(|key| key.clone()).collect()
    }

    pub fn increment_ticks(&mut self) {
        let mut got_update = false;
        
        for (_name, pattern_holder) in self.patterns.iter_mut(){
            let elapsed = pattern_holder.start_time.elapsed().as_millis();
            let old_tick = pattern_holder.curr_tick;
            let curr_tick = pattern_holder.pattern.elapsed_to_raw_tick(elapsed);

            if curr_tick > old_tick {
                let leds = &mut pattern_holder.leds;
                // Only run if there is going to be an update
                if curr_tick - old_tick > 1 {
                    println!("Catching up on {} by {} ticks", _name, curr_tick-old_tick);
                }
                for j in old_tick..curr_tick {
                    // This runs the number of times that the pattern should tick
                    if pattern_holder.pattern.start_tick(j + 1, leds) {
                        got_update = true;
                    }
                }
                pattern_holder.curr_tick = curr_tick;
            }
        }

        if got_update {
            // Then we need to render the controller
            self.tick();
        };
    }

    pub fn tick(&mut self) {
        let leds = self.controller.leds_mut(0);

        for i in 0..self.num_leds {
            let mut led: [u32; 4] = [0, 0, 0, 0];
            for (_name, pattern_manager) in self.patterns.iter() {
                for l in 0..4 {
                    led[l as usize] += pattern_manager.leds[i as usize][l as usize] as u32; // / len;
                }
            }

            if false {
                // let max = leds[i as usize].iter().max().expect("Could not get max for brightness");
                let mut max = 0;
                let brightness: i32 = 255;
                for k in 0..3 {
                    if led[k] > max { max = led[k] };
                }
                if max > 0 {
                    for k in 0..3 {
                        let val = led[k] as u32 * brightness as u32 / max as u32;
                        led[k] = val;
                    }
                }
            } else {
                let mut max = 0;
                for k in 0..3 {
                    if led[k] > max { max = led[k] };
                }
                if max > 255 {
                    for k in 0..3 {
                        let val = led[k] as u32 * 255 as u32 / max as u32;
                        led[k] = val;
                    }
                }
            }
            for l in 0..3 {
                leds[i as usize][l] = led[l] as u8;
            }
        }

        self.controller.render()
            .expect("Failed to render leds");
    }
}
