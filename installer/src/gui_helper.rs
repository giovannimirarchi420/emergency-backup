
use native_dialog::FileDialog;
use slint::{ComponentHandle, SharedString};
use std::{fs, process};
use std::fs::{File};
use std::io::Write;
#[cfg(all(target_family="unix"))] use std::os::unix::fs::PermissionsExt;
#[cfg(all(target_family="unix"))] use nix::unistd::Uid;
use std::path::{PathBuf};
use anyhow::bail;
#[cfg(all(target_os="macos"))] use include_directory::{include_directory};
use settings::settings::{set_installation_dir};
#[cfg(all(target_family="unix"))] use settings::settings::{get_config_dir};
use window_spawn_util::pop_out_error_window;
use crate::{AppWindow, InstallerUiAdapter};

#[cfg(
    all(
        target_os = "windows",
        target_arch = "x86_64"
    )
)]
fn get_gui_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-pc-windows-gnu/release/embgui.exe")
}

#[cfg(
    all(
        target_os = "windows",
        target_arch = "x86_64"
    )
)]
fn get_cli_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-pc-windows-gnu/release/embctl.exe")
}

#[cfg(
    all(
        target_os = "windows",
        target_arch = "x86_64"
    )
)]
fn get_daemon_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-pc-windows-gnu/release/mouse_tracker.exe")
}

#[cfg(
    all(
        target_os = "linux",
        target_arch = "x86_64"
    )
)]
fn get_gui_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-unknown-linux-gnu/release/embgui")
}

#[cfg(
    all(
        target_os = "linux",
        target_arch = "x86_64"
    )
)]
fn get_cli_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-unknown-linux-gnu/release/embctl")
}

#[cfg(
    all(
        target_os = "linux",
        target_arch = "x86_64")
)
]
fn get_daemon_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-unknown-linux-gnu/release/mouse_tracker")
}

/// Just for macOS the embgui executable is embedded in an OS Application (so that could be used as normal application, put in Dock etc...)
#[cfg(
    all(
        target_os = "macos",
        target_arch = "aarch64")
)]
fn install_gui_bin(installation_dir: &PathBuf) -> anyhow::Result<()> {
    let bundle_install_dir= &installation_dir.join("embgui.app");
    match
    include_directory!("$CARGO_MANIFEST_DIR/../target/aarch64-apple-darwin/release/bundle/osx/embgui.app").extract(&bundle_install_dir) {
        Ok(_) => {
            let exe_path = bundle_install_dir.join("Contents").join("MacOS").join("embgui");
            println!("{:?}", exe_path);
            let mut permission = exe_path.metadata()?.permissions();
            permission.set_mode(0o777);
            match fs::set_permissions(&exe_path, permission) {
                Ok(_) => { println!("{:?} successfully installed", exe_path.to_str()); }
                Err(err) => { eprintln!("{:?} installation FAILED.\n{:?}", exe_path.to_str(), err) }
            };
            Ok(())
        }
        Err(err) => { println!("Error installing gui: {:?}", err); bail!(err) }
    }
}

#[cfg(
    all(
        target_os = "macos",
        target_arch = "aarch64")
)
]
fn get_cli_binary() -> &'static [u8] {
    include_bytes!("../../target/aarch64-apple-darwin/release/embctl")
}

#[cfg(
    all(
        target_os = "macos",
        target_arch = "aarch64")
)
]
fn get_daemon_binary() -> &'static [u8] {
    include_bytes!("../../target/aarch64-apple-darwin/release/mouse_tracker")
}

/// Just for macOS the embgui executable is embedded in an OS Application (so that could be used as normal application, put in Dock etc...)
#[cfg(
    all(
        target_os = "macos",
        target_arch = "x86_64")
)]
fn install_gui_bin(installation_dir: &PathBuf) -> anyhow::Result<()> {
    let bundle_install_dir= &installation_dir.join("embgui.app");
    match
    include_directory!("$CARGO_MANIFEST_DIR/../target/x86_64-apple-darwin/release/bundle/osx/embgui.app").extract(&bundle_install_dir) {
        Ok(_) => {
            let exe_path = bundle_install_dir.join("Contents").join("MacOS").join("embgui");
            println!("{:?}", exe_path);
            let mut permission = exe_path.metadata()?.permissions();
            permission.set_mode(0o777);
            match fs::set_permissions(&exe_path, permission) {
                Ok(_) => { println!("{:?} successfully installed", exe_path.to_str()); }
                Err(err) => { eprintln!("{:?} installation FAILED.\n{:?}", exe_path.to_str(), err) }
            };
            Ok(())
        }
        Err(err) => { println!("Error installing gui: {:?}", err); bail!(err) }
    }
}

#[cfg(
    all(
        target_os = "macos",
        target_arch = "x86_64")
)
]
fn get_cli_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-apple-darwin/release/embctl")
}

#[cfg(
    all(
        target_os = "macos",
        target_arch = "x86_64")
)
]
fn get_daemon_binary() -> &'static [u8] {
    include_bytes!("../../target/x86_64-apple-darwin/release/mouse_tracker")
}

fn create_executable( executable_name: &str, executable: &[u8], installation_dir: &PathBuf ) -> anyhow::Result<()> {
    let exe_path = installation_dir.join(executable_name);
    let mut file = File::create(&exe_path)?;
    file.write_all(executable)?;

    #[cfg(target_family="unix")]
    {
        let mut permission = file.metadata()?.permissions();
        permission.set_mode(0o777);
        match fs::set_permissions(&exe_path, permission) {
            Ok(_) => { println!("{} successfully installed", executable_name); }
            Err(err) => { eprintln!("{} installation FAILED.\n{:?}", executable_name, err) }
        };
    }

    Ok(())
}

#[cfg(target_family = "unix")]
fn set_permission(path: &PathBuf) -> anyhow::Result<()> {
    let mut permission = fs::metadata(&path)?.permissions();
    permission.set_mode(0o777);
    match fs::set_permissions(&path, permission) {
        Ok(_) => { println!("{:?} successfully installed", &path); Ok(())}
        Err(err) => { eprintln!("{:?} installation FAILED.\n{:?}", &path, err); bail!(err) }
    }

}
fn install(installation_dir: PathBuf, is_cli_to_install: bool, is_gui_to_install: bool) -> anyhow::Result<()> {

    match set_installation_dir(String::from(installation_dir.to_str().unwrap())) {
        Ok(_) => { println!("Installation dir configuration saved")}
        Err(err) => { bail!(err) }
    }

    //Create installation directory
    fs::create_dir_all(&installation_dir)
        .unwrap_or_else(|err| pop_out_error_window(String::from("Error creating installation directory folder. Did you run the Emergency Backup Installer with administrator privileges?"), Some(err.to_string())));

    // This code is ran as root.
    // This means that we need to explicitly set permissions to file and folder in order to be later used from normal users.
    #[cfg(target_family = "unix")]
    {
        let config_dir = match get_config_dir() {
            Ok(dir) => { dir },
            Err(err) => { eprintln!("Error retrieving config dir {:?}", err); bail!(err) }
        };

        match set_permission(&installation_dir) {
            Ok(_) => { println!("{:?} permission setted", &installation_dir)},
            Err(err) => { eprintln!("Error setting permissions to: {:?}\nError: {:?}", &installation_dir, err) }
        };
        match set_permission(&config_dir) {
            Ok(_) => { println!("{:?} permission setted", &config_dir)},
            Err(err) => { eprintln!("Error setting permissions to: {:?}\nError: {:?}", &config_dir, err) }
        };
        match set_permission(&config_dir.join("config.yaml")) {
            Ok(_) => { println!("{:?} permission setted", &config_dir.join("config.yaml"))},
            Err(err) => { eprintln!("Error setting permissions to: {:?}\nError: {:?}", &config_dir.join("config.yaml"), err) }
        };
    }
    /********************************************
    *                                           *
    *          Install embcli daemon            *
    *                                           *
    *********************************************/
    if is_cli_to_install {
        let cli = get_cli_binary();
        if cfg!(target_family = "windows") {
            create_executable("embctl.exe", cli, &installation_dir)
                .unwrap_or_else(|err| pop_out_error_window(String::from("Error creating Emergency Backup CLI executable. Did you run the Emergency Backup Installer with administrator privileges?"), Some(err.to_string())));
        } else {
            create_executable("embctl", cli, &PathBuf::from("/usr/local/bin"))
                .unwrap_or_else(|err| pop_out_error_window(String::from("Error creating Emergency Backup CLI executable. Did you run the Emergency Backup Installer with administrator privileges?"), Some(err.to_string())));
        }
    }

    /********************************************
    *                                           *
    *             Install embgui                *
    *                                           *
    *********************************************/
    if is_gui_to_install {

        #[cfg(target_os = "macos")]
        {
            match install_gui_bin(&installation_dir) {
                Ok(_) => {println!("embgui installed succesfully")}
                Err(err) => { pop_out_error_window(String::from("Error creating Emergency Backup GUI executable. Did you run the Emergency Backup Installer with administrator privileges?"), Some(err.to_string())) }
            }
        }

        #[cfg(not(target_os = "macos"))]
       {
            let gui = get_gui_binary();
            let exe_name = if cfg!(target_family = "windows") { "embgui.exe" } else { "embgui "};
            let _ = create_executable(exe_name, gui, &installation_dir)
                .unwrap_or_else(|err| pop_out_error_window(String::from("Error creating Emergency Backup GUI executable. Did you run the Emergency Backup Installer with administrator privileges?"), Some(err.to_string())));
        }
    }

    /********************************************
    *                                           *
    *      Install mouse_tracker daemon         *
    *                                           *
    *********************************************/
    let daemon = get_daemon_binary();
    let daemon_exe_name = if cfg!(target_family = "windows") { "mouse_tracker.exe" } else { "mouse_tracker" };
    create_executable(daemon_exe_name, daemon, &installation_dir)
        .unwrap_or_else(|err| pop_out_error_window(String::from("Error creating mouse_tracker executable. Did you run the Emergency Backup Installer with administrator privileges?"), Some(err.to_string())));


    Ok(())
}
pub fn open_file_dialog_and_get_path() -> anyhow::Result<SharedString> {
    let result_path = FileDialog::new().show_open_single_dir()?;
    match result_path {
        Some(path) => {
            Ok(SharedString::from(path.join("EmergencyBackup").into_os_string().into_string().unwrap()))
        }
        None => { Ok(SharedString::new())  }
    }
}

