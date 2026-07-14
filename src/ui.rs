//! Window construction, month/year pickers, and keyboard interaction.

use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Datelike, Local, NaiveDate};
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::date;

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Days,
    Months,
    Years,
}

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
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, false);
    }

    let month_label = gtk4::Label::new(None);
    month_label.add_css_class("waycal-header");
    month_label.set_halign(gtk4::Align::End);
    month_label.add_css_class("waycal-clickable");

    let year_label = gtk4::Label::new(None);
    year_label.add_css_class("waycal-header");
    year_label.set_halign(gtk4::Align::Start);
    year_label.add_css_class("waycal-clickable");

    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    header.set_halign(gtk4::Align::Center);
    header.append(&month_label);
    header.append(&year_label);

    let grid = gtk4::Grid::new();
    grid.set_row_spacing(2);
    grid.set_column_spacing(2);
    grid.set_halign(gtk4::Align::Center);

    let footer = gtk4::Label::new(Some("hjkl nav   t today   ? help"));
    footer.add_css_class("waycal-footer");
    footer.set_halign(gtk4::Align::Center);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    root.add_css_class("waycal-root");
    root.append(&header);
    root.append(&grid);
    root.append(&footer);
    window.set_child(Some(&root));

    let state = Rc::new(RefCell::new(Local::now().date_naive()));
    let view = Rc::new(RefCell::new(ViewMode::Days));
    let cursor = Rc::new(RefCell::new(0usize));
    let help_popover: Rc<RefCell<Option<gtk4::Popover>>> = Rc::new(RefCell::new(None));
    render_days(&grid, &month_label, &year_label, *state.borrow());

    // Click month label → month picker
    {
        let state = state.clone();
        let view = view.clone();
        let cursor = cursor.clone();
        let grid = grid.clone();
        let ml = month_label.clone();
        let yl = year_label.clone();
        let ft = footer.clone();
        let click = gtk4::GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            *view.borrow_mut() = ViewMode::Months;
            *cursor.borrow_mut() = state.borrow().month() as usize - 1;
            render_months(&grid, &ml, &yl, &ft, &state, &view, &cursor, *state.borrow());
        });
        month_label.add_controller(click);
    }

    // Click year label → year picker
    {
        let state = state.clone();
        let view = view.clone();
        let cursor = cursor.clone();
        let grid = grid.clone();
        let ml = month_label.clone();
        let yl = year_label.clone();
        let ft = footer.clone();
        let click = gtk4::GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            *view.borrow_mut() = ViewMode::Years;
            *cursor.borrow_mut() = 0;
            render_years(&grid, &ml, &yl, &ft, &state, &view, &cursor, *state.borrow());
        });
        year_label.add_controller(click);
    }

    let key = gtk4::EventControllerKey::new();
    {
        let state = state.clone();
        let view = view.clone();
        let cursor = cursor.clone();
        let help_popover = help_popover.clone();
        let grid = grid.clone();
        let month_label = month_label.clone();
        let year_label = year_label.clone();
        let footer = footer.clone();
        let window = window.clone();
        key.connect_key_pressed(move |_, keyval, _, _| {
            // q always closes help popover first if open
            if (keyval == gdk::Key::q || keyval == gdk::Key::Q)
                && let Some(pop) = help_popover.borrow_mut().take()
            {
                pop.popdown();
                pop.unparent();
                return glib::Propagation::Stop;
            }

            let current_view = *view.borrow();
            match current_view {
                ViewMode::Days => {
                    let current = *state.borrow();
                    match keyval {
                        gdk::Key::Left | gdk::Key::h => {
                            let next = date::shift_days(current, -1);
                            *state.borrow_mut() = next;
                            render_days(&grid, &month_label, &year_label, next);
                        }
                        gdk::Key::Right | gdk::Key::l => {
                            let next = date::shift_days(current, 1);
                            *state.borrow_mut() = next;
                            render_days(&grid, &month_label, &year_label, next);
                        }
                        gdk::Key::Up | gdk::Key::k => {
                            let next = date::shift_days(current, -7);
                            *state.borrow_mut() = next;
                            render_days(&grid, &month_label, &year_label, next);
                        }
                        gdk::Key::Down | gdk::Key::j => {
                            let next = date::shift_days(current, 7);
                            *state.borrow_mut() = next;
                            render_days(&grid, &month_label, &year_label, next);
                        }
                        gdk::Key::Page_Up => {
                            let next = date::shift_month(current, -1);
                            *state.borrow_mut() = next;
                            render_days(&grid, &month_label, &year_label, next);
                        }
                        gdk::Key::Page_Down => {
                            let next = date::shift_month(current, 1);
                            *state.borrow_mut() = next;
                            render_days(&grid, &month_label, &year_label, next);
                        }
                        gdk::Key::t | gdk::Key::T => {
                            *state.borrow_mut() = Local::now().date_naive();
                            render_days(&grid, &month_label, &year_label, Local::now().date_naive());
                        }
                        gdk::Key::m | gdk::Key::M => {
                            *view.borrow_mut() = ViewMode::Months;
                            *cursor.borrow_mut() = current.month() as usize - 1;
                            render_months(&grid, &month_label, &year_label, &footer, &state, &view, &cursor, current);
                        }
                        gdk::Key::y | gdk::Key::Y => {
                            *view.borrow_mut() = ViewMode::Years;
                            *cursor.borrow_mut() = 0;
                            render_years(&grid, &month_label, &year_label, &footer, &state, &view, &cursor, current);
                        }
                        gdk::Key::question => {
                            let pop = show_help(&window);
                            *help_popover.borrow_mut() = Some(pop);
                        }
                        gdk::Key::Return | gdk::Key::KP_Enter => {
                            copy_date(current);
                            window.close();
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::q | gdk::Key::Q => {
                            window.close();
                            return glib::Propagation::Stop;
                        }
                        _ => return glib::Propagation::Proceed,
                    };
                }
                ViewMode::Months => {
                    let c = *cursor.borrow();
                    match keyval {
                        gdk::Key::Left | gdk::Key::h => {
                            if c > 0 { *cursor.borrow_mut() = c - 1; }
                        }
                        gdk::Key::Right | gdk::Key::l => {
                            if c < 11 { *cursor.borrow_mut() = c + 1; }
                        }
                        gdk::Key::Up | gdk::Key::k => {
                            if c >= 4 { *cursor.borrow_mut() = c - 4; }
                        }
                        gdk::Key::Down | gdk::Key::j => {
                            if c < 8 { *cursor.borrow_mut() = c + 4; }
                        }
                        gdk::Key::Return | gdk::Key::KP_Enter => {
                            let m = (c + 1) as u32;
                            let current = *state.borrow();
                            let day = current.day().min(date::days_in_month(current.year(), m));
                            let next = NaiveDate::from_ymd_opt(current.year(), m, day).unwrap();
                            *state.borrow_mut() = next;
                            *view.borrow_mut() = ViewMode::Days;
                            render_days(&grid, &month_label, &year_label, next);
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::q | gdk::Key::Q => {
                            *view.borrow_mut() = ViewMode::Days;
                            render_days(&grid, &month_label, &year_label, *state.borrow());
                            return glib::Propagation::Stop;
                        }
                        _ => return glib::Propagation::Proceed,
                    }
                    render_months(&grid, &month_label, &year_label, &footer, &state, &view, &cursor, *state.borrow());
                }
                ViewMode::Years => {
                    let c = *cursor.borrow();
                    match keyval {
                        gdk::Key::Left | gdk::Key::h => {
                            if c > 0 { *cursor.borrow_mut() = c - 1; }
                        }
                        gdk::Key::Right | gdk::Key::l => {
                            if c < 11 { *cursor.borrow_mut() = c + 1; }
                        }
                        gdk::Key::Up | gdk::Key::k => {
                            if c >= 4 { *cursor.borrow_mut() = c - 4; }
                        }
                        gdk::Key::Down | gdk::Key::j => {
                            if c < 8 { *cursor.borrow_mut() = c + 4; }
                        }
                        gdk::Key::Return | gdk::Key::KP_Enter => {
                            let current = *state.borrow();
                            let base_year = (current.year() / 12) * 12;
                            let y = base_year + c as i32;
                            let day = current.day().min(date::days_in_month(y, current.month()));
                            let next = NaiveDate::from_ymd_opt(y, current.month(), day).unwrap();
                            *state.borrow_mut() = next;
                            *view.borrow_mut() = ViewMode::Days;
                            render_days(&grid, &month_label, &year_label, next);
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::q | gdk::Key::Q => {
                            *view.borrow_mut() = ViewMode::Days;
                            render_days(&grid, &month_label, &year_label, *state.borrow());
                            return glib::Propagation::Stop;
                        }
                        _ => return glib::Propagation::Proceed,
                    }
                    render_years(&grid, &month_label, &year_label, &footer, &state, &view, &cursor, *state.borrow());
                }
            }
            glib::Propagation::Stop
        });
    }
    window.add_controller(key);

    window.present();
}

fn show_help(parent: &gtk4::ApplicationWindow) -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    popover.set_parent(parent);
    popover.set_autohide(false);
    popover.set_has_arrow(false);
    popover.set_halign(gtk4::Align::Center);
    popover.set_valign(gtk4::Align::Center);

    let text = gtk4::Label::new(None);
    text.set_use_markup(true);
    text.set_markup(
        "<b>Day view</b>\n\
         h j k l / arrows   navigate days\n\
         Page Up/Down       prev / next month\n\
         m                  month picker\n\
         y                  year picker\n\
         t                  jump to today\n\
         Enter              copy date\n\
         q                  close\n\
         ?                  this help\n\n\
         <b>Month / Year picker</b>\n\
         h j k l / arrows   navigate\n\
         Enter              select\n\
         q                  back to day view",
    );
    text.set_halign(gtk4::Align::Start);
    text.set_margin_start(24);
    text.set_margin_end(24);
    text.set_margin_top(16);
    text.set_margin_bottom(16);

    popover.set_child(Some(&text));
    popover.popup();
    popover
}

fn render_days(grid: &gtk4::Grid, month_label: &gtk4::Label, year_label: &gtk4::Label, selected: NaiveDate) {
    month_label.set_text(date::month_name(selected.month()));
    year_label.set_text(&selected.year().to_string());

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

#[allow(clippy::too_many_arguments)]
fn render_months(
    grid: &gtk4::Grid,
    month_label: &gtk4::Label,
    year_label: &gtk4::Label,
    footer: &gtk4::Label,
    state: &Rc<RefCell<NaiveDate>>,
    view: &Rc<RefCell<ViewMode>>,
    cursor: &Rc<RefCell<usize>>,
    selected: NaiveDate,
) {
    month_label.set_text(&selected.year().to_string());
    year_label.set_text("");
    footer.set_text("hjkl nav   Enter select   q back");

    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let cur = *cursor.borrow();

    for (i, name) in months.iter().enumerate() {
        let col = (i % 4) as i32;
        let row = i / 4;
        let lbl = gtk4::Label::new(Some(name));
        lbl.add_css_class("waycal-month");
        if i == cur {
            lbl.add_css_class("selected");
        }
        {
            let state = state.clone();
            let view = view.clone();
            let grid = grid.clone();
            let month_label = month_label.clone();
            let year_label = year_label.clone();
            let m = (i + 1) as u32;
            let click = gtk4::GestureClick::new();
            click.connect_pressed(move |_, _, _, _| {
                let current = *state.borrow();
                let day = current.day().min(date::days_in_month(current.year(), m));
                let next = NaiveDate::from_ymd_opt(current.year(), m, day).unwrap();
                *state.borrow_mut() = next;
                *view.borrow_mut() = ViewMode::Days;
                render_days(&grid, &month_label, &year_label, next);
            });
            lbl.add_controller(click);
        }
        grid.attach(&lbl, col, row as i32, 1, 1);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_years(
    grid: &gtk4::Grid,
    month_label: &gtk4::Label,
    year_label: &gtk4::Label,
    footer: &gtk4::Label,
    state: &Rc<RefCell<NaiveDate>>,
    view: &Rc<RefCell<ViewMode>>,
    cursor: &Rc<RefCell<usize>>,
    selected: NaiveDate,
) {
    let base_year = (selected.year() / 12) * 12;
    month_label.set_text(&format!("{}–{}", base_year, base_year + 11));
    year_label.set_text("");
    footer.set_text("hjkl nav   Enter select   q back");

    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }

    let cur = *cursor.borrow();
    for i in 0..12 {
        let y = base_year + i;
        let col = i % 4;
        let row = i / 4;
        let lbl = gtk4::Label::new(Some(&y.to_string()));
        lbl.add_css_class("waycal-year");
        if i as usize == cur {
            lbl.add_css_class("selected");
        }
        {
            let state = state.clone();
            let view = view.clone();
            let grid = grid.clone();
            let month_label = month_label.clone();
            let year_label = year_label.clone();
            let click = gtk4::GestureClick::new();
            click.connect_pressed(move |_, _, _, _| {
                let current = *state.borrow();
                let day = current.day().min(date::days_in_month(y, current.month()));
                let next = NaiveDate::from_ymd_opt(y, current.month(), day).unwrap();
                *state.borrow_mut() = next;
                *view.borrow_mut() = ViewMode::Days;
                render_days(&grid, &month_label, &year_label, next);
            });
            lbl.add_controller(click);
        }
        grid.attach(&lbl, col, row, 1, 1);
    }
}
