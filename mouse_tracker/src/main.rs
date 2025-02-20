#![windows_subsystem = "windows"]

use std::env;
use anyhow::bail;
use log::{error, info};

use errors::Error::{LoadSettingsError};
use settings::settings::{ensure_config_dir, load_settings};
use crate::mouse_tracker::MouseTracker;

mod mouse_tracker;
mod data_type;

fn main() -> anyhow::Result<()>{
    let args: Vec<String> = env::args().collect();

    //Check if the configuration directory exists, if not it will be created
    match ensure_config_dir() {
        Ok(_) => {}
        Err(_) => { eprintln!("Error ensuring config dir")}
    }

    //Loading log configuration from 'config' lib crate
    let config_log = config::log_config(if args.len() == 2 { true } else { false });
    log4rs::init_config(config_log?).unwrap();
    info!(target: "general", "Mouse tracker service START");

    let settings = match load_settings() {
        Ok(config) => { info!("Loaded settings: {:?}", config); config }
        Err(_) => { bail!(LoadSettingsError) }
    };

    match MouseTracker::from(settings) {
        Ok(tracker) => { tracker.start(); Ok(()) }
        Err(error) => { error!("{}", error.to_string()); Err(error) }
    }

}