pub fn define_callbacks_and_props(ui: &AppWindow) -> anyhow::Result<()>{
    let homedir_path = homedir::my_home()?;

    if homedir_path.is_some() {
        ui.global::<InstallerUiAdapter>().set_installation_dir(SharedString::from(homedir_path.unwrap().join("EmergencyBackup").to_str().unwrap()));
    }

    ui.global::<InstallerUiAdapter>().on_close({
        || {
            process::exit(0);
        }
    });

    ui.global::<InstallerUiAdapter>().on_check_root_permissions({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();

            #[cfg(target_family = "unix")]
            if !Uid::effective().is_root() {
                pop_out_error_window(String::from("You must run this executable with root permissions"), None);
                ui.global::<InstallerUiAdapter>().set_root_granted(false)
            } else {
                ui.global::<InstallerUiAdapter>().set_root_granted(true)
            }

            #[cfg(target_family = "windows")]
            {
                ui.global::<InstallerUiAdapter>().set_root_granted(true)
            }
        }
    });

    ui.global::<InstallerUiAdapter>().on_choose_installation_destination({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            match open_file_dialog_and_get_path() {
                Ok(string_path) => { ui.global::<InstallerUiAdapter>().set_installation_dir(string_path); }
                Err(_) => {
                    eprintln!("No path chosen");
                }
            };
        }
    });

    ui.global::<InstallerUiAdapter>().on_install({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let installation_dir = ui.global::<InstallerUiAdapter>().get_installation_dir();
            let is_cli_to_install = ui.global::<InstallerUiAdapter>().get_cli_selected();
            let is_gui_to_install = ui.global::<InstallerUiAdapter>().get_gui_selected();
            match install(PathBuf::from(installation_dir.to_string()), is_cli_to_install, is_gui_to_install) {
                Ok(_) => { ui.global::<InstallerUiAdapter>().set_installation_success(true) }
                Err(err) => {
                    ui.global::<InstallerUiAdapter>().set_installation_error(SharedString::from(err.to_string()))
                }
            };
        }
    });

    Ok(())
}

