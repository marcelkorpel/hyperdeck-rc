// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;
use std::{result::Result, sync::Arc};

use tauri::Manager;
use telnet::{Event, Telnet};
use tokio::sync::{mpsc, Mutex};

pub const CRLF: &[u8] = b"\r\n";

struct AsyncProcInputTx {
    tx: Mutex<mpsc::Sender<String>>,
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
}

#[tauri::command]
async fn connect(
    host: String,
    port: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AsyncProcInputTx>,
) -> Result<(), String> {
    let rx = Arc::clone(&state.rx);

    tauri::async_runtime::spawn(async move {
        let mut telnet =
            Telnet::connect((host, port.parse().unwrap()), 7024).expect("Couldn't connect to the server...");

        loop {
            let mut rx = rx.try_lock().expect("Could not lock");
            let buf = match rx.try_recv() {
                Ok(buf) => buf,
                Err(_) => String::from(""),
            };

            if !buf.is_empty() {
                telnet.write(buf.as_bytes()).expect("Write error!");
                telnet.write(CRLF).expect("Write error!");
            }

            let event = telnet.read_nonblocking().expect("Read error");

            if let Event::Data(buffer) = event {
                app_handle
                    .emit_all("message", String::from_utf8_lossy(&buffer).trim())
                    .unwrap();
            }

            // give the CPU some air
            std::thread::sleep(Duration::from_millis(50));
        }
    });

    Ok(())
}

#[tauri::command]
async fn write(command: String, state: tauri::State<'_, AsyncProcInputTx>) -> Result<(), String> {
    let tx = state.tx.lock().await;
    tx.send(command).await.map_err(|e| e.to_string())
}

fn main() {
    let (tx, rx) = mpsc::channel(100);

    tauri::Builder::default()
        .manage(AsyncProcInputTx {
            tx: Mutex::new(tx),
            rx: Arc::new(Mutex::new(rx)),
        })
        .invoke_handler(tauri::generate_handler![connect, write])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
