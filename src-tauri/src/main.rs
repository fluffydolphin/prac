#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpStream;
use std::io::{Read, Write};
use std::process::exit;
use std::thread;
use fernet;
use tauri::Window;
use lazy_static::lazy_static;
use std::sync::Mutex;
use chrono::prelude::*;
use tauri::ClipboardManager;
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, SystemTrayMenuItem};
use tauri::Manager;

#[derive(Clone, serde::Serialize)]
struct Payload {
    output_user: String,
    output: String
}

lazy_static! {
    static ref SHOWSTATE: Mutex<Option<String>> = Mutex::new(None);
    static ref WINAME: Mutex<Option<String>> = Mutex::new(None);
    static ref ROOM_NUMBER: Mutex<Option<String>> = Mutex::new(None);
    static ref USERNAME: Mutex<Option<String>> = Mutex::new(None);
    static ref REAL_STREAM: Mutex<Option<TcpStream>> = Mutex::new(None);
}

fn main() {
    *WINAME.lock().unwrap() = Some("main".to_string());
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
    .add_item(quit)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(hide);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
    .on_window_event(|event| match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
          api.prevent_close();
          win2exit();
        }
        _ => {}
      })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
              position: _,
              size: _,
              ..
            } => {
              println!("system tray received a left click");
              if SHOWSTATE.lock().unwrap().clone().unwrap_or_default() == "1".to_string() {
                let window = app.get_window(WINAME.lock().unwrap().clone().unwrap_or_default().as_str()).unwrap();
                window.show().unwrap();
                *SHOWSTATE.lock().unwrap() = Some("0".to_string());
              } 
            }
            SystemTrayEvent::RightClick {
              position: _,
              size: _,
              ..
            } => {
              println!("system tray received a right click");
            }
            SystemTrayEvent::DoubleClick {
              position: _,
              size: _,
              ..
            } => {
              println!("system tray received a double click");
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
              match id.as_str() {
                "quit" => {
                    win2exit();
                }
                "hide" => {
                  let window = app.get_window(WINAME.lock().unwrap().clone().unwrap_or_default().as_str()).unwrap();
                  window.hide().unwrap();
                  *SHOWSTATE.lock().unwrap() = Some("1".to_string());
                }
                _ => {}
              }
            }
            _ => {}
          })
        .invoke_handler(tauri::generate_handler![send_login, get_room_key, send_msg])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

}

#[tauri::command]
async fn send_login(window: Window, handle: tauri::AppHandle, ip: String, username: String, password: String, room_number: String) -> String {
    let seperator: &str = "<sep>";
    let key = fernet::Fernet::new("fXpsGp9mJFfNYCTtGeB2zpY9bzjPAoaC0Fkcc13COy4=").unwrap();
    let address = format!("{}:430", ip);
    let mut stream = match TcpStream::connect(address) {
        Ok(stream) => stream,
        Err(_) => {
            return "Could not connect to server".to_string();
        }
    };
    let _ = stream.write_all(key.encrypt(format!("{}{}{}{}{}", room_number, seperator, username, seperator, password).as_bytes()).as_bytes());
    let mut buffer = [0; 1024];
    let bytes = stream.read(&mut buffer).unwrap();
    let out_mesg = String::from_utf8(key.decrypt(&String::from_utf8_lossy(&buffer[..bytes])).unwrap()).unwrap();

    drop(stream);
    if out_mesg == "successful" {
        *WINAME.lock().unwrap() = Some("editor".to_string());
        let _ = tauri::WindowBuilder::new(
            &handle,
            "editor",
            tauri::WindowUrl::App("index2.html".into())
          )
          .title("Fluffy Chat")
          .build().unwrap();
        window.close().unwrap();
        *ROOM_NUMBER.lock().unwrap() = Some(room_number.clone());
        *USERNAME.lock().unwrap() = Some(username.clone());
        let stream = match TcpStream::connect(format!("{}:431", ip)) {
            Ok(stream) => {
                *REAL_STREAM.lock().unwrap() = Some(stream.try_clone().unwrap());
                stream
            },
            Err(_) => {
                return "Could not connect to server".to_string();
            }
        };
        let _ = thread::spawn(move || {
            listens(window, stream);
        });
        return "success".to_string();
    } else if out_mesg == "already in" {
        return "This user is already logged in".to_string();
    } else if out_mesg == "failed" {
        return "Invalid username or password".to_string();
    } else { 
        return "Unknown error".to_string();
    }
}

#[tauri::command]
fn get_room_key(handle: tauri::AppHandle) {
    let room_number = ROOM_NUMBER.lock().unwrap().clone().unwrap_or_default();
    let mut manager = handle.clipboard_manager();
    let _ = manager.write_text(room_number);
}

#[tauri::command]
fn send_msg(msg: String) {
    let time = Local::now().format("%H:%M:%S").to_string();
    let seperator: &str = "<sep>";
    let sepz: &str = "<sepz>";
    let key = fernet::Fernet::new("fXpsGp9mJFfNYCTtGeB2zpY9bzjPAoaC0Fkcc13COy4=").unwrap();
    let room_number = ROOM_NUMBER.lock().unwrap().clone().unwrap_or_default();
    let username = USERNAME.lock().unwrap().clone().unwrap_or_default();
    let mut stream = REAL_STREAM.lock().unwrap().as_ref().unwrap().try_clone().unwrap();
    let _ = stream.write_all(key.encrypt(format!("{}{}{} @ {}{}{}", room_number, sepz, username, time, seperator, msg).as_bytes()).as_bytes());
}

fn listens(window: Window, mut stream: TcpStream) {
    let seperator: &str = "<sep>";
    let key = fernet::Fernet::new("fXpsGp9mJFfNYCTtGeB2zpY9bzjPAoaC0Fkcc13COy4=").unwrap();
    let username = USERNAME.lock().unwrap().clone().unwrap_or_default();
    loop {

        let mut buffer = [0; 1024];
        let bytes = stream.read(&mut buffer).unwrap();
        let out_msg = String::from_utf8(key.decrypt(&String::from_utf8_lossy(&buffer[..bytes])).unwrap()).unwrap();
        let parts: Vec<&str> = out_msg.split(seperator).collect();
        let output_user = parts[0];
        let output = parts[1];
        if output == "serverexit" {
            drop(stream);
            window.close().unwrap();
            break;
        }
        if !output_user.contains(&username) {
            let _ = window.emit("listen", serde_json::to_string(&Payload {
                output_user: output_user.to_string(),
                output: output.to_string()}).unwrap());
        }
    }
}

fn win2exit() {
    let sepz: &str = "<sepz>";
    let key = fernet::Fernet::new("fXpsGp9mJFfNYCTtGeB2zpY9bzjPAoaC0Fkcc13COy4=").unwrap();
    let room_number = ROOM_NUMBER.lock().unwrap().clone().unwrap_or_default();
    let mut stream = REAL_STREAM.lock().unwrap().as_ref().unwrap().try_clone().unwrap();
    let _ = stream.write_all(key.encrypt(format!("{}{}{}", room_number, sepz, "/exit").as_bytes()).as_bytes());
    exit(0);
}