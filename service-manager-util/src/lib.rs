use std::ffi::OsString;
use std::path::{PathBuf};
use std::str::FromStr;
use anyhow::bail;
use log::{error, info};
use notify_rust::{Notification};
use errors::Error::{BackupPathNotConfigured, DaemonStopError, InstallationDirectoryNotFound};
use settings::settings::{BackupConfig, load_settings, set_tracker_off, set_tracker_on};
#[cfg(target_family = "unix")] use log::{debug};
#[cfg(target_family = "unix")] use service_manager::{ServiceInstallCtx, ServiceLevel, ServiceManager, ServiceStartCtx, ServiceStopCtx, ServiceUninstallCtx};
#[cfg(target_family = "unix")] use errors::Error::{DaemonInstallationError, DaemonStartError, DaemonStartupError, DaemonUninstallationError, UserLeverNotSupportedError};
#[cfg(target_family = "unix")] use settings::settings::{get_config_dir, ensure_config_dir};
#[cfg(target_family = "unix")]
pub fn setup() -> anyhow::Result<Box<dyn ServiceManager>> {
    match ensure_config_dir() {
        Ok(_) => {}
        Err(_) => { error!(target: "general", "Error ensuring config dir")}
    }

    // Get generic service manager by detecting what is available on the platform
    let mut manager = match <dyn ServiceManager>::native() {
        Ok(service_manager) => { service_manager }
        Err(_) => {
            error!(target: "general", "An error occurred retrieving daemon service manager");
            bail!(DaemonStartupError);
        }
    };


    match manager.set_level(ServiceLevel::User) {
        Ok(_) => { debug!(target: "general", "Service Level set: USER"); }
        Err(_) => {
            error!(target: "general", "Service manager does not support user-level services");
            bail!(UserLeverNotSupportedError);
        }
    };

    Ok(manager)
}

pub fn check_and_get_settings() -> anyhow::Result<BackupConfig> {
    let settings = match load_settings() {
        Ok(settings) => {
            let source_path = PathBuf::from_str(&*settings.backup_source);
            let destination_path = PathBuf::from_str(&*settings.backup_destination);
            if source_path.is_err() || destination_path.is_err() ||
                source_path.unwrap().as_os_str().is_empty() || destination_path.unwrap().as_os_str().is_empty() {
                bail!(BackupPathNotConfigured)
            }
            settings
        }
        Err(err) => { bail!(err) }
    };

    let _ = match &settings.installation_dir {
        Some(_) => { },
        None => { error!(target: "general", "Installation directory not found in .emergency_backup"); bail!(InstallationDirectoryNotFound) }
    };

    Ok(settings)
}

#[cfg(target_family = "windows")]
pub fn start(debug: bool) -> anyhow::Result<()> {
    use std::path::Path;
    use winreg::enums::*;
    use winreg::RegKey;

    let installation_dir = match check_and_get_settings() {
        Ok(settings) => { settings },
        Err(err) => { bail!(err) }
    }.installation_dir.unwrap(); //The unwrap() here is safe because has been already checked in check_and_get_settings()

    let exe_path = PathBuf::from(installation_dir).join("mouse_tracker.exe");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("Software").join("Microsoft").join("Windows").join("CurrentVersion").join("Run");
    let (key, _disp) = hkcu.create_subkey(&path).unwrap();

    key.set_value(&"Emergency Backup", &exe_path.clone().into_os_string()).unwrap();

    let mut command = std::process::Command::new(&exe_path);
    if debug { command.arg(OsString::from("debug")) } else { &mut command };
    command.spawn()?;

    info!(target: "general", "Mouse Tracker STARTED");
    set_tracker_on()
}

