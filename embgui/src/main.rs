#![windows_subsystem = "windows"]

use anyhow::bail;
use log::{debug, error};
use slint::{LogicalPosition, WindowPosition};
use config::app_config;
use errors::Error::LoadSettingsError;
use settings::settings::load_settings;
use window_spawn_util::get_screen_size;
use crate::gui_action::gui_action::{define_ui_callbacks, set_ui_settings_fields};
slint::include_modules!();
mod gui_action;


fn main() -> anyhow::Result<()> {
    let ui = AppWindow::new()?;

    let config = match load_settings() {
        Ok(config) => { config }
        Err(_) => { bail!(LoadSettingsError) }
    };

    set_ui_settings_fields(&ui, &config);
    define_ui_callbacks(&ui);

    let (max_x, max_y) = match get_screen_size() {
        Ok((x,y)) => { (x,y) }
        Err(_) => { (1000f32, 1000f32) }
    };

    let (x,y) = app_config().main_window_gui_size;
    ui.window().set_position(WindowPosition::Logical(LogicalPosition::new((max_x - x)/2f32, (max_y - y)/2f32)));

    debug!("Starting the UI");
    match ui.run() {
        Err(err) => { error!("An error occurred running UI: {:?}", err );}
        _ => { debug!("UI closed") }
    };
    Ok(())
}
