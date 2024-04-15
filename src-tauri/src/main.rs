// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod chipst8;

use std::fs::File;
use std::io::Read;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use serde::{Deserialize, Serialize};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::{generate_context, Manager};

use tauri_plugin_dialog::DialogExt;

use crate::chipst8::Chipst8;

#[derive(Clone, Serialize)]
struct ScreenPayload {
    //   #[serde(serialize_with = "<[_]>::serialize")]
    screen: Vec<Vec<bool>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyPayload {
    key: usize,
    press: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct BeepPayload {
    beep: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct SpeedPayload {
    speed: i64,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let (display_tx, display_rx) = channel();
            let (beep_tx, beep_rx) = channel();

            let emu = Arc::new(Mutex::new(Chipst8::new(display_tx, beep_tx)));
            let emu_load = emu.clone();

            let keys = Arc::new(RwLock::new([false; 16]));

            {
                let handle = app.handle().clone();
                thread::spawn(move || loop {
                    match display_rx.recv() {
                        Ok(display) => {
                            let payload = ScreenPayload {
                                screen: display.iter().map(|row| row.to_vec()).collect(),
                            };
                            match handle.emit("draw", payload) {
                                Ok(_) => continue,
                                Err(e) => eprintln!("receive display: {e}"),
                            }
                        }
                        Err(e) => eprintln!("channel receive: {e}"),
                    }
                });
            }

            {
                let handle = app.handle().clone();
                let emu = emu.clone();
                handle.listen_any("speed", move |event| {
                    println!("{:?}", event.payload());

                    let payload: SpeedPayload = match serde_json::from_str(event.payload()) {
                        Ok(p) => p,
                        Err(e) => {
                            println!("{:?}", e);
                            return;
                        }
                    };

                    match emu.lock() {
                        Ok(mut emu) => {
                            if payload.speed > 0 {
                                emu.speedup();
                            } else {
                                emu.speeddown();
                            }
                        },
                        Err(e) => eprintln!("speed: {e}"),
                    };
                });
            }

            {
                let handle = app.handle().clone();
                thread::spawn(move || loop {
                    match beep_rx.recv() {
                        Ok(beep) => {
                            match handle.emit("beep", BeepPayload { beep }) {
                                Ok(_) => continue,
                                Err(e) => eprintln!("receive beep: {e}"),
                            }
                        }
                        Err(e) => eprintln!("channel receive: {e}"),
                    }
                });
            }

            {
                let handle = app.handle().clone();
                let keys = keys.clone();
                handle.listen_any("keyEvent", move |event| {
                    //println!("{:?}", event.payload());

                    let payload: KeyPayload = match serde_json::from_str(event.payload()) {
                        Ok(p) => p,
                        Err(e) => {
                            println!("{:?}", e);
                            return;
                        }
                    };

                    match keys.write() {
                        Ok(mut keys) => {
                            keys[payload.key] = payload.press;
                        },
                        Err(e) => eprintln!("key event: {e}"),
                    }
                });
            }

            thread::spawn(move || loop {
                match emu.lock() {
                    Ok(mut emu) => {
                        match keys.read() {
                            Ok(keys) => {
                                emu.cycle(*keys);
                            },
                            Err(e) => eprintln!("in cycle: {e}"),
                        }
                    },
                    Err(e) => eprintln!("in cycle: {e}"),
                }
            });

            let load = MenuItemBuilder::with_id("load", "Load").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&load]).build()?;

            app.set_menu(menu)?;

            app.on_menu_event(move |app, event| {
                if event.id() == "load" {
                    let file_path = app.dialog().file().blocking_pick_file();
                    match file_path {
                        Some(file_path) => {
                            let mut file = File::open(file_path.path).unwrap();
                            let mut buf = Vec::new();
                            file.read_to_end(&mut buf).unwrap();
                            emu_load.lock().unwrap().load_rom(buf);
                        }
                        None => {}
                    }
                }
            });

            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application")
}

// event.window().listen_global("keypress", |event| {

// });
// event.window().listen_global("keyrelease", |event| {

// });
