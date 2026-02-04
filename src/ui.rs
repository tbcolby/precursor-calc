//! Drawing utilities and layout constants

use gam::menu::*;
use gam::{Gam, GlyphStyle};

// Screen dimensions (Precursor)
pub const SCREEN_WIDTH: isize = 336;
pub const SCREEN_HEIGHT: isize = 536;

// Layout constants
pub const MARGIN: isize = 4;
pub const STATUS_HEIGHT: isize = 20;
pub const DISPLAY_HEIGHT: isize = 120;
pub const HISTORY_HEIGHT: isize = 200;
pub const MENU_HEIGHT: isize = 24;

// Colors
pub fn dark_style() -> DrawStyle {
    DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1)
}

pub fn light_style() -> DrawStyle {
    DrawStyle::new(PixelColor::Light, PixelColor::Light, 1)
}

pub fn outline_style() -> DrawStyle {
    DrawStyle {
        fill_color: None,
        stroke_color: Some(PixelColor::Dark),
        stroke_width: 1,
    }
}

/// Clear the entire screen
pub fn clear_screen(gam: &Gam, gid: gam::Gid) {
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, 0),
            Point::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            light_style(),
        ),
    )
    .ok();
}

/// Draw a horizontal separator line
pub fn draw_separator(gam: &Gam, gid: gam::Gid, y: isize) {
    gam.draw_line(
        gid,
        Line::new_with_style(
            Point::new(MARGIN, y),
            Point::new(SCREEN_WIDTH - MARGIN, y),
            dark_style(),
        ),
    )
    .ok();
}

/// Draw status bar at top
pub fn draw_status_bar(
    gam: &Gam,
    gid: gam::Gid,
    mode_label: &str,
    angle_label: &str,
    base_label: &str,
    has_memory: bool,
) {
    // Clear status area
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, 0),
            Point::new(SCREEN_WIDTH, STATUS_HEIGHT),
            light_style(),
        ),
    )
    .ok();

    // Mode indicator [ALG] or [RPN]
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(MARGIN, 2, 60, STATUS_HEIGHT)),
    );
    tv.style = GlyphStyle::Bold;
    tv.draw_border = true;
    tv.border_width = 1;
    tv.margin = Point::new(2, 0);
    use core::fmt::Write;
    write!(tv.text, "{}", mode_label).ok();
    gam.post_textview(&mut tv).ok();

    // Angle mode [DEG]/[RAD]/[GRAD]
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(65, 2, 120, STATUS_HEIGHT)),
    );
    tv.style = GlyphStyle::Small;
    tv.draw_border = true;
    tv.border_width = 1;
    tv.margin = Point::new(2, 0);
    write!(tv.text, "{}", angle_label).ok();
    gam.post_textview(&mut tv).ok();

    // Base [DEC]/[HEX]/[OCT]/[BIN]
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(125, 2, 175, STATUS_HEIGHT)),
    );
    tv.style = GlyphStyle::Small;
    tv.draw_border = true;
    tv.border_width = 1;
    tv.margin = Point::new(2, 0);
    write!(tv.text, "{}", base_label).ok();
    gam.post_textview(&mut tv).ok();

    // Memory indicator
    if has_memory {
        let mut tv = TextView::new(
            gid,
            TextBounds::BoundingBox(Rectangle::new_coords(SCREEN_WIDTH - 40, 2, SCREEN_WIDTH - MARGIN, STATUS_HEIGHT)),
        );
        tv.style = GlyphStyle::Small;
        write!(tv.text, "M").ok();
        gam.post_textview(&mut tv).ok();
    }

    draw_separator(gam, gid, STATUS_HEIGHT);
}

/// Draw the main display area (algebraic mode)
pub fn draw_algebraic_display(
    gam: &Gam,
    gid: gam::Gid,
    expression: &str,
    result: &str,
    error: Option<&str>,
) {
    let y_start = STATUS_HEIGHT + 2;
    let y_end = STATUS_HEIGHT + DISPLAY_HEIGHT;

    // Clear display area
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, y_start),
            Point::new(SCREEN_WIDTH, y_end),
            light_style(),
        ),
    )
    .ok();

    // Expression (right-aligned, regular size)
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(
            MARGIN,
            y_start + 5,
            SCREEN_WIDTH - MARGIN,
            y_start + 30,
        )),
    );
    tv.style = GlyphStyle::Regular;
    use core::fmt::Write;
    // Right-align by padding
    let expr_display = if expression.is_empty() { "0" } else { expression };
    write!(tv.text, "{}_", expr_display).ok();
    gam.post_textview(&mut tv).ok();

    // Result or error (right-aligned, large)
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(
            MARGIN,
            y_start + 40,
            SCREEN_WIDTH - MARGIN,
            y_end - 5,
        )),
    );

    if let Some(err) = error {
        tv.style = GlyphStyle::Bold;
        write!(tv.text, "{}", err).ok();
    } else {
        tv.style = GlyphStyle::Large;
        write!(tv.text, "= {}", result).ok();
    }
    gam.post_textview(&mut tv).ok();

    draw_separator(gam, gid, y_end);
}

/// Draw RPN stack display
pub fn draw_rpn_display(
    gam: &Gam,
    gid: gam::Gid,
    stack: [&str; 4], // [X, Y, Z, T]
    entry: &str,
    entering: bool,
    last_x: &str,
    error: Option<&str>,
) {
    let y_start = STATUS_HEIGHT + 2;
    let y_end = STATUS_HEIGHT + DISPLAY_HEIGHT;

    // Clear display area
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, y_start),
            Point::new(SCREEN_WIDTH, y_end),
            light_style(),
        ),
    )
    .ok();

    use core::fmt::Write;

    // Stack label
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(MARGIN, y_start + 2, 60, y_start + 16)),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "Stack:").ok();
    gam.post_textview(&mut tv).ok();

    // T register
    draw_stack_register(gam, gid, "T:", stack[3], y_start + 18, false);
    // Z register
    draw_stack_register(gam, gid, "Z:", stack[2], y_start + 34, false);
    // Y register
    draw_stack_register(gam, gid, "Y:", stack[1], y_start + 50, false);

    // X register (current entry, highlighted)
    let x_display = if entering {
        entry
    } else {
        stack[0]
    };
    draw_stack_register(gam, gid, "X:", x_display, y_start + 66, true);

    // Error display
    if let Some(err) = error {
        let mut tv = TextView::new(
            gid,
            TextBounds::BoundingBox(Rectangle::new_coords(MARGIN, y_start + 85, SCREEN_WIDTH - MARGIN, y_end - 5)),
        );
        tv.style = GlyphStyle::Bold;
        write!(tv.text, "{}", err).ok();
        gam.post_textview(&mut tv).ok();
    } else {
        // LastX
        let mut tv = TextView::new(
            gid,
            TextBounds::BoundingBox(Rectangle::new_coords(MARGIN, y_start + 85, SCREEN_WIDTH - MARGIN, y_end - 5)),
        );
        tv.style = GlyphStyle::Small;
        write!(tv.text, "LastX: {}", last_x).ok();
        gam.post_textview(&mut tv).ok();
    }

    draw_separator(gam, gid, y_end);
}

/// Draw a single stack register line
fn draw_stack_register(gam: &Gam, gid: gam::Gid, label: &str, value: &str, y: isize, highlight: bool) {
    use core::fmt::Write;

    // Label
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(MARGIN, y, MARGIN + 20, y + 14)),
    );
    tv.style = if highlight { GlyphStyle::Bold } else { GlyphStyle::Small };
    write!(tv.text, "{}", label).ok();
    gam.post_textview(&mut tv).ok();

    // Value (right side)
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(MARGIN + 24, y, SCREEN_WIDTH - MARGIN, y + 14)),
    );
    tv.style = if highlight { GlyphStyle::Regular } else { GlyphStyle::Small };
    if highlight {
        write!(tv.text, "{}_", value).ok();
    } else {
        write!(tv.text, "{}", value).ok();
    }
    gam.post_textview(&mut tv).ok();
}

