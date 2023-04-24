// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

use std::net::TcpStream;
use std::io::{Read, Write};
use fernet;
use tauri::Window;
use clipboard::{ClipboardProvider, ClipboardContext};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref ROOM_NUMBER: Mutex<Option<String>> = Mutex::new(None);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_login, get_room_key])
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
        let _ = tauri::WindowBuilder::new(
            &handle,
            "editor", /* the unique window label */
            tauri::WindowUrl::App("index2.html".into())
          )
          .title("Fluffy Chat")
          .build().unwrap();
        window.close().unwrap();
        *ROOM_NUMBER.lock().unwrap() = Some(room_number.clone());
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
fn get_room_key() {
    let room_number = ROOM_NUMBER.lock().unwrap().clone().unwrap_or_default();
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(room_number).unwrap();
}

fn send_msg(msg: String) {

}

fn listen(window: Window, mut stream: TcpStream) {
    let seperator: &str = "<sep>";
    let key = fernet::Fernet::new("fXpsGp9mJFfNYCTtGeB2zpY9bzjPAoaC0Fkcc13COy4=").unwrap();
    while true {

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
    }
}
