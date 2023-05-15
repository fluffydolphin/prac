#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpStream;
use std::io::{Read, Write};
use std::process::exit;
use std::{thread, time};
use fernet;
use tauri::Window;
use lazy_static::lazy_static;
use std::sync::Mutex;
use chrono::prelude::*;
use tauri::ClipboardManager;
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, SystemTrayMenuItem};
use tauri::Manager;
use std::net::ToSocketAddrs;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use discord_rich_presence::activity::{Assets, Button, Timestamps};

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
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
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
async fn send_login(handle: tauri::AppHandle, domain_name: String, username: String, password: String, room_number: String) -> String {
    let port = 430;

    let mut addresses = match (domain_name, port).to_socket_addrs() {
        Ok(addresses) => addresses,
        Err(e) => {
            return format!("Failed to connect to: {}", e);
        }
    };

    let address = match addresses.next() {
        Some(addr) => addr,
        None => {
            return "No addresses found for domain name".to_string();
        }
    };
    let ip = match address.ip() {
        std::net::IpAddr::V4(ipv4) => ipv4.to_string(),
        std::net::IpAddr::V6(ipv6) => ipv6.to_string(),
    };
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
        *ROOM_NUMBER.lock().unwrap() = Some(room_number.clone());
        let _ = thread::spawn(move || {
            discord_pres(room_number.clone(),  time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Failed to get timestamp")
            .as_millis() as i64);
        });

        *WINAME.lock().unwrap() = Some("editor".to_string());
        let _ = tauri::WindowBuilder::new(
            &handle,
            "editor",
            tauri::WindowUrl::App("index2.html".into())
          )
          .title("Fluffy Chat")
          .build().unwrap();
        let window = handle.get_window("main").unwrap();
        window.close().unwrap();
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
            listens(handle.get_window("editor").unwrap(), stream);
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

fn discord_pres(room_number: String, start_time: i64) {
    loop {
        let mut buttons = Vec::new();

    
        let mut client = DiscordIpcClient::new("1107278007820890112").expect("Could not connect to Discord client");
        client.connect().expect("Could not connect to Discord client");
    
        buttons.push(Button::new("Github", "https://github.com/fluffydolphin/fluffy-chat"));
    
        let details = format!("Room: {}", room_number);
    
        let payload = activity::Activity::new().assets(Assets::new().large_image("fluffy-chat").large_text("Fluffy Chat")).buttons(buttons).details(details.as_str()).timestamps(Timestamps::new().start(start_time));
        client.set_activity(payload).expect("Could not connect to Discord client");

        thread::sleep(time::Duration::from_millis(15));
    }
}