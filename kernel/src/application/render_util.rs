//! Userspace utils 
//! 
//! Helpful functions for rendering etc

use embedded_graphics::prelude::*;

pub const DISPLAY_CENTRE: i32 = 64;
pub const DISPLAY_WIDTH: i32 = 128;
pub const DISPLAY_HEIGHT: i32 = 128;


pub fn horizontal_centre<F>(text: F, y: i32) -> F
    where F: Dimensions + Transform
{
    text.translate(Coord::new(DISPLAY_CENTRE - text.size().0 as i32 / 2, y))
}

pub fn vertical_centre<F>(text: F, x: i32) -> F
    where F: Dimensions + Transform
{
    text.translate(Coord::new(x, DISPLAY_CENTRE - text.size().1 as i32 / 2))
}

pub fn centre<F>(text: F) -> F
    where F: Dimensions + Transform
{
    text.translate(Coord::new(DISPLAY_CENTRE - text.size().0 as i32 / 2, DISPLAY_CENTRE - text.size().1 as i32 / 2))
}