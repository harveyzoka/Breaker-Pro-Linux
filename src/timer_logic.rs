use crate::settings::AppSettings;
use crate::idle_monitor::SystemIdleMonitor;
use chrono::Local;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TimerMode {
    Sitting,
    Standing,
    Transition,
}

pub struct TimerState {
    pub mode: TimerMode,
    pub next_mode: TimerMode,
    pub remaining_seconds: u32,
    pub is_running: bool,
    pub status_reason: String,
}

pub struct AppTimer {
    settings: AppSettings,
    idle_monitor: SystemIdleMonitor,
    pub state: Rc<RefCell<TimerState>>,
}

impl AppTimer {
    pub fn new(settings: AppSettings) -> Self {
        let state = TimerState {
            mode: TimerMode::Sitting,
            next_mode: TimerMode::Standing,
            remaining_seconds: settings.sitting_duration * 60,
            is_running: false,
            status_reason: "".to_string(),
        };

        Self {
            settings,
            idle_monitor: SystemIdleMonitor::new(),
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn start(&self) {
        self.state.borrow_mut().is_running = true;
    }

    pub fn pause(&self) {
        self.state.borrow_mut().is_running = false;
    }

    pub fn reset(&self) {
        let mut state = self.state.borrow_mut();
        state.is_running = false;
        if state.mode == TimerMode::Transition {
            state.mode = state.next_mode;
        }
        
        if state.mode == TimerMode::Sitting {
            state.remaining_seconds = self.settings.sitting_duration * 60;
        } else {
            state.remaining_seconds = self.settings.standing_duration * 60;
        }
        state.status_reason = "Reset".to_string();
    }

    pub fn skip(&self) {
        let mut state = self.state.borrow_mut();
        
        match state.mode {
            TimerMode::Sitting => {
                state.mode = TimerMode::Transition;
                state.next_mode = TimerMode::Standing;
                state.remaining_seconds = self.settings.transition_duration;
            }
            TimerMode::Standing => {
                state.mode = TimerMode::Transition;
                state.next_mode = TimerMode::Sitting;
                state.remaining_seconds = self.settings.transition_duration;
            }
            TimerMode::Transition => {
                state.mode = state.next_mode;
                if state.mode == TimerMode::Sitting {
                    state.remaining_seconds = self.settings.sitting_duration * 60;
                } else {
                    state.remaining_seconds = self.settings.standing_duration * 60;
                }
            }
        }
        state.status_reason = "Skipped".to_string();
    }

    pub fn update_settings(&mut self, settings: AppSettings) {
        self.settings = settings.clone();
        if !self.state.borrow().is_running {
            self.reset();
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.clone()
    }

    fn is_within_work_hours(&self) -> bool {
        if self.settings.work_schedules.is_empty() {
            return true;
        }

        let now = Local::now();
        let current_min = now.format("%H").to_string().parse::<u32>().unwrap() * 60 
            + now.format("%M").to_string().parse::<u32>().unwrap();

        for schedule in &self.settings.work_schedules {
            let parts: Vec<&str> = schedule.split('-').collect();
            if parts.len() == 2 {
                let start_parts: Vec<&str> = parts[0].split(':').collect();
                let end_parts: Vec<&str> = parts[1].split(':').collect();
                
                if start_parts.len() == 2 && end_parts.len() == 2 {
                    if let (Ok(sh), Ok(sm), Ok(eh), Ok(em)) = (
                        start_parts[0].parse::<u32>(), start_parts[1].parse::<u32>(),
                        end_parts[0].parse::<u32>(), end_parts[1].parse::<u32>()
                    ) {
                        let start_min = sh * 60 + sm;
                        let end_min = eh * 60 + em;

                        if start_min < end_min {
                            if current_min >= start_min && current_min < end_min {
                                return true;
                            }
                        } else {
                            if current_min >= start_min || current_min < end_min {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub fn tick(&self) -> Option<TimerMode> {
        let mut state = self.state.borrow_mut();
        
        if !state.is_running {
            return None;
        }

        if !self.is_within_work_hours() {
            state.status_reason = "Outside Work Hours".to_string();
            return None;
        }

        if state.mode == TimerMode::Sitting {
            let idle_sec = self.idle_monitor.get_idle_seconds();
            if idle_sec > (self.settings.idle_threshold * 60) as f64 {
                state.remaining_seconds = self.settings.sitting_duration * 60;
                state.status_reason = "User Idle".to_string();
                return None;
            }
        }

        state.status_reason = "".to_string();
        
        if state.remaining_seconds > 0 {
            state.remaining_seconds -= 1;
        }

        if state.remaining_seconds == 0 {
            state.is_running = false;
            
            match state.mode {
                TimerMode::Sitting => {
                    state.mode = TimerMode::Transition;
                    state.next_mode = TimerMode::Standing;
                    state.remaining_seconds = self.settings.transition_duration;
                }
                TimerMode::Standing => {
                    state.mode = TimerMode::Transition;
                    state.next_mode = TimerMode::Sitting;
                    state.remaining_seconds = self.settings.transition_duration;
                }
                TimerMode::Transition => {
                    state.mode = state.next_mode;
                    if state.mode == TimerMode::Sitting {
                        state.remaining_seconds = self.settings.sitting_duration * 60;
                    } else {
                        state.remaining_seconds = self.settings.standing_duration * 60;
                    }
                }
            }
            
            state.is_running = true; 
            return Some(state.mode); 
        }

        None
    }
}
