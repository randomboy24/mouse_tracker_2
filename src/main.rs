use eframe::egui;
use std::{
    fs::OpenOptions,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use rdev::{listen, Event, EventType};

struct App {
    input_text: String,
    tracking: bool,
    events: Arc<Mutex<Vec<String>>>,
    tracking_flag: Arc<Mutex<bool>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            tracking: false,
            events: Arc::new(Mutex::new(Vec::new())),
            tracking_flag: Arc::new(Mutex::new(false)),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Create a task:");
                ui.text_edit_singleline(&mut self.input_text);
            });

            if self.tracking {
                if ui.button("Stop").clicked() {
                    self.tracking = false;
                    *self.tracking_flag.lock().unwrap() = false;
                    self.save_events();
                }
            } else {
                if ui.button("Create").clicked() && !self.input_text.is_empty() {
                    self.start_tracking();
                }
            }
        });
    }
}

impl App {
    fn start_tracking(&mut self) {
        if self.tracking {
            return; // Prevent multiple tracking sessions
        }
        
        self.tracking = true;
        {
            let mut tracking_flag = self.tracking_flag.lock().unwrap();
            *tracking_flag = true;
        }
        
        {
            let mut events = self.events.lock().unwrap();
            events.clear();
        }

        let events = Arc::clone(&self.events);
        let tracking_flag = Arc::clone(&self.tracking_flag);
        let task_name = self.input_text.clone();

        // Mouse event listener
        let tracking_flag_listener = Arc::clone(&tracking_flag);
        thread::spawn(move || {
            listen(move |event: Event| {
                if !*tracking_flag_listener.lock().unwrap() {
                    return;
                }

                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();

                let event_string = match event.event_type {
                    EventType::MouseMove { x, y } => {
                        format!("{},{},MouseMove,{:.3},{:.3}", task_name, timestamp, x, y)
                    }
                    EventType::ButtonPress(button) => {
                        format!("{},{},ButtonPress,{:?}", task_name, timestamp, button)
                    }
                    EventType::ButtonRelease(button) => {
                        format!("{},{},ButtonRelease,{:?}", task_name, timestamp, button)
                    }
                    _ => return,
                };

                let mut data = events.lock().unwrap();
                data.push(event_string);
            })
            .unwrap();
        });

        // Timer thread to stop tracking after 10 seconds
        let tracking_flag_timer = Arc::clone(&tracking_flag);
        let events = Arc::clone(&self.events);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            *tracking_flag_timer.lock().unwrap() = false;
            
            let data = {
                let data = events.lock().unwrap();
                data.clone() // Clone once to minimize lock duration
            };

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("mouse_tracking_data.csv")
                .unwrap();

            for line in data.iter() {
                writeln!(file, "{}", line).unwrap();
            }
        });
    }

    fn save_events(&self) {
        let data = {
            let data = self.events.lock().unwrap();
            data.clone()
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("mouse_tracking_data.csv")
            .unwrap();

        for line in data.iter() {
            writeln!(file, "{}", line).unwrap();
        }
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mouse Tracker",
        options,
        Box::new(|_| Ok(Box::new(App::default()))),
    )
    .unwrap();
}
