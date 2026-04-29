use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, Box, Orientation, Align, Button, CssProvider};
use gtk4_layer_shell::{LayerShell, Layer, KeyboardMode};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct OverlayInstance {
    pub window: ApplicationWindow,
    pub timer_label: Label,
    pub emergency_btn: Button,
    pub on_unlock: Rc<RefCell<Option<std::boxed::Box<dyn Fn()>>>>,
}

impl OverlayInstance {
    pub fn close(&self) {
        self.window.close();
    }
}

pub struct OverlayWindow;

impl OverlayWindow {
    pub fn show_all(app: &Application, msg: &str, alpha: f64) -> Vec<OverlayInstance> {
        let display = gtk4::gdk::Display::default().unwrap();
        let monitors = display.monitors();
        let mut instances = Vec::new();

        let n_monitors = monitors.n_items();
        for i in 0..n_monitors {
            if let Some(monitor_obj) = monitors.item(i) {
                if let Ok(monitor) = monitor_obj.downcast::<gtk4::gdk::Monitor>() {
                    instances.push(Self::create_for_window(app, msg, alpha, Some(&monitor)));
                }
            }
        }

        if instances.is_empty() {
            // Fallback (should theoretically not happen if display is valid)
            instances.push(Self::create_for_window(app, msg, alpha, None));
        }

        instances
    }

    fn create_for_window(app: &Application, msg: &str, alpha: f64, monitor: Option<&gtk4::gdk::Monitor>) -> OverlayInstance {
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .css_classes(["overlay-window"])
            .build();

        if gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            if let Some(m) = monitor {
                window.set_monitor(m);
            }
            window.set_layer(Layer::Overlay);
            
            // Strict Mode
            window.set_keyboard_mode(KeyboardMode::Exclusive);

            // Anchor to all edges (No exclusive_zone to overlap panels)
            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            window.set_anchor(gtk4_layer_shell::Edge::Top, true);
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        } else {
            // GNOME không hỗ trợ Layer Shell, dùng Fullscreen truyền thống
            // GNOME Wayland ép cửa sổ Fullscreen thành màu đen đặc. 
            // Cố tình bỏ fullscreen() và dùng maximize() để giữ kênh alpha.
            window.maximize();
        }

        let vbox = Box::new(Orientation::Vertical, 0);
        vbox.set_valign(Align::Fill);
        vbox.set_halign(Align::Fill);
        vbox.set_vexpand(true);
        vbox.set_hexpand(true);
        vbox.add_css_class("overlay-box");

        let css = format!(
            "window.overlay-window {{ background: transparent; }}
             .overlay-box {{ background-color: rgba(0, 0, 0, {:.2}); }}
             label.overlay-msg {{ font-size: 48px; color: white; font-weight: bold; margin-top: 150px; text-wrap: true; }}
             label.overlay-timer {{ font-size: 150px; color: #27ae60; font-weight: bold; }}
             #emergency_btn {{ background-image: none; background-color: #d32f2f; color: white; font-size: 24px; font-weight: bold; padding: 15px 40px; border-radius: 8px; margin-bottom: 100px; border: none; }}
             #emergency_btn:hover {{ background-color: #f44336; }}",
            alpha
        );

        let provider = CssProvider::new();
        provider.load_from_data(&css);
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let msg_label = Label::builder().label(msg).css_classes(["overlay-msg"]).build();
        msg_label.set_valign(Align::Start);
        msg_label.set_halign(Align::Center);
        msg_label.set_wrap(true);
        msg_label.set_justify(gtk4::Justification::Center);
        msg_label.set_max_width_chars(40);

        let timer_label = Label::builder().label("00:00").css_classes(["overlay-timer"]).build();
        timer_label.set_valign(Align::Center);
        timer_label.set_halign(Align::Center);
        timer_label.set_vexpand(true);

        let emergency_btn = Button::builder().label("EMERGENCY EXIT (Hold 5s)").css_classes(["emergency"]).build();
        emergency_btn.set_widget_name("emergency_btn");
        emergency_btn.set_valign(Align::End);
        emergency_btn.set_halign(Align::Center);

        let on_unlock: Rc<RefCell<Option<std::boxed::Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        
        let active_hold = Rc::new(RefCell::new(None::<gtk4::glib::SourceId>));
        let hold_time = Rc::new(RefCell::new(5));
        
        let gesture = gtk4::GestureClick::new();
        gesture.set_button(0);
        gesture.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let btn_clone = emergency_btn.clone();
        let ah_clone = active_hold.clone();
        let ht_clone = hold_time.clone();
        let unlock_cb = on_unlock.clone();

        gesture.connect_pressed(move |_, _, _, _| {
            if let Some(source) = ah_clone.borrow_mut().take() {
                source.remove();
            }

            *ht_clone.borrow_mut() = 5;
            btn_clone.set_label("HOLD 5s...");
            
            let btn_tick = btn_clone.clone();
            let ht_tick = ht_clone.clone();
            let cb_tick = unlock_cb.clone();
            
            let source = gtk4::glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
                let mut time = ht_tick.borrow_mut();
                *time -= 1;
                if *time <= 0 {
                    btn_tick.set_label("UNLOCKED");
                    if let Some(cb) = cb_tick.borrow().as_ref() {
                        cb();
                    }
                    return gtk4::glib::ControlFlow::Break;
                }
                btn_tick.set_label(&format!("HOLD {}s...", time));
                gtk4::glib::ControlFlow::Continue
            });
            *ah_clone.borrow_mut() = Some(source);
        });

        let btn_clone2 = emergency_btn.clone();
        let ah_clone2 = active_hold.clone();
        let ht_clone2 = hold_time.clone();
        gesture.connect_released(move |_, _, _, _| {
            if let Some(source) = ah_clone2.borrow_mut().take() {
                source.remove();
            }
            if *ht_clone2.borrow() > 0 {
                btn_clone2.set_label("EMERGENCY EXIT (Hold 5s)");
            }
        });
        emergency_btn.add_controller(gesture);

        vbox.append(&msg_label);
        vbox.append(&timer_label);
        vbox.append(&emergency_btn);

        window.set_child(Some(&vbox));
        window.present();

        OverlayInstance {
            window,
            timer_label,
            emergency_btn,
            on_unlock,
        }
    }
}
