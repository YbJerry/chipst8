// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod chipst8;

use serde::{Deserialize, Serialize};
use tauri::{generate_context, Manager};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::menu::{
    MenuBuilder, MenuItemBuilder,
};

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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let (tx, rx) = std::sync::mpsc::channel();

            let emu = Arc::new(Mutex::new(Chipst8::new(tx)));
            let emu_load = emu.clone();
            let emu_key_event = emu.clone();

            {
                let handle = app.handle().clone();
                thread::spawn(move || {
                    loop {
                        match rx.recv() {
                            Ok(display) => {
                                let payload = ScreenPayload {
                                    screen: display.iter().map(|row| row.to_vec()).collect(),
                                };
                                match handle.emit("draw", Some(payload)) {
                                    Ok(_) => continue,
                                    Err(e) => eprintln!("receive display: {e}"),
                                }
                            },
                            Err(e) => eprintln!("channel receive: {e}"),
                        }
                    }
                });
            }

            {
                let handle = app.handle().clone();
                handle.listen_any("keyEvent", move |event| {
                    //println!("{:?}", event.payload());

                    let payload: KeyPayload = match serde_json::from_str(event.payload()) {
                        Ok(p) => p,
                        Err(e) => {
                            println!("{:?}", e);
                            return;
                        },
                    };

                    match emu_key_event.lock() {
                        Ok(mut emu) => emu.set_key(payload.key, payload.press),
                        Err(e) => eprintln!("key event: {e}"),
                    };
                });
            }

            thread::spawn(move || loop {
                match emu.lock() {
                    Ok(mut emu) => emu.cycle(),
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
