//! Application state
//!
//! Wraps the application manager in a display manager state
//!  

use crate::application::states::prelude::*;

use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::prelude::*;

pub struct NotificationState {
    buffer: String<U512>,
    is_running: bool,
    state: InternalState,
    menu: Menu
}

const MAX_ITEMS: i8 = 8;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Menu {
    state_idx: i8,
    item_count: i8,
}

impl Menu {

    pub const fn new() -> Self {
        Menu {
            state_idx: 0,
            item_count: MAX_ITEMS
        }
    }
    /// Move to the previous state in a wrapping fashion
    fn prev(&mut self) {
        self.state_idx -= 1;
        if self.state_idx < 0 {
            self.state_idx = MAX_ITEMS - 1;
        }
    }

    /// Move to the next state in a wrapping fashion
    fn next(&mut self) {
        self.state_idx += 1;
        if self.state_idx > MAX_ITEMS - 1 {
            self.state_idx = 0;
        }
    }

    fn selected(&self) -> i8 {
        self.state_idx
    }

    fn update_count(&mut self, item_count: i8) {
        self.item_count = item_count;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum InternalState {
    Menu,
    Body,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            is_running: false,
            state: InternalState::Menu,
            menu: Menu::new()
        }
    }
}

impl State for NotificationState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        self.buffer.clear();
        self.menu.update_count(system.nm().idx() as i8 + 1);
        match self.state {
            InternalState::Menu => {
                for item in 0..system.nm().idx() {
                    system.nm().peek_notification(item, |notification| {
                        write!(self.buffer, "{}", notification.title()).unwrap();
                        display.draw(horizontal_centre(Font6x12::render_str(self.buffer.as_str()), item as i32 * 12)
                                .with_stroke(Some(0x02D4_u16.into()))
                                .into_iter(),
                        );
                    });
                }
            },
            InternalState::Body => {
                system.nm().peek_notification(self.menu.selected() as usize, |notification| {
                    let body = notification.body().as_bytes();
                    for (idx, line) in body[0..body.len()].chunks(128 / 6).enumerate() { // screen pixels / character width
                        //TODO remove unsafe
                        write!(self.buffer, "{}", unsafe { core::str::from_utf8_unchecked(line) }).unwrap();
                        display.draw(horizontal_centre(Font6x12::render_str(self.buffer.as_str()), idx as i32 * 12)
                                .with_stroke(Some(0x02D4_u16.into()))
                                .into_iter(),
                        );
                        self.buffer.clear();
                    }
                    
                });
            }
        }
        None     
    }

    fn input(&mut self, system: &mut System, _display: &mut Ssd1351, input: InputEvent) -> Option<Signal> {
        if input == InputEvent::Multi {
            self.stop(system);
            return Some(Signal::Home) // signal to dm to go home
        }
        self.menu.update_count(system.nm().idx() as i8 + 1);
        match self.state {
            InternalState::Menu => {
                match input {
                    InputEvent::Left => {
                        self.menu.prev();
                    },
                    InputEvent::Right => {
                        self.menu.next();
                    },
                    InputEvent::Middle => {
                        self.state = InternalState::Body;
                    }
                    _ => {}
                }
                info!("In menu input {:?}", self.menu);
            },
            InternalState::Body => {
                match input {
                    InputEvent::Middle => {
                        self.state = InternalState::Menu;
                    }
                    _ => {}
                }
            }
        }
        None
    }
}

impl ScopedState for NotificationState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, _system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        self.buffer.clear();
        write!(self.buffer, "Notifications").unwrap(); 
        display.draw(horizontal_centre(Font6x12::render_str(self.buffer.as_str()), 24)
                .with_stroke(Some(0x02D4_u16.into()))
                .into_iter(),
        );
        None
    }

    fn is_running(&self, _system: &mut System) -> bool {
        self.is_running
    }

    /// Start 
    fn start(&mut self, _system: &mut System) {
        self.is_running = true;
    }

    /// Stop
    fn stop(&mut self, _system: &mut System) {
        self.is_running = false;
    }
}