use std::process::{Command, Child};
use std::sync::Mutex;
use tauri::{Manager, State};

struct MimoServeState {
    child: Mutex<Option<Child>>,
}

#[tauri::command]
fn get_mimo_port() -> u16 {
    // Find an available port
    let candidates = [3000, 4000, 5000, 8000, 8080, 9000, 49152, 49153];
    for &port in &candidates {
        if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_err() {
            return port;
        }
    }
    3000
}

#[tauri::command]
fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(MimoServeState {
            child: Mutex::new(None),
        })
        .setup(|app| {
            // Set window icon
            if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../../assets/icon.png")) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon);
                }
            }

            // Find available port
            let candidates = [3000, 4000, 5000, 8000, 8080, 9000, 49152, 49153];
            let mut port = 3000u16;
            for &p in &candidates {
                if std::net::TcpStream::connect(format!("127.0.0.1:{}", p)).is_err() {
                    port = p;
                    break;
                }
            }

            let handle = app.handle().clone();

            std::thread::spawn(move || {
                let available = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok();
                if available {
                    println!("Port {} already in use, assuming mimo serve is running", port);
                    return;
                }

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
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_mimo_port, get_platform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
