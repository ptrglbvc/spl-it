use std::env;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use tempfile::Builder;
use tauri::{Window, Emitter};

fn fix_path_env() {
    #[cfg(target_os = "macos")] {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        
        let output = Command::new(&shell)
            .arg("-l")  // login shell
            .arg("-i")  // interactive (loads .zshrc/.bashrc)
            .arg("-c")
            .arg("echo \"__PATH__$PATH\" && echo \"__JAVA__$JAVA_HOME\"") 
            .output();

        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            
            for line in stdout.lines() {
                if let Some(path) = line.strip_prefix("__PATH__") {
                    if !path.is_empty() {
                        eprintln!("[DEBUG] Setting PATH: {}", path);
                        env::set_var("PATH", path);
                    }
                } else if let Some(java_home) = line.strip_prefix("__JAVA__") {
                    if !java_home.is_empty() {
                        eprintln!("[DEBUG] Setting JAVA_HOME: {}", java_home);
                        env::set_var("JAVA_HOME", java_home);
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")] {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        
        if let Ok(o) = Command::new(&shell)
            .arg("-l")
            .arg("-c")
            .arg("echo $PATH")
            .output() 
        {
            let path = String::from_utf8_lossy(&o.stdout);
            let path = path.trim();
            if !path.is_empty() {
                env::set_var("PATH", path);
            }
        }
    }
}

fn find_kotlinc() -> Option<String> {
    if Command::new("kotlinc")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok() 
    {
        eprintln!("[DEBUG] Found kotlinc in PATH");
        return Some("kotlinc".to_string());
    }
    
    let home = env::var("HOME").unwrap_or_default();
    
    // Common installation locations
    let common_paths: Vec<String> = vec![
        // Homebrew (Apple Silicon)
        "/opt/homebrew/bin/kotlinc".to_string(),
        // Homebrew (Intel)
        "/usr/local/bin/kotlinc".to_string(),
        // SDKMAN
        format!("{}/.sdkman/candidates/kotlin/current/bin/kotlinc", home),
        // Manual install
        "/usr/bin/kotlinc".to_string(),
        // Snap (Linux)
        "/snap/bin/kotlinc".to_string(),
        format!("{}/.local/share/JetBrains/Toolbox/scripts/kotlinc", home),
    ];
    
    for path in common_paths {
        if Path::new(&path).exists() {
            eprintln!("[DEBUG] Found kotlinc at: {}", path);
            return Some(path);
        }
    }
    
    eprintln!("[DEBUG] kotlinc not found anywhere!");
    eprintln!("[DEBUG] Current PATH: {:?}", env::var("PATH"));
    None
}

#[tauri::command]
async fn run_kotlin_code(window: Window, code: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let kotlinc_path = match find_kotlinc() {
            Some(path) => path,
            None => {
                let _ = window.emit("stream-data", 
                    "❌ Kotlin compiler not found!\n\n\
                     Please install Kotlin:\n\
                     • macOS: brew install kotlin\n\
                     • Linux: sudo snap install kotlin --classic\n\
                     • Or via SDKMAN: sdk install kotlin\n");
                return;
            }
        };

        let mut tmp_file = Builder::new()
            .suffix(".kts")
            .tempfile()
            .expect("Failed to create temp file");

        writeln!(tmp_file, "{}", code).expect("Failed to write code");
        let path = tmp_file.path().to_path_buf();

        let mut child = match Command::new(&kotlinc_path)
            .arg("-script")
            .arg(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() 
        {
            Ok(child) => child,
            Err(e) => {
                let _ = window.emit("stream-data", 
                    format!("❌ Failed to start kotlinc: {}\n", e));
                return;
            }
        };

        // Stream stdout
        let stdout = child.stdout.take().unwrap();
        let window_out = window.clone();
        let stdout_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let _ = window_out.emit("stream-data", format!("{}\n", line));
            }
        });

        // Stream stderr
        let stderr = child.stderr.take().unwrap();
        let window_err = window.clone();
        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = window_err.emit("stream-data", format!("⚠️ {}\n", line));
            }
        });

        // Dis waits for everything to complete
        let status = child.wait();
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        
        match status {
            Ok(s) if s.success() => {
                let _ = window.emit("stream-data", "\n✅ Execution completed\n");
            }
            Ok(s) => {
                let _ = window.emit("stream-data", 
                    format!("\n❌ Process exited with: {}\n", s));
            }
            Err(e) => {
                let _ = window.emit("stream-data", 
                    format!("\n❌ Failed to wait on process: {}\n", e));
            }
        }
        
    }).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // We gotta fix PATH before anything else
    fix_path_env();
    
    // Debug: print what we ended up with
    eprintln!("[DEBUG] Final PATH: {:?}", env::var("PATH"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_kotlin_code])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
