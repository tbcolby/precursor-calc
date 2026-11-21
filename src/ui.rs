//! Drawing. Layout comes from the canvas size; nothing is hard-coded to the
//! panel resolution. One full redraw per key press (no timed redraws).

use core::fmt::Write;

use calc_core::calculator::Overlay;
use calc_core::format::ellipsize;
use calc_core::{Calculator, Mode};
use gam::menu::*;
use gam::GlyphStyle;
use precursor_appkit::screen::{Screen, MARGIN};

const FOOTER: &str = "F1 math  F2 trig  F3 more  F4 back";

/// Right-aligned single line ending at the right margin, top at `y`.
fn text_right(screen: &Screen, y: isize, style: GlyphStyle, text: &str) {
    let w = screen.size().x;
    let mut tv = TextView::new(
        screen.gid(),
        TextBounds::GrowableFromTr(Point::new(w - MARGIN, y), (w - 2 * MARGIN) as u16),
    );
    tv.style = style;
    tv.clear_area = true;
    tv.draw_border = false;
    tv.ellipsis = true;
    tv.margin = Point::new(0, 0);
    write!(tv.text, "{text}").ok();
    screen.gam().post_textview(&mut tv).ok();
}

/// Label at the left, value right-aligned on the same row.
fn row(screen: &Screen, y: isize, style: GlyphStyle, label: &str, value: &str) {
    let h = screen.line_height(style);
    screen.text_in(Point::new(MARGIN, y), Point::new(MARGIN + 40, y + h), GlyphStyle::Small, label);
    text_right(screen, y, style, value);
}

pub fn draw(screen: &Screen, calc: &Calculator, status: &str) {
    screen.clear();
    let top = screen.header(&format!("Calculator   {}", calc.status_line()));
    let mut bottom = screen.footer(FOOTER);
    let small = screen.line_height(GlyphStyle::Small);
    let regular = screen.line_height(GlyphStyle::Regular);
    let large = screen.line_height(GlyphStyle::Large);

    if !status.is_empty() {
        bottom -= small;
        screen.text_in(Point::new(MARGIN, bottom), Point::new(screen.size().x - MARGIN, bottom + small), GlyphStyle::Small, status);
    }

    // Bottom-up: main value, then the line above it, then history in what's left.
    let mut y = bottom - large;
    let history_floor = match calc.mode() {
        Mode::Algebraic => {
            match calc.error() {
                Some(e) => text_right(screen, y, GlyphStyle::Bold, e),
                None => text_right(screen, y, GlyphStyle::Large, &calc_core::format::format_number(calc.ans(), calc.base())),
            }
            y -= regular;
            let mut input = String::from(calc.input());
            input.push('_');
            // Show the end of a long expression (where the user is typing).
            let shown: String = if input.chars().count() > 38 {
                let skip = input.chars().count() - 37;
                core::iter::once('…').chain(input.chars().skip(skip)).collect()
            } else {
                input
            };
            text_right(screen, y, GlyphStyle::Regular, &shown);
            y - 4
        }
        Mode::Rpn => {
            row(screen, y, GlyphStyle::Large, "X", &calc.rpn_register(0));
            for (i, label) in [(1, "Y"), (2, "Z"), (3, "T")] {
                y -= regular;
                row(screen, y, GlyphStyle::Regular, label, &calc.rpn_register(i));
            }
            y -= small + 2;
            let last = calc_core::format::format_number(calc.rpn().last_x(), calc.base());
            match calc.error() {
                Some(e) => text_right(screen, y, GlyphStyle::Small, e),
                None => row(screen, y, GlyphStyle::Small, "LASTx", &last),
            }
            y - 4
        }
    };
    screen.line(Point::new(MARGIN, history_floor), Point::new(screen.size().x - MARGIN, history_floor));

    // History tape (algebraic results), newest nearest the display.
    let rows = ((history_floor - top - 2) / small).max(0) as usize;
    let lines = calc.history_lines(rows);
    let mut hy = history_floor - 2 - small * lines.len() as isize;
    for line in &lines {
        screen.text_in(
            Point::new(MARGIN, hy),
            Point::new(screen.size().x - MARGIN, hy + small),
            GlyphStyle::Small,
            &ellipsize(line, 48),
        );
        hy += small;
    }

    match calc.overlay() {
        Overlay::None => {}
        Overlay::Menu(menu) => {
            let items: Vec<String> = menu.items().iter().map(|i| format!("{}  {}", i.key, i.text())).collect();
            draw_box(screen, top, menu.title(), &items, "key: choose   F4/Bksp: close");
        }
        Overlay::Store => draw_box(screen, top, "Store X / ans", &[String::from("0-9  register M0-M9")], "F4/Bksp: cancel"),
        Overlay::Recall => {
            let items: Vec<String> = calc
                .memory()
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{}  {}", i, calc_core::format::format_number(*v, calc.base())))
                .collect();
            draw_box(screen, top, "Recall M0-M9", &items, "F4/Bksp: cancel");
        }
    }
    screen.flush();
}

/// A framed panel over the main display.
fn draw_box(screen: &Screen, top: isize, title: &str, items: &[String], hint: &str) {
    let w = screen.size().x;
    let reg = screen.line_height(GlyphStyle::Regular);
    let small = screen.line_height(GlyphStyle::Small);
    let height = reg + 6 + reg * items.len() as isize + small + 8;
    let tl = Point::new(MARGIN + 4, top + 4);
    let br = Point::new(w - MARGIN - 4, (top + 4 + height).min(screen.size().y - 2));
    screen.fill(tl, br, PixelColor::Light);
    screen.frame(tl, br, 2);
    let mut y = tl.y + 4;
    screen.text_in(Point::new(tl.x + 8, y), Point::new(br.x - 8, y + reg), GlyphStyle::Bold, title);
    y += reg + 4;
    for item in items {
        if y + reg > br.y - small {
            break;
        }
        screen.text_in(Point::new(tl.x + 8, y), Point::new(br.x - 8, y + reg), GlyphStyle::Regular, item);
        y += reg;
    }
    screen.text_in(Point::new(tl.x + 8, br.y - small - 2), Point::new(br.x - 8, br.y - 2), GlyphStyle::Small, hint);
}
