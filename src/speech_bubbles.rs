use std::cmp::{min, max};
use quicksilver::{
    geom::Rectangle,
    graphics::{Background::Blended, Color, Image},
    lifecycle::Window,
};
use crate::fontdata::{Glyph, GLYPH, LINE_HEIGHT};
use crate::cell_grid::Point;

pub fn guard_speech(pos: Point, s: &str) {
}

pub fn noise(pos: Point, s: &str) {
}

pub fn narration(s: &str) {
}

pub fn clear() {
}

pub fn layout(view_min: Point, view_max: Point, focus: Point) {
}

pub fn draw(/* viewport: &Viewport */) {
}

pub fn glyph_lookup(c: char) -> Option<&'static Glyph> {
    let id = c as usize;
    GLYPH.iter().find(|&glyph| glyph.id == id)
}

pub fn get_horizontal_extents(s: &str) -> (i32, i32) {
	let mut x_min = std::i32::MAX;
	let mut x_max = std::i32::MIN;
	let mut x = 0;

    for c in s.chars() {
        if let Some(glyph) = glyph_lookup(c) {
            x_min = min(x_min, x + glyph.x_offset);
            x_max = max(x_max, x + glyph.x_offset + glyph.width);
            x += glyph.x_advance;
        }
	}

    (x_min, x_max)
}

pub fn puts_proportional(window: &mut Window, font_image: &Image, mut x: i32, mut y: i32, s: &str, color: &Color) -> i32 {
	let x_base = x;

    for c in s.chars() {
		if c == '\n' {
			y -= if x == x_base {LINE_HEIGHT / 2} else {LINE_HEIGHT};
			x = x_base;
			continue;
		}

        if let Some(glyph) = glyph_lookup(c) {
            window.draw(
                &Rectangle::new((x + glyph.x_offset, y + glyph.y_offset), (glyph.width, glyph.height)),
                Blended(&font_image.subimage(Rectangle::new((glyph.x, glyph.y), (glyph.width, glyph.height))), *color)
            );

            x += glyph.x_advance;
        }
	}

	x
}
