use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

struct MimoServeState {
    child: Mutex<Option<Child>>,
    chosen_port: Arc<Mutex<u16>>,
}

#[tauri::command]
fn get_mimo_port(state: State<'_, MimoServeState>) -> u16 {
    *state.chosen_port.lock().unwrap()
}

#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port = find_port();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(MimoServeState {
            child: Mutex::new(None),
            chosen_port: Arc::new(Mutex::new(port)),
        })
        .setup(move |app| {
            // Set window icon
            if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../../assets/icon.png")) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon);
                }
            }

            let handle = app.handle().clone();

            // Check if mimo serve is already running on this port
            let already_running = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok();

            if already_running {
                println!("Port {} already in use, assuming mimo serve is running", port);
            } else {
                println!("Starting mimo serve on port {}...", port);
                match Command::new("mimo")
                    .args(["serve", "--port", &port.to_string(), "--pure"])
                    .spawn()
                {
                    Ok(child) => {
                        println!("mimo serve started with PID {}", child.id());
                        let state: State<MimoServeState> = handle.state();
                        *state.child.lock().unwrap() = Some(child);
                    }
                    Err(e) => {
                        eprintln!("Failed to start mimo serve: {}", e);
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_mimo_port, get_platform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn find_port() -> u16 {
    // First check if port 3000 has mimo serve
    if is_mimo_running(3000) {
        return 3000;
    }

    // Try common ports
    let candidates = [3000, 4000, 5000, 8000, 8080, 9000, 49152, 49153];
    for &port in &candidates {
        if is_port_free(port) {
            return port;
        }
    }

    // All taken, use last one (mimo serve might already be on one)
    3000
}

fn is_mimo_running(port: u16) -> bool {
    match std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
        Ok(stream) => {
            drop(stream);
            // Port is in use, check if it responds with JSON
            use std::io::{BufRead, BufReader, Write};
            if let Ok(mut stream) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
                let _ = stream.write_all(b"GET /session HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let _ = reader.read_line(&mut line);
                // HTTP 200 = mimo serve running
                line.contains("200")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn is_port_free(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_err()
}
