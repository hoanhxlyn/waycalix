mod date;
mod style;
mod ui;

use gtk4::glib;
use gtk4::prelude::*;

const APP_ID: &str = "com.forrestknight.waycal";

fn main() -> glib::ExitCode {
    let app = gtk4::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| style::load());
    // Single-instance toggle: launching again (e.g. a second Mod+C) re-activates
    // the running instance instead of stacking a new window, so close the open
    // popup instead of building another one.
    app.connect_activate(|app| {
        if let Some(win) = app.windows().first() {
            win.close();
            return;
        }
        ui::build(app);
    });
    app.run()
}
