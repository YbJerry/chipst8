// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod chipst8;

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

#[derive(Clone, serde::Serialize)]
struct Payload {
    //   #[serde(serialize_with = "<[_]>::serialize")]
    screen: Vec<Vec<bool>>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let (tx, rx) = std::sync::mpsc::channel();

            let emu = Arc::new(Mutex::new(Chipst8::new(tx)));
            let emu_in_app = emu.clone();

            {
                let handle = app.handle().clone();
                thread::spawn(move || {
                    for display in rx {
                        let payload = Payload {
                            screen: display.iter().map(|row| row.to_vec()).collect(),
                        };
                        handle.emit("draw", Some(payload)).expect("failed to emit");
                    }
                });
            }

            {
                let handle = app.handle().clone();
                app.listen_any("key", move |event| {
                    println!("app is ready");
              
                    // we no longer need to listen to the event
                    // we also could have used `app.once_global` instead
                    handle.unlisten(event.id());
                });
            }

            
        

            thread::spawn(move || loop {
                emu.lock().unwrap().cycle();
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
                            emu_in_app.lock().unwrap().load_rom(buf);
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
