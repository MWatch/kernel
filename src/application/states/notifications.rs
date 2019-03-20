//! Application state
//!
//! Wraps the application manager in a display manager state
//!  

use crate::application::states::prelude::*;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::prelude::*;

use crate::system::notification::Notification;
use crate::application::render_util::DISPLAY_WIDTH;


const MAX_ITEMS: i8 = 8;
const CHAR_WIDTH: i32 = 6;
const CHAR_HEIGHT: i32 = 12;
const LINE_WIDTH: i32 = DISPLAY_WIDTH / CHAR_WIDTH;

#[derive(Debug, Copy, Clone, PartialEq)]
enum InternalState {
    Menu,
    Body,
}

pub struct NotificationState {
    is_running: bool,
    state: InternalState,
    menu: Menu,
    body: Body,
}

impl State for NotificationState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        self.menu.update_count(system.nm().idx() as i8 + 1);
        match self.state {
            InternalState::Menu => {
                if system.nm().idx() > 0 {
                    // Display a selection indicator
                    display.draw(Font6x12::render_str(">")
                            .translate(Coord::new(0, self.menu.selected() as i32 * CHAR_HEIGHT))
                            .with_stroke(Some(0x02D4_u16.into()))
                            .into_iter(),
                    );
                    for item in 0..system.nm().idx() {
                        system.nm().peek_notification(item, |notification| {
                            display.draw(horizontal_centre(Font6x12::render_str(notification.title()), item as i32 * CHAR_HEIGHT)
                                    .with_stroke(Some(0x02D4_u16.into()))
                                    .into_iter(),
                            );
                        });
                    }
                } else {
                    display.draw(horizontal_centre(Font6x12::render_str("Nothing to display!"), 24)
                            .with_stroke(Some(0x02D4_u16.into()))
                            .into_iter(),
                    );
                }
            },
            InternalState::Body => {
                system.nm().peek_notification(self.menu.selected() as usize, |notification| {
                    self.body.render(display, &notification);
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
                if system.nm().idx() > 0 {
                    match input {
                        InputEvent::Left => {
                            self.menu.prev();
                        },
                        InputEvent::Right => {
                            self.menu.next();
                        },
                        InputEvent::Middle => {
                            self.state = InternalState::Body;
                            system.nm().peek_notification(self.menu.selected() as usize, |notification| {
                                self.body =  Body::new(notification.body().len() as i32 / LINE_WIDTH);
                            });
                        }
                        _ => {}
                    }
                }
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

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            is_running: false,
            state: InternalState::Menu,
            menu: Menu::new(),
            body: Body::new(0)
        }
    }
}

impl ScopedState for NotificationState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, _system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        display.draw(horizontal_centre(Font6x12::render_str("Notifications"), 24)
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

#[derive(Debug, Copy, Clone, PartialEq)]
struct Body {
    scroll_y: i32,
    max_scroll_y: i32,
}

impl Body {
    
    pub const fn new(max_scroll: i32) -> Self {
        Body {
            scroll_y: 0,
            max_scroll_y: max_scroll,
        }
    }

    pub fn render(&mut self, display: &mut Ssd1351, notification: &Notification) {
        let body = notification.body().as_bytes();
        for (idx, line) in body[0..body.len()].chunks(LINE_WIDTH as usize).enumerate() { // screen pixels / character width
            // safe because the protocol guarentees no unicode bytes will be sent
            display.draw(
                    horizontal_centre(
                        Font6x12::render_str(unsafe { core::str::from_utf8_unchecked(line) }),
                        (idx as i32 + self.scroll_y) * CHAR_HEIGHT
                    )
                    .with_stroke(Some(0x02D4_u16.into()))
                    .into_iter(),
            );
        }
    }
    
}

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