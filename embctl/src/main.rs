use std::fmt::Display;
use std::path::PathBuf;
use anyhow::bail;
use clap::{Args, Parser, Subcommand};
use log::{debug, error, info};
use errors::Error::{DaemonStartupError, DaemonStopError, ExtensionOnlyValueError, LoadSettingsError};
use serde::Serialize;
use settings::settings::{load_settings, set_destination, set_extension_only, set_extension_types, set_millis_update_time, set_mode, set_source, set_tolerance, set_tracking_window_sec};

/// This tool allow to perform emergency backups using a mouse command
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Embctl {

    /// Turn debugging information on
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start emergency backup daemon process
    Start,

    /// Stop emergency backup daemon process
    Stop,

    /// Check mouse tracker daemon service status
    Status,

    /// Print current configuration
    ShowConfig,

    /// Mouse position sampling frequency. Higher frequency results in higher CPU usage. (default: 200ms)
    SetMillisUpdateTime(MillisUpdateTimeArg),

    /// Time window (in seconds) within which the user must perform the mouse command to trigger the backup action. (default: 15sec)
    SetTrackingWindowSec(TrackingWindowSecArgs),

    /// Tolerance for touching in the corners of the display. (default: 5px)
    SetTolerance(ToleranceArgs),

    /// Backup source path
    SetSource(SourceArg),

    /// Backup destination path, a new emergency-backup/ folder will be created containing the content of the backup_source.
    SetDestination(DestinationArg),

    /// If true, only files with an extension contained in extension_type vector will be copied. (default: false)
    SetExtensionOnly(ExtensionOnlyArg),

    /// List of backup target file extensions, separated by comma or space
    SetExtensionType(ExtensionTypeArg),

    /// 'file' or 'folder' mode (default: Folder)
    SetMode(ModeArg),
}


#[derive(Debug, Args)]
struct MillisUpdateTimeArg {
    /// An integer that represent the time in ms
    #[arg(default_value_t = 200)]
    time: usize,
}

#[derive(Debug, Args)]
struct ToleranceArgs {
    /// An integer that represent the tolerance for touching in the corners of the display
    #[arg(default_value_t = 5)]
    tolerance: u32,
}

#[derive(Debug, Args)]
struct SourceArg {
    /// Backup source path
    source: PathBuf,
}

#[derive(Debug, Args)]
struct DestinationArg {
    /// Backup destination path, a new emergency-backup/ folder will be created containing the content of the backup_source.
    destination: PathBuf,
}

#[derive(Debug, Args)]
struct ExtensionOnlyArg {
    /// If true, only files with an extension contained in extension_type vector will be copied.
    extension_only: String,
}
#[derive(Debug, Clone, Args)]
struct ModeArg {
    /// 'Folder' or 'File', if the backup has as objective a file or an entire directory
    mode: ModeEnum
}

#[derive(clap::ValueEnum, Clone, Debug, Default, Serialize)]
enum ModeEnum {
    #[default]
    Folder,
    File
}

impl Display for ModeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ModeEnum::Folder => { String::from("folder") }
            ModeEnum::File => { String::from("file") }
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Args)]
struct ExtensionTypeArg {
    /// A space separated list of file extensions to filter the backup source files
    extension_type: Vec<String>,
}

#[derive(Debug, Args)]
struct TrackingWindowSecArgs {
    /// An integer that represent the time in seconds
    time: usize,
}


fn main() -> anyhow::Result<()> {

    let args = Embctl::parse();

    let log_config = match config::log_config(args.debug) {
        Ok(config) => { config },
        Err(err) => { eprintln!("Logging configuration creation FAILED: {:?}", err); bail!(err) }
    };
    let config = log4rs::init_config(log_config);
    if config.is_err() {
        eprintln!("Logging configuration creation FAILED");
    } else {
        config.unwrap();
    }

    let settings = match load_settings() {
        Ok(config) => { config }
        Err(_) => { bail!(LoadSettingsError) }
    };

    if args.command.is_some() {
        match args.command.unwrap() {

            Commands::Start => {
                match service_manager_util::start(args.debug) {
                    Ok(_) => {}
                    Err(err) => { error!(target: "general", "An error occurred starting the mouse tracker: {:?}", err); bail!(DaemonStartupError) }
                };
            }

            Commands::Stop => {
                match service_manager_util::stop_and_uninstall() {
                    Ok(_) => {}
                    Err(_) => { bail!(DaemonStopError) }
                };
            }

            Commands::ShowConfig => {
                info!(target: "general", "{:?}", settings)
            }

            Commands::Status => {
                debug!(target: "general", "Status check...");
                match load_settings() {
                    Ok(config) => {
                        if config.active {
                            info!(target: "general", "Mouse tracker RUNNING")
                        } else {
                            info!(target: "general", "Mouse tracker NOT RUNNING")
                        }
                    }
                    Err(err) => { bail!(err) }
                }
            }

            Commands::SetMillisUpdateTime(arg) => {
                match set_millis_update_time(arg.time) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetMode(arg) => {
                match set_mode(arg.mode.to_string()) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetExtensionOnly(arg) => {
                let bool_val = match arg.extension_only.as_str() {
                    "true" => { true }
                    "false" => { false },
                    _ => { bail!(ExtensionOnlyValueError) }
                };
                match set_extension_only(bool_val) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetExtensionType(arg) => {
                match set_extension_types(arg.extension_type) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetDestination(arg) => {
                match set_destination(arg.destination) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetSource(arg) => {
                match set_source(arg.source) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetTolerance(arg) => {
                match set_tolerance(arg.tolerance) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

            Commands::SetTrackingWindowSec(arg) => {
                match set_tracking_window_sec(arg.time) {
                    Ok(_) => { info!("Configuration has been successfully updated, restart the mouse_tracker to make it effective") }
                    Err(err) => { error!("Error: {:?}", err) }
                }
            }

        }
    };

    Ok(())
}