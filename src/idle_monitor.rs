use std::process::Command;
use zbus::blocking::Connection;
use zbus::proxy;

#[proxy(
    interface = "org.gnome.Mutter.IdleMonitor",
    default_service = "org.gnome.Mutter.IdleMonitor",
    default_path = "/org/gnome/Mutter/IdleMonitor/Core"
)]
trait IdleMonitor {
    fn get_idletime(&self) -> zbus::Result<u64>;
}

pub struct SystemIdleMonitor {
    use_dbus: bool,
    dbus_conn: Option<Connection>,
}

impl SystemIdleMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            use_dbus: false,
            dbus_conn: None,
        };

        // Try to connect to D-Bus and check if the IdleMonitor interface is working
        if let Ok(conn) = Connection::session() {
            if let Ok(proxy) = IdleMonitorProxyBlocking::new(&conn) {
                if proxy.get_idletime().is_ok() {
                    monitor.use_dbus = true;
                    monitor.dbus_conn = Some(conn);
                    println!("[IdleMonitor] Using D-Bus (GNOME Mutter)");
                    return monitor;
                }
            }
        }

        // Try xprintidle fallback
        if Command::new("xprintidle").output().is_ok() {
            println!("[IdleMonitor] Using xprintidle fallback");
        } else {
            println!("[IdleMonitor] WARNING: No idle detection method found.");
        }

        monitor
    }

    pub fn get_idle_seconds(&self) -> f64 {
        if self.use_dbus {
            if let Some(conn) = &self.dbus_conn {
                if let Ok(proxy) = IdleMonitorProxyBlocking::new(conn) {
                    if let Ok(time_ms) = proxy.get_idletime() {
                        return time_ms as f64 / 1000.0;
                    }
                }
            }
        }

        // Fallback to xprintidle
        if let Ok(output) = Command::new("xprintidle").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if let Ok(ms) = stdout.trim().parse::<u64>() {
                    return ms as f64 / 1000.0;
                }
            }
        }

        0.0
    }
}
