use std::path::PathBuf;
use std::{process};
use anyhow::bail;
use display_info::DisplayInfo;
use slint::{ComponentHandle, LogicalPosition, SharedString, WindowPosition};
use slint::private_unstable_api::re_exports::load_image_from_embedded_data;
use slint_interpreter::{Compiler, ComponentInstance, Image, Value};
use errors::Error::{BuildDenyWindowsError, BuildErrorWindowsError, BuildSuccessWindowsError, ScreenSizeError};
use config::{app_config};

#[cfg(target_family="windows")]
fn get_error_icon() -> Image {
    load_image_from_embedded_data(include_bytes!("../img/error_icon.png").as_slice().into(), Default::default())

}

#[cfg(not(target_family="windows"))]
fn get_error_icon() -> Image {
    load_image_from_embedded_data(include_bytes!("../img/error_icon.png").as_slice().into(), Default::default())
}

#[cfg(target_family="windows")]
fn get_success_icon() -> Image {
    load_image_from_embedded_data(include_bytes!("../img/success.png").as_slice().into(), Default::default())

}

#[cfg(not(target_family="windows"))]
fn get_success_icon() -> Image {
    load_image_from_embedded_data(include_bytes!("../img/success.png").as_slice().into(), Default::default())
}

#[cfg(target_family="windows")]
fn get_deny_icon() -> Image {
    load_image_from_embedded_data(include_bytes!("../img/warning.png").as_slice().into(), Default::default())

}

#[cfg(not(target_family="windows"))]
fn get_deny_icon() -> Image {
    load_image_from_embedded_data(include_bytes!("../img/warning.png").as_slice().into(), Default::default())
}

#[cfg(target_family="windows")]
fn get_error_window_code() -> String {
    include_str!("../windows/error_window.slint").to_string()

}

#[cfg(not(target_family="windows"))]
fn get_error_window_code() -> String {
    include_str!("../windows/error_window.slint").to_string()
}

#[cfg(target_family="windows")]
fn get_success_window_code() -> String {
    include_str!("../windows/success_window.slint").to_string()

}

#[cfg(not(target_family="windows"))]
fn get_success_window_code() -> String {
    include_str!("../windows/success_window.slint").to_string()
}

#[cfg(target_family="windows")]
fn get_deny_window_code() -> String {
    include_str!("../windows/deny_window.slint").to_string()

}

#[cfg(not(target_family="windows"))]
fn get_deny_window_code() -> String {
    include_str!("../windows/deny_window.slint").to_string()
}

#[tokio::main]
async fn build_error_window() -> anyhow::Result<ComponentInstance> {
    let compiler = Compiler::default();

    let error_window = get_error_window_code();
    let compilation_result = compiler.build_from_source(error_window, PathBuf::new()).await;
    let window = compilation_result.component("ErrorWindow");
    match window {
        Some(component_definition) => {
            match component_definition.create() {
                Ok(win_instance) => { Ok(win_instance) }
                Err(_) => { bail!(BuildErrorWindowsError) }
            }
        }
        None => { bail!(BuildErrorWindowsError) }
    }
}

pub fn pop_out_error_window(error: String, details: Option<String>) {
    let (max_x, max_y) = match get_screen_size() {
        Ok((x,y)) => { (x,y) }
        Err(_) => { (1920f32, 1080f32) }
    };

    match build_error_window() {
        Ok(win_instance) => {
            let _ = win_instance.set_property("error", Value::from(SharedString::from(error)));
            if details.is_some() {
                let _ = win_instance.set_property("details", Value::from(SharedString::from(details.unwrap())));
            }
            let error_icon = get_error_icon();
            let _ = win_instance.set_property("error_icon", Value::Image(error_icon));
            let window = win_instance.window();
            let (x, y) = app_config().pop_up_window_size;
            window.set_position(WindowPosition::Logical(LogicalPosition::new((max_x - x)/2f32, (max_y - y) / 2f32)));
            let _ = win_instance.run();
        },
        Err(_) => { }
    }
}

#[tokio::main]
async fn build_success_window() -> anyhow::Result<ComponentInstance> {

    let compiler = Compiler::default();
    let success_window = get_success_window_code();
    let compilation_result = compiler.build_from_source(success_window, PathBuf::new()).await;
    let window = compilation_result.component("SuccessWindow");
    match window {
        Some(component_definition) => {
            match component_definition.create() {
                Ok(win_instance) => { Ok(win_instance) }
                Err(_) => { bail!(BuildSuccessWindowsError) }
            }
        }
        None => { bail!(BuildSuccessWindowsError) }
    }
}
pub fn pop_out_success_window(message: String) {
    let (max_x, max_y) = match get_screen_size() {
        Ok((x,y)) => { (x,y) }
        Err(_) => { (1920f32, 1080f32) }
    };

    match build_success_window() {
        Ok(win_instance) => {
            let _ = win_instance.set_property("message", Value::from(SharedString::from(message)));
            let success_icon = get_success_icon();
            let _ = win_instance.set_property("success_icon", Value::Image(success_icon));
            let window = win_instance.window();
            let (x, y) = app_config().pop_up_window_size;
            window.set_position(WindowPosition::Logical(LogicalPosition::new((max_x - x)/2f32, (max_y - y) / 2f32)));
            let _ = win_instance.run();
        },
        Err(_) => { }
    }
}


#[tokio::main]
async fn build_deny_window() -> anyhow::Result<ComponentInstance> {
    let compiler = Compiler::default();
    let deny_window = get_deny_window_code();
    let compilation_result = compiler.build_from_source(deny_window, PathBuf::new()).await;
    let window = compilation_result.component("DenyWindow");
    match window {
        Some(component_definition) => {
            match component_definition.create() {
                Ok(win_instance) => { Ok(win_instance) }
                Err(_) => { bail!(BuildDenyWindowsError) }
            }
        }
        None => { bail!(BuildDenyWindowsError) }
    }
}

pub fn pop_out_deny_window(message: String) {
    let (max_x, max_y) = match get_screen_size() {
        Ok((x,y)) => { (x,y) }
        Err(_) => { (1920f32, 1080f32) }
    };

    match build_deny_window() {
        Ok(win_instance) => {

            let _ = win_instance.set_property("message", Value::from(SharedString::from(message)));
            let deny_icon = get_deny_icon();
            let _ = win_instance.set_property("warning_icon", Value::Image(deny_icon));
            let _ = win_instance.set_callback("cancel_backup",  move | _ : &[Value]| -> Value {
                //This will stop the mouse_tracker service, the service manager will take care to restart it automatically
                process::exit(0);
            });
            let window = win_instance.window();
            let (x, y) = app_config().pop_up_window_size;
            window.set_position(WindowPosition::Logical(LogicalPosition::new((max_x - x)/2f32, (max_y - y) / 2f32)));
            window.on_close_requested(|| {
                process::exit(0);
            });

            let _ = win_instance.run();
        },
        Err(_) => {  }
    }
}

pub fn get_screen_size() -> anyhow::Result<(f32, f32)> {
    let display_infos = DisplayInfo::all();

    match display_infos {
        Ok(infos) => {
            match infos.iter().find(|display_info| { display_info.is_primary }) {
                Some(display_info) => {
                    Ok((display_info.width as f32, display_info.height as f32))
                },
                None => { bail!(ScreenSizeError) }
            }
        },
        Err(_err) => { bail!(ScreenSizeError) }
    }
}