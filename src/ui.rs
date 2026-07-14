//! Window construction, month rendering, and keyboard interaction.

use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Datelike, Local, NaiveDate};
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::date;

// Copy the selected date to the Wayland clipboard via wl-copy. wl-copy forks a
// background process to serve the selection, so it survives this app exiting;
// notify-send gives visible confirmation.
fn copy_date(date: NaiveDate) {
    let text = date.format(date::COPY_FORMAT).to_string();
    let _ = std::process::Command::new("wl-copy").arg(&text).spawn();
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "waycal", "-t", "2000", "Copied", &text])
        .spawn();
}

/// Build and present the calendar popup window.
pub fn build(app: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(app);
    window.set_decorated(false);
    window.set_resizable(false);
    window.add_css_class("waycal");

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    // Exclusive so the popup grabs the keyboard the moment it opens; OnDemand
    // only grants focus on a pointer click, which never happens when launched
    // from a compositor keybind (arrows/Enter/Esc would go nowhere).
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    // Anchor nothing: with no edges set, the layer-shell compositor centers the
    // surface on the output.
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, false);
    }

    let header = gtk4::Label::new(None);
    header.add_css_class("waycal-header");
    header.set_halign(gtk4::Align::Center);

    let grid = gtk4::Grid::new();
    grid.set_row_spacing(2);
    grid.set_column_spacing(2);
    grid.set_halign(gtk4::Align::Center);

    let footer = gtk4::Label::new(Some(
        "\u{2190}\u{2192} day   \u{2191}\u{2193} week   \u{21DF}\u{21DE} mo   \u{23CE} copy   t today",
    ));
    footer.add_css_class("waycal-footer");
    footer.set_halign(gtk4::Align::Center);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    root.add_css_class("waycal-root");
    root.append(&header);
    root.append(&grid);
    root.append(&footer);
    window.set_child(Some(&root));

    let state = Rc::new(RefCell::new(Local::now().date_naive()));
    render(&grid, &header, *state.borrow());

    let key = gtk4::EventControllerKey::new();
    {
        let state = state.clone();
        let grid = grid.clone();
        let header = header.clone();
        let window = window.clone();
        key.connect_key_pressed(move |_, keyval, _, _| {
            let current = *state.borrow();
            let next = match keyval {
                gdk::Key::Left => date::shift_days(current, -1),
                gdk::Key::Right => date::shift_days(current, 1),
                gdk::Key::Up => date::shift_days(current, -7),
                gdk::Key::Down => date::shift_days(current, 7),
                gdk::Key::Page_Up => date::shift_month(current, -1),
                gdk::Key::Page_Down => date::shift_month(current, 1),
                gdk::Key::t | gdk::Key::T => Local::now().date_naive(),
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    copy_date(current);
                    window.close();
                    return glib::Propagation::Stop;
                }
                gdk::Key::Escape => {
                    window.close();
                    return glib::Propagation::Stop;
                }
                _ => return glib::Propagation::Proceed,
            };
            *state.borrow_mut() = next;
            render(&grid, &header, next);
            glib::Propagation::Stop
        });
    }
    window.add_controller(key);

    window.present();
}

fn render(grid: &gtk4::Grid, header: &gtk4::Label, selected: NaiveDate) {
    header.set_text(&format!(
        "{} {}",
        date::month_name(selected.month()),
        selected.year()
    ));

    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    let weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    for (i, name) in weekdays.iter().enumerate() {
        let lbl = gtk4::Label::new(Some(name));
        lbl.add_css_class("waycal-weekday");
        grid.attach(&lbl, i as i32, 0, 1, 1);
    }

    let year = selected.year();
    let month = selected.month();
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let lead = first.weekday().num_days_from_monday() as i32;
    let days = date::days_in_month(year, month) as i32;

    let today = Local::now().date_naive();
    let is_current_month = today.year() == year && today.month() == month;
    let today_day = today.day() as i32;
    let selected_day = selected.day() as i32;

    let prev = date::shift_month(first, -1);
    let prev_days = date::days_in_month(prev.year(), prev.month()) as i32;
    for i in 0..lead {
        let day = prev_days - lead + 1 + i;
        let lbl = gtk4::Label::new(Some(&day.to_string()));
        lbl.add_css_class("waycal-day");
        lbl.add_css_class("dim");
        grid.attach(&lbl, i, 1, 1, 1);
    }

    for d in 1..=days {
        let idx = lead + d - 1;
        let col = idx % 7;
        let row = idx / 7 + 1;
        let lbl = gtk4::Label::new(Some(&d.to_string()));
        lbl.add_css_class("waycal-day");
        if is_current_month && d == today_day {
            lbl.add_css_class("today");
        }
        if d == selected_day {
            lbl.add_css_class("selected");
        }
        grid.attach(&lbl, col, row, 1, 1);
    }

    let total = lead + days;
    let trailing = (7 - total % 7) % 7;
    for i in 0..trailing {
        let day = i + 1;
        let idx = total + i;
        let col = idx % 7;
        let row = idx / 7 + 1;
        let lbl = gtk4::Label::new(Some(&day.to_string()));
        lbl.add_css_class("waycal-day");
        lbl.add_css_class("dim");
        grid.attach(&lbl, col, row, 1, 1);
    }
}