#[cfg(target_family = "unix")]
pub fn start(debug: bool) -> anyhow::Result<()> {
    let app_config = config::app_config();
    let config_dir = get_config_dir()?;

    let installation_dir = match check_and_get_settings() {
        Ok(settings) => { settings },
        Err(err) => { bail!(err) }
    }.installation_dir.unwrap(); //The unwrap() here is safe because has been already checked in check_and_get_settings()

    let mouse_tracker_path = PathBuf::from(installation_dir).join("mouse_tracker");

    match setup() {
        Ok(manager) => {
            match manager.install(ServiceInstallCtx {
                label: app_config.serviced_label.clone().parse().unwrap(),
                program: mouse_tracker_path,
                args: if debug { vec![OsString::from("debug")] } else { vec![] },
                contents: None, // Optional String for system-specific service content.
                username: None, // Optional String for alternative user to run service.
                working_directory: Option::from(config_dir), // Optional String for the working directory for the service process.
                environment: None, // Optional list of environment variables to supply the service process.
                autostart: true, // Specify whether the service should automatically start upon OS reboot.
            }) {
                Ok(_) => { info!(target: "general", "Mouse tracker service successfully INSTALLED") }
                Err(err) => {
                    error!(target: "general", "An error occurred installing the daemon service to the service manager: {:?}", err);
                    bail!(DaemonInstallationError)
                }
            }

            match manager.start(ServiceStartCtx {
                label: app_config.serviced_label.clone().parse().unwrap(),
            }) {
                Ok(_) => { info!(target: "general", "Mouse tracker service successfully STARTED") }
                Err(err) => {
                    error!(target: "general", "An error occurred starting the daemon service: {:?}", err);
                    bail!(DaemonStartError)
                }
            }

            set_tracker_on()
        },
        Err(err) => { bail!(err) }
    }
}

#[cfg(target_family = "unix")]
pub fn stop_and_uninstall() -> anyhow::Result<()> {
    let app_config = config::app_config();
    match setup() {
        Ok(manager) => {
            match manager.stop(ServiceStopCtx {
                label: app_config.serviced_label.clone().parse().unwrap()
            }) {
                Ok(_) => { info!(target: "general", "Mouse tracker service successfully STOPPED") }
                Err(_) => {
                    error!(target: "general", "An error occurred stopping the daemon service");
                    bail!(DaemonStopError)
                }
            };

            match manager.uninstall(ServiceUninstallCtx {
                label: app_config.serviced_label.clone().parse().unwrap()
            }) {
                Ok(_) => { info!(target: "general", "Mouse tracker service successfully UNINSTALLED") }
                Err(err) => {
                    error!(target: "general", "An error occurred uninstalling the daemon service from the service manager: {}", err);
                    bail!(DaemonUninstallationError)
                }
            };

            set_tracker_off()
        },
        Err(err) => { bail!(err) }
    }
}

#[cfg(target_family = "windows")]
pub fn stop_and_uninstall() -> anyhow::Result<()> {
    use std::path::Path;
    use winreg::enums::*;
    use winreg::RegKey;

    let mut system = sysinfo::System::new();
    system.refresh_all();
    for p in system.processes_by_name("mouse_tracker.exe") {
        println!("Killing process: [{}] - [{}]", p.pid(), p.name());
        p.kill();
        match set_tracker_off() {
            Ok(_) => { info!(target: "general", "Mouse Tracker STOPPED"); }

            //Here is intentionally not returned, in this way the system will try at least to remove the Registry Key
            Err(err) => { error!(target: "general", "Error stopping mouse_tracker process: {:?}", err); }
        }
    }


    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("Software").join("Microsoft").join("Windows").join("CurrentVersion").join("Run");
    match hkcu.open_subkey_with_flags(&path, KEY_ALL_ACCESS)?.delete_value("Emergency Backup") {
        Ok(_) => {
            info!(target: "general", "Mouse Tracker UNINSTALLED");  Ok(())
        },
        Err(err) => { error!(target: "general", "Error UNINSTALLING the mouse_tracker: {:?}", err); bail!(DaemonStopError) }
    }


}

//From notify-rust docs, summary and body are supported on Unix and Windows env
pub fn notify(summary: String, body: String) {
    Notification::new()
        .summary(&*summary)
        .body(&*body)
        .show().unwrap();
}