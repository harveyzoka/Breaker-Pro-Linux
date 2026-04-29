use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Button, CheckButton, Entry, Label, Notebook, Orientation,
    CssProvider, ScrolledWindow, Align
};
use std::rc::Rc;
use std::cell::RefCell;

use crate::timer_logic::{AppTimer, TimerMode};

pub struct AppUI {
    pub window: ApplicationWindow,
    pub timer_label: Label,
    pub status_label: Label,
    pub start_button: Button,
    pub timer: Rc<RefCell<AppTimer>>,
}

impl AppUI {
    pub fn new(app: &Application, timer: Rc<RefCell<AppTimer>>) -> Rc<Self> {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Breaker Pro")
            .default_width(450)
            .default_height(650)
            .build();

        window.connect_close_request(move |w| {
            w.hide();
            gtk4::glib::Propagation::Stop
        });

        let provider = CssProvider::new();
        provider.load_from_data(
            "window { background-color: #1a1a2e; color: #e0e0e0; }
             notebook { background-color: #1a1a2e; }
             label.status-text { font-size: 16px; font-weight: bold; color: #a0a0a0; margin-bottom: 20px; }
             label.timer-text { font-size: 72px; font-weight: bold; margin-bottom: 30px; }
             button { background-image: none; background-color: #1f6aa5; color: white; border-radius: 4px; padding: 10px; margin: 5px; border: none; }
             button:hover { background-color: #2980b9; }
             button.skip { background-color: #555555; }
             button.reset { background-color: #aa3333; }
             button.save { background-color: #44aa44; font-weight: bold; }
             button.save:hover { background-color: #1a6b1a; }
             entry { background-color: #0f3460; color: white; margin-bottom: 10px; }
             checkbutton { margin-bottom: 10px; }
             windowcontrols button { background-color: transparent; box-shadow: none; border: none; }
             windowcontrols button:hover { background-color: rgba(255,255,255,0.1); }
             "
        );
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let notebook = Notebook::new();
        window.set_child(Some(&notebook));

        // Timer Tab
        let timer_box = Box::new(Orientation::Vertical, 0);
        timer_box.set_margin_top(40);
        timer_box.set_margin_bottom(20);
        timer_box.set_margin_start(20);
        timer_box.set_margin_end(20);
        timer_box.set_valign(Align::Center);
        timer_box.set_halign(Align::Center);

        let status_label = Label::builder().label("SITTING TIME").css_classes(["status-text"]).build();
        let timer_label = Label::builder().label("00:00").css_classes(["timer-text"]).build();

        let controls_box = Box::new(Orientation::Horizontal, 10);
        let start_button = Button::with_label("START");
        let skip_button = Button::builder().label("SKIP").css_classes(["skip"]).build();
        let reset_button = Button::builder().label("RESET").css_classes(["reset"]).build();

        controls_box.append(&start_button);
        controls_box.append(&skip_button);
        controls_box.append(&reset_button);
        controls_box.set_halign(Align::Center);

        timer_box.append(&status_label);
        timer_box.append(&timer_label);
        timer_box.append(&controls_box);

        notebook.append_page(&timer_box, Some(&Label::new(Some("Timer"))));

        // Settings Tab
        let settings_scroll = ScrolledWindow::new();
        let settings_box = Box::new(Orientation::Vertical, 5);
        settings_box.set_margin_top(20);
        settings_box.set_margin_start(20);
        settings_box.set_margin_end(20);
        settings_box.set_margin_bottom(20);

        let current_settings = timer.borrow().get_settings();

        let add_setting = |container: &Box, label_text: &str, value: &str| -> Entry {
            let b = Box::new(Orientation::Horizontal, 10);
            let lbl = Label::builder().label(label_text).halign(Align::Start).width_request(150).build();
            let entry = Entry::builder().text(value).hexpand(true).build();
            b.append(&lbl);
            b.append(&entry);
            container.append(&b);
            entry
        };

        settings_box.append(&Label::builder().label("--- TIME (Minutes / Seconds) ---").halign(Align::Start).margin_bottom(10).build());
        let e_sit = add_setting(&settings_box, "Sitting (min):", &current_settings.sitting_duration.to_string());
        let e_stand = add_setting(&settings_box, "Standing (min):", &current_settings.standing_duration.to_string());
        let e_trans = add_setting(&settings_box, "Transition (sec):", &current_settings.transition_duration.to_string());
        let e_idle = add_setting(&settings_box, "Idle Reset (min):", &current_settings.idle_threshold.to_string());

        settings_box.append(&Label::builder().label("--- WORK HOURS (HH:MM-HH:MM) ---").halign(Align::Start).margin_top(10).margin_bottom(10).build());
        let e_sched = add_setting(&settings_box, "Schedule:", &current_settings.work_schedules.join(", "));

        settings_box.append(&Label::builder().label("--- NOTIFICATIONS ---").halign(Align::Start).margin_top(10).margin_bottom(10).build());
        let e_sit_msg = add_setting(&settings_box, "Sit msg:", &current_settings.sit_msg);
        let e_stand_msg = add_setting(&settings_box, "Stand msg:", &current_settings.stand_msg);
        
        // Sound path with FileDialog
        let file_box = Box::new(Orientation::Horizontal, 10);
        let lbl = Label::builder().label("Sound File:").halign(Align::Start).width_request(150).build();
        let e_sound = Entry::builder().text(&current_settings.sound_path).hexpand(true).build();
        let btn_browse = Button::with_label("Browse...");
        
        file_box.append(&lbl);
        file_box.append(&e_sound);
        file_box.append(&btn_browse);
        settings_box.append(&file_box);

        let e_sound_clone = e_sound.clone();
        let window_clone = window.clone();
        btn_browse.connect_clicked(move |_| {
            let dialog = gtk4::FileChooserNative::new(
                Some("Select audio file"),
                Some(&window_clone),
                gtk4::FileChooserAction::Open,
                Some("Select"),
                Some("Cancel"),
            );
            
            let e_sound_inner = e_sound_clone.clone();
            dialog.connect_response(move |d, response| {
                if response == gtk4::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            e_sound_inner.set_text(&path.to_string_lossy());
                        }
                    }
                }
            });
            dialog.show();
        });

        settings_box.append(&Label::builder().label("--- OTHERS ---").halign(Align::Start).margin_top(10).margin_bottom(10).build());
        let cb_strict = CheckButton::builder().label("Strict Mode (Lock screen)").active(current_settings.strict_mode).build();
        let cb_auto = CheckButton::builder().label("Auto Start (System boot)").active(current_settings.auto_start).build();
        settings_box.append(&cb_strict);
        settings_box.append(&cb_auto);

        let alpha_box = Box::new(Orientation::Horizontal, 10);
        let alpha_lbl = Label::builder().label("Overlay Alpha:").halign(Align::Start).width_request(150).build();
        let alpha_scale = gtk4::Scale::with_range(Orientation::Horizontal, 0.0, 1.0, 0.05);
        alpha_scale.set_value(current_settings.overlay_alpha as f64);
        alpha_scale.set_hexpand(true);
        alpha_box.append(&alpha_lbl);
        alpha_box.append(&alpha_scale);
        settings_box.append(&alpha_box);

        let save_button = Button::builder().label("Save Settings").css_classes(["save"]).margin_top(20).build();
        settings_box.append(&save_button);

        settings_scroll.set_child(Some(&settings_box));
        notebook.append_page(&settings_scroll, Some(&Label::new(Some("Settings"))));

        let ui = Rc::new(Self {
            window,
            timer_label,
            status_label,
            start_button: start_button.clone(),
            timer: timer.clone(),
        });

        // Timer Tab Handlers
        let ui_clone = ui.clone();
        start_button.connect_clicked(move |_| {
            let t = ui_clone.timer.borrow_mut();
            if t.state.borrow().is_running {
                t.pause();
                ui_clone.start_button.set_label("START");
            } else {
                t.start();
                ui_clone.start_button.set_label("PAUSE");
            }
        });

        let ui_clone2 = ui.clone();
        skip_button.connect_clicked(move |_| {
            {
                let t = ui_clone2.timer.borrow_mut();
                t.skip();
            }
            ui_clone2.start_button.set_label("START");
            ui_clone2.update_display();
        });

        let ui_clone3 = ui.clone();
        reset_button.connect_clicked(move |_| {
            {
                let t = ui_clone3.timer.borrow_mut();
                t.reset();
            }
            ui_clone3.start_button.set_label("START");
            ui_clone3.update_display();
        });

        // Settings Save Handler
        let timer_save = timer.clone();
        let current_for_save = current_settings.clone();
        save_button.connect_clicked(move |btn| {
            let mut new_settings = current_for_save.clone();
            
            if let Ok(val) = e_sit.text().parse::<u32>() { new_settings.sitting_duration = val; }
            if let Ok(val) = e_stand.text().parse::<u32>() { new_settings.standing_duration = val; }
            if let Ok(val) = e_trans.text().parse::<u32>() { new_settings.transition_duration = val; }
            if let Ok(val) = e_idle.text().parse::<u32>() { new_settings.idle_threshold = val; }
            
            let sched_text = e_sched.text().to_string();
            new_settings.work_schedules = sched_text.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            
            new_settings.sit_msg = e_sit_msg.text().to_string();
            new_settings.stand_msg = e_stand_msg.text().to_string();
            new_settings.sound_path = e_sound.text().to_string();
            
            new_settings.strict_mode = cb_strict.is_active();
            new_settings.auto_start = cb_auto.is_active();
            new_settings.overlay_alpha = alpha_scale.value() as f32;

            new_settings.save();
            
            crate::system_utils::AutoStarter::new().set_autostart(new_settings.auto_start);

            timer_save.borrow_mut().update_settings(new_settings.clone());
            
            btn.set_label("Saved Successfully!");
            gtk4::glib::timeout_add_local(std::time::Duration::from_secs(2), gtk4::glib::clone!(@weak btn => @default-return gtk4::glib::ControlFlow::Break, move || {
                btn.set_label("Save Settings");
                gtk4::glib::ControlFlow::Break
            }));
        });

        ui.update_display();
        ui
    }

    pub fn update_display(&self) {
        let t = self.timer.borrow();
        let state = t.state.borrow();
        
        let mins = state.remaining_seconds / 60;
        let secs = state.remaining_seconds % 60;
        self.timer_label.set_label(&format!("{:02}:{:02}", mins, secs));

        let mut status = match state.mode {
            TimerMode::Sitting => "SITTING TIME",
            TimerMode::Standing => "STANDING TIME",
            TimerMode::Transition => "TRANSITION",
        }.to_string();

        if !state.status_reason.is_empty() {
            status = format!("{} - {}", status, state.status_reason);
        }

        self.status_label.set_label(&status);

        if state.is_running {
            self.start_button.set_label("PAUSE");
        } else {
            self.start_button.set_label("START");
        }
    }
}
