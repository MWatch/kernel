//! Notification state
//!
//! A simple notification manager
//!  

use crate::application::FrameBuffer;
use crate::application::states::prelude::*;
use crate::system::input::InputEvent;
use crate::system::{System, Host};

use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Text, Alignment, Baseline};

use crate::system::notification::Notification;

const DISPLAY_HEIGHT: i32 = 128; // TODO
const CHAR_WIDTH: i32 = 6;
const CHAR_HEIGHT: i32 = 12;
const LINE_WIDTH: i32 = DISPLAY_HEIGHT / CHAR_WIDTH;

#[derive(Debug, Copy, Clone, PartialEq)]
/// The internal state of the notification application
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
    /// Render the notification state
    fn render(&mut self, system: &mut System<impl Host>, display: &mut FrameBuffer) -> Option<Signal> {
        self.menu.update_count(system.nm.idx() as i8);
        match self.state {
            InternalState::Menu => {
                let size = display.bounding_box().size;
                let style = MonoTextStyle::new(&FONT_6X12, RawU16::from(0x02D4).into());

                if system.nm.idx() > 0 {
                    // Display a selection indicator
                    Text::with_baseline(
                        ">",
                        Point::new(0, self.menu.selected() as i32 * CHAR_HEIGHT),
                        style,
                        Baseline::Top,
                    )
                    .draw(display).ok();
                    for item in 0..system.nm.idx() {
                        system.nm.peek_notification(item, |notification| {
                            Text::with_baseline(
                                notification.title(),
                                Point::new(size.width as i32 / 2, item as i32 * CHAR_HEIGHT),
                                style,
                                Baseline::Top,
                            )
                            .draw(display).ok();
                        });
                    }
                } else {
                    Text::with_alignment(
                        "Nothing to display!",
                        Point::new(size.width as i32 / 2, size.height as i32 / 2),
                        style,
                        Alignment::Center,
                    )
                    .draw(display).ok();
                }
            }
            InternalState::Body => {
                system
                    .nm
                    .peek_notification(self.menu.selected() as usize, |notification| {
                        self.body.render(display, notification);
                    });
            }
        }
        None
    }

    /// Handle the input for the notification
    fn input(&mut self, system: &mut System<impl Host>, input: InputEvent) -> Option<Signal> {
        if input == InputEvent::Multi {
            self.stop(system);
            return Some(Signal::Home); // signal to dm to go home
        }
        self.menu.update_count(system.nm.idx() as i8);
        match self.state {
            InternalState::Menu => {
                if system.nm.idx() > 0 {
                    match input {
                        InputEvent::Left => {
                            self.menu.prev();
                        }
                        InputEvent::Right => {
                            self.menu.next();
                        }
                        InputEvent::Middle => {
                            self.state = InternalState::Body;
                            system.nm.peek_notification(
                                self.menu.selected() as usize,
                                |notification| {
                                    let line_count = notification.body().len() as i32 / LINE_WIDTH;
                                    self.body = Body::new(line_count - line_count / 2);
                                },
                            );
                        }
                        _ => {}
                    }
                } else {
                    self.stop(system);
                }
            }
            InternalState::Body => match input {
                InputEvent::Middle => {
                    self.state = InternalState::Menu;
                }
                InputEvent::Left => {
                    self.body.down();
                }
                InputEvent::Right => {
                    self.body.up();
                }
                _ => {}
            },
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
            body: Body::new(0),
        }
    }
}

impl ScopedState for NotificationState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, _system: &mut System<impl Host>, display: &mut FrameBuffer) -> Option<Signal> {
        let size = display.bounding_box().size;
        let style = MonoTextStyle::new(&FONT_6X12, RawU16::from(0x02D4).into());
        Text::with_alignment(
            "Notifications",
            Point::new(size.width as i32 / 2, size.height as i32 / 2),
            style,
            Alignment::Center
        )
        .draw(display).ok();
        None
    }

    /// Is the notification app opened?
    fn is_running(&self, _system: &mut System<impl Host>) -> bool {
        self.is_running
    }

    /// Start
    fn start(&mut self, _system: &mut System<impl Host>) {
        self.is_running = true;
    }

    /// Stop
    fn stop(&mut self, _system: &mut System<impl Host>) {
        self.is_running = false;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Body {
    scroll_y: i32,
    max_scroll_y: i32,
}

impl Body {
    /// Create a new body, with a maximum vertical scroll
    pub fn new(max_scroll: i32) -> Self {
        let max_scroll = if max_scroll < (DISPLAY_HEIGHT / CHAR_HEIGHT) / 2 {
            // no scroll required if it doesnt go past a page
            0
        } else {
            max_scroll
        };
        info!("Creating body with max scroll of {}", -max_scroll);
        Body {
            scroll_y: 0,
            max_scroll_y: -max_scroll,
        }
    }

    /// Render the notification
    pub fn render(&mut self, display: &mut FrameBuffer, notification: &Notification) {
        let body = notification.body().as_bytes();
        for (idx, line) in body[0..body.len()].chunks(LINE_WIDTH as usize).enumerate() {
            // screen pixels / character width
            // safe because the protocol guarentees no unicode bytes will be sent
            let style = MonoTextStyle::new(&FONT_6X12, RawU16::from(0x02D4).into());
            Text::with_baseline(
                unsafe { core::str::from_utf8_unchecked(line) },
                Point::new(0, ((idx as i32) + self.scroll_y) * CHAR_HEIGHT),
                style,
                Baseline::Top
            )
            .draw(display).ok();
        }
    }

    /// Move to the previous line
    fn up(&mut self) {
        self.scroll_y -= 1;
        if self.scroll_y < self.max_scroll_y {
            self.scroll_y = self.max_scroll_y;
        }
    }

    /// Move to the next line
    fn down(&mut self) {
        self.scroll_y += 1;
        if self.scroll_y > 0 {
            self.scroll_y = 0;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Menu {
    state_idx: i8,
    item_count: i8,
}

impl Menu {
    /// create a new menu
    pub const fn new() -> Self {
        Menu {
            state_idx: 0,
            item_count: 0,
        }
    }

    /// Move to the previous item in a wrapping fashion
    fn prev(&mut self) {
        self.state_idx -= 1;
        if self.state_idx < 0 {
            self.state_idx = self.item_count - 1;
        }
    }

    /// Move to the next item in a wrapping fashion
    fn next(&mut self) {
        self.state_idx += 1;
        if self.state_idx > self.item_count - 1 {
            self.state_idx = 0;
        }
    }

    /// The currently selected index
    fn selected(&self) -> i8 {
        self.state_idx
    }

    /// Update the number of elements in the list
    fn update_count(&mut self, item_count: i8) {
        self.item_count = item_count;
    }
}
