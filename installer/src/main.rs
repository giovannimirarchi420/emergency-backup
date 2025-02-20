#![windows_subsystem = "windows"]

mod gui_helper;

use slint::{LogicalPosition, LogicalSize, WindowPosition, WindowSize};
use config::app_config;
use window_spawn_util::get_screen_size;
slint::include_modules!();

fn main() -> anyhow::Result<()>{

    let ui = AppWindow::new()?;

    let (x,y) = app_config().installation_window_gui_size;
    let (max_x, max_y) = match get_screen_size() {
        Ok((x,y)) => { (x,y) }
        Err(_) => { (1000f32, 1000f32) }
    };

    match gui_helper::define_callbacks_and_props(&ui) {
        Ok(_) => {}
        Err(err) => { window_spawn_util::pop_out_error_window(String::from("Error preparing the installer"), Some(err.to_string())) }
    };

    let window = ui.window();

    window.set_size(WindowSize::Logical(LogicalSize::new(x , y)));
    window.set_position(WindowPosition::Logical(LogicalPosition::new((max_x - x)/2f32, (max_y - y)/2f32)));

    match ui.run() {
        Err(err) => { eprintln!("An error occurred running UI: {:?}", err );}
        _ => { println!("UI closed") }
    };



    Ok(())
}