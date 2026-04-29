mod settings;
mod timer_logic;
mod idle_monitor;
mod system_utils;
mod overlay;
mod app_ui;

use gtk4::prelude::*;
use gtk4::Application;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;

use settings::AppSettings;
use timer_logic::{AppTimer, TimerMode};
use app_ui::AppUI;
use overlay::OverlayWindow;
use system_utils::play_notification_sound;

extern "C" {
    fn malloc_trim(pad: usize) -> i32;
}

fn build_ui(app: &Application) {
    if let Some(display) = gtk4::gdk::Display::default() {
        let icon_theme = gtk4::IconTheme::for_display(&display);
        icon_theme.add_search_path("/run/media/zoka/Harvey/Workspace/Breaker-Pro-Rust");
        gtk4::Window::set_default_icon_name("app");
    }

    let app_settings = AppSettings::load();
    
    // AutoStart sync
    let autostarter = system_utils::AutoStarter::new();
    autostarter.set_autostart(app_settings.auto_start);

    let timer = Rc::new(RefCell::new(AppTimer::new(app_settings.clone())));
    
    // Auto start on launch
    timer.borrow_mut().start();

    let ui = AppUI::new(app, timer.clone());
    
    // We only show window if not hidden
    let args: Vec<String> = std::env::args().collect();
    if !args.contains(&"--minimized".to_string()) {
        ui.window.present();
    }

    let ui_ref = ui.clone();
    let app_ref = app.clone();
    
    // Khai báo active_overlays bên ngoài closure
    let active_overlays: Rc<RefCell<Vec<overlay::OverlayInstance>>> = Rc::new(RefCell::new(Vec::new()));
    let active_overlays_ref = active_overlays.clone();
    
    // Trigger cho System Tray
    let should_show = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let should_show_clone = should_show.clone();

    let mut tick_count = 0;

    // Timer tick loop
    gtk4::glib::timeout_add_local(Duration::from_secs(1), move || {
        tick_count += 1;

        if should_show_clone.swap(false, std::sync::atomic::Ordering::Relaxed) {
            ui_ref.window.present();
        }
        
        let new_mode = ui_ref.timer.borrow_mut().tick();
        if let Some(new_mode) = new_mode {
            let settings_ref = ui_ref.timer.borrow().get_settings();
            play_notification_sound(&settings_ref.sound_path);

            if new_mode == TimerMode::Transition {
                let msg = if ui_ref.timer.borrow().state.borrow().next_mode == TimerMode::Sitting {
                    &settings_ref.sit_msg
                } else {
                    &settings_ref.stand_msg
                };
                
                if settings_ref.strict_mode {
                    let instances = OverlayWindow::show_all(&app_ref, msg, settings_ref.overlay_alpha as f64);
                    
                    let active_clone = active_overlays_ref.clone();
                    
                    for inst in &instances {
                        let ac = active_clone.clone();
                        *inst.on_unlock.borrow_mut() = Some(std::boxed::Box::new(move || {
                            for o in ac.borrow().iter() {
                                o.close();
                            }
                            ac.borrow_mut().clear();
                        }));
                    }
                    
                    *active_overlays_ref.borrow_mut() = instances;
                }
            } else {
                // Đóng Overlay nếu chuyển sang mode làm việc
                for o in active_overlays_ref.borrow().iter() {
                    o.close();
                }
                active_overlays_ref.borrow_mut().clear();
            }
        }
        
        // Cập nhật đếm ngược trên TẤT CẢ Overlay đang hiển thị
        if ui_ref.timer.borrow().state.borrow().mode == TimerMode::Transition {
            let s = ui_ref.timer.borrow().state.borrow().remaining_seconds;
            let mins = s / 60;
            let secs = s % 60;
            let label_text = format!("{:02}:{:02}", mins, secs);
            
            for overlay in active_overlays_ref.borrow().iter() {
                overlay.timer_label.set_label(&label_text);
            }
        }

        
        ui_ref.update_display();
        
        // Tối ưu hóa bộ nhớ: Buộc glibc trả lại bộ nhớ không dùng cho OS mỗi 10 phút (600 giây)
        if tick_count % 600 == 0 {
            unsafe {
                malloc_trim(0);
            }
        }

        gtk4::glib::ControlFlow::Continue
    });

    std::thread::spawn(move || {
        struct AppTray {
            should_show: std::sync::Arc<std::sync::atomic::AtomicBool>,
        }
        impl ksni::Tray for AppTray {
            fn icon_name(&self) -> String { "app".into() }
            fn icon_theme_path(&self) -> String { "/run/media/zoka/Harvey/Workspace/Breaker-Pro-Rust".into() }
            fn id(&self) -> String { "breaker-pro-rust".into() }
            fn category(&self) -> ksni::Category { ksni::Category::ApplicationStatus }
            fn title(&self) -> String { "Breaker Pro".into() }
            fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
                use ksni::menu::*;
                let trigger = self.should_show.clone();
                vec![
                    StandardItem {
                        label: "Show Window".into(),
                        activate: Box::new(move |_| {
                            trigger.store(true, std::sync::atomic::Ordering::Relaxed);
                        }),
                        ..Default::default()
                    }.into(),
                    StandardItem {
                        label: "Quit".into(),
                        activate: Box::new(|_| std::process::exit(0)),
                        ..Default::default()
                    }.into(),
                ]
            }
        }
        
        let service = ksni::TrayService::new(AppTray { should_show });
        if let Err(e) = service.run() {
            println!("Tray error: {}", e);
        }
    });
}

fn main() {
    // Tối ưu hóa RAM: Buộc GTK4 sử dụng Cairo (Software Rendering)
    // Nếu dùng NGL/Vulkan mặc định, GTK4 sẽ ngốn khoảng 180MB RAM cho bộ đệm GPU.
    // Với Cairo, ứng dụng chỉ tốn khoảng 40MB-50MB RAM.
    std::env::set_var("GSK_RENDERER", "cairo");

    let application = Application::builder()
        .application_id("com.github.breaker.pro")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    application.connect_activate(build_ui);

    application.run_with_args(&["breaker-pro-rust"]);
}
