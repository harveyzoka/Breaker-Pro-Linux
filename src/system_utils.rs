use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::env;

pub fn play_notification_sound(sound_path: &str) {
    if sound_path.is_empty() {
        return;
    }
    
    let path = PathBuf::from(sound_path);
    if !path.exists() {
        return;
    }

    let p = sound_path.to_string();
    std::thread::spawn(move || {
        let players = ["paplay", "aplay", "play", "ffplay"];
        for player in players {
            let mut cmd = Command::new(player);
            cmd.arg(&p);
            
            if player == "ffplay" {
                cmd.arg("-nodisp").arg("-autoexit");
            }
    
            if let Ok(mut child) = cmd.spawn() {
                let _ = child.wait(); // Prevent zombie process
                return;
            }
        }
        
        println!("[Sound] No CLI player found. Install pulseaudio-utils (paplay) or alsa-utils (aplay).");
    });
}

pub struct AutoStarter {
    desktop_filename: &'static str,
}

impl AutoStarter {
    pub fn new() -> Self {
        Self {
            desktop_filename: "breaker-pro.desktop",
        }
    }

    fn autostart_dir(&self) -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("autostart");
        path
    }

    fn desktop_file_path(&self) -> PathBuf {
        let mut path = self.autostart_dir();
        path.push(self.desktop_filename);
        path
    }

    pub fn set_autostart(&self, enable: bool) {
        let path = self.desktop_file_path();

        if enable {
            let dir = self.autostart_dir();
            if !dir.exists() {
                let _ = fs::create_dir_all(&dir);
            }

            let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("breaker-pro-rust"));
            let exe_str = exe_path.to_string_lossy();
            let exec_cmd = format!("bash -c 'sleep 5 && \"{}\" --minimized'", exe_str);

            let content = format!(
r#"[Desktop Entry]
Type=Application
Name=Breaker Pro
Comment=Sit/Stand Timer for Health
Exec={}
Icon=preferences-desktop-screensaver
Terminal=false
Hidden=false
X-GNOME-Autostart-enabled=true
StartupNotify=false
Categories=Utility;Health;
"#, exec_cmd);

            let _ = fs::write(&path, content);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mut perms) = fs::metadata(&path).map(|m| m.permissions()) {
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&path, perms);
                }
            }
        } else {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_autostart_enabled(&self) -> bool {
        let path = self.desktop_file_path();
        if !path.exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("Hidden=true") || content.contains("X-GNOME-Autostart-enabled=false") {
                return false;
            }
            return true;
        }

        false
    }
}