/// Draw history tape
pub fn draw_history(gam: &Gam, gid: gam::Gid, entries: &[&str]) {
    let y_start = STATUS_HEIGHT + DISPLAY_HEIGHT + 4;
    let y_end = SCREEN_HEIGHT - MENU_HEIGHT - 4;

    // Clear history area
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, y_start),
            Point::new(SCREEN_WIDTH, y_end),
            light_style(),
        ),
    )
    .ok();

    use core::fmt::Write;

    // History label
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(MARGIN, y_start, 80, y_start + 14)),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "History:").ok();
    gam.post_textview(&mut tv).ok();

    // History entries
    let y = y_start + 16;
    let line_height = 14;
    let max_entries = ((y_end - y) / line_height) as usize;

    for (i, entry) in entries.iter().take(max_entries).enumerate() {
        let mut tv = TextView::new(
            gid,
            TextBounds::BoundingBox(Rectangle::new_coords(
                MARGIN + 4,
                y + (i as isize) * line_height,
                SCREEN_WIDTH - MARGIN,
                y + (i as isize + 1) * line_height,
            )),
        );
        tv.style = GlyphStyle::Small;
        write!(tv.text, "{}", entry).ok();
        gam.post_textview(&mut tv).ok();
    }

    draw_separator(gam, gid, y_end);
}

/// Draw function menu bar at bottom
pub fn draw_menu_bar(gam: &Gam, gid: gam::Gid) {
    let y = SCREEN_HEIGHT - MENU_HEIGHT;

    // Clear menu area
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, y),
            Point::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            light_style(),
        ),
    )
    .ok();

    use core::fmt::Write;

    // F1-F4 labels
    let labels = ["F1:MATH", "F2:TRIG", "F3:MODE", "F4:MEM"];
    let width = SCREEN_WIDTH / 4;

    for (i, label) in labels.iter().enumerate() {
        let x = (i as isize) * width;
        let mut tv = TextView::new(
            gid,
            TextBounds::BoundingBox(Rectangle::new_coords(x + 2, y + 4, x + width - 2, SCREEN_HEIGHT - 2)),
        );
        tv.style = GlyphStyle::Small;
        tv.draw_border = true;
        tv.border_width = 1;
        tv.margin = Point::new(2, 2);
        write!(tv.text, "{}", label).ok();
        gam.post_textview(&mut tv).ok();
    }
}

/// Draw function menu overlay
pub fn draw_fn_menu(gam: &Gam, gid: gam::Gid, title: &str, items: &[(&str, &str)]) {
    let menu_width = 280;
    let menu_height = 160;
    let x = (SCREEN_WIDTH - menu_width) / 2;
    let y = (SCREEN_HEIGHT - menu_height) / 2;

    // Background
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(x, y),
            Point::new(x + menu_width, y + menu_height),
            light_style(),
        ),
    )
    .ok();

    // Border
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(x, y),
            Point::new(x + menu_width, y + menu_height),
            outline_style(),
        ),
    )
    .ok();

    use core::fmt::Write;

    // Title
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(x + 4, y + 4, x + menu_width - 4, y + 22)),
    );
    tv.style = GlyphStyle::Bold;
    write!(tv.text, "[{}]", title).ok();
    gam.post_textview(&mut tv).ok();

    // Draw separator
    gam.draw_line(
        gid,
        Line::new_with_style(
            Point::new(x + 4, y + 24),
            Point::new(x + menu_width - 4, y + 24),
            dark_style(),
        ),
    )
    .ok();

    // Menu items in 3 columns
    let col_width = (menu_width - 8) / 3;
    let line_height = 18;
    let start_y = y + 28;

    for (i, (key, label)) in items.iter().enumerate() {
        let col = (i % 3) as isize;
        let row = (i / 3) as isize;
        let item_x = x + 4 + col * col_width;
        let item_y = start_y + row * line_height;

        let mut tv = TextView::new(
            gid,
            TextBounds::BoundingBox(Rectangle::new_coords(item_x, item_y, item_x + col_width, item_y + line_height)),
        );
        tv.style = GlyphStyle::Small;
        write!(tv.text, "{}: {}", key, label).ok();
        gam.post_textview(&mut tv).ok();
    }

    // Cancel hint
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(x + 4, y + menu_height - 20, x + menu_width - 4, y + menu_height - 4)),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "Press 0-9 or ESC to cancel").ok();
    gam.post_textview(&mut tv).ok();
}
