pub mod gui_action {

    use std::path::{PathBuf};
    use anyhow::bail;
    use log::{debug, error};
    use native_dialog::FileDialog;
    use slint::{ComponentHandle, SharedString};
    use crate::{AppWindow, HomePageAdapter, SettingsPageAdapter};
    use errors::Error::{ApplySettingsError, LoadSettingsError, ModeSelectionError, NoPathChosenError};
    use settings::settings::{apply_settings, BackupConfig, load_settings};

    pub fn set_ui_settings_fields(app_window: &AppWindow, config: &BackupConfig) {
        app_window.global::<SettingsPageAdapter>().set_backup_source(SharedString::from(config.backup_source.clone()));
        app_window.global::<SettingsPageAdapter>().set_backup_destination(SharedString::from(config.backup_destination.clone()));
        app_window.global::<SettingsPageAdapter>().set_tolerance(config.tolerance.clone() as i32);
        app_window.global::<SettingsPageAdapter>().set_extension_only(config.extension_only.clone());
        app_window.global::<SettingsPageAdapter>().set_extension_type(SharedString::from(config.extension_type.clone().join("|")));
        app_window.global::<SettingsPageAdapter>().set_millis_update_frequency(config.millis_update_frequency.clone() as i32);
        app_window.global::<SettingsPageAdapter>().set_tracking_window_sec(config.tracking_window_sec.clone() as i32);
        app_window.global::<SettingsPageAdapter>().set_mode(SharedString::from(config.mode.clone()));
        app_window.global::<HomePageAdapter>().set_active(config.active.clone());
    }

    pub fn define_ui_callbacks(ui: &AppWindow) {

        let config_log = config::log_config(false);
        if config_log.is_ok() {
            log4rs::init_config(config_log.unwrap()).unwrap();
        }

        ui.global::<SettingsPageAdapter>().on_choose_backup_source({
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                match open_file_dialog_and_get_path( ui.global::<SettingsPageAdapter>().get_mode(), true ) {
                    Ok(string_path) => { ui.global::<SettingsPageAdapter>().set_backup_source(string_path); }
                    Err(err) => {
                        window_spawn_util::pop_out_error_window( String::from("An error occurred selecting file/folder for source selection"), Some(err.to_string()) )
                    }
                };

            }
        });

        ui.global::<SettingsPageAdapter>().on_choose_backup_destination({
            let ui_handle = ui.as_weak();
            move || {
                let ui = ui_handle.unwrap();
                match open_file_dialog_and_get_path( ui.global::<SettingsPageAdapter>().get_mode(), false ) {
                    Ok(string_path) => { ui.global::<SettingsPageAdapter>().set_backup_destination(string_path); }
                    Err(err) => {
                        window_spawn_util::pop_out_error_window( String::from("An error occurred selecting file/folder for destination selection"), Some(err.to_string()) )
                    }
                };
            }
        });

        ui.global::<SettingsPageAdapter>().on_apply_settings({
            let ui_handle = ui.as_weak();
            move || {
                match apply_settings_ui(&ui_handle.unwrap()) {
                    Ok(_) => {
                        window_spawn_util::pop_out_success_window(String::from("Settings successfully saved\nRestart the mouse tracker to make it effective."))
                    } //Changes successfully applied
                    Err(err) => {
                        window_spawn_util::pop_out_error_window( String::from("Settings could not be saved. Please, try again."), Some(err.to_string()) )
                    }
                }
            }
        });

        ui.global::<SettingsPageAdapter>().on_edited_millis_update_frequency( {
            let ui_handle = ui.as_weak();
            move |val| {
                ui_handle.unwrap().global::<SettingsPageAdapter>().set_millis_update_frequency(val)
            }
        });

        ui.global::<SettingsPageAdapter>().on_edited_tracking_window_sec( {
            let ui_handle = ui.as_weak();
            move |val| {
                ui_handle.unwrap().global::<SettingsPageAdapter>().set_tracking_window_sec(val);
            }
        });

        ui.global::<SettingsPageAdapter>().on_edited_tolerance( {
            let ui_handle = ui.as_weak();
            move |val| {
                ui_handle.unwrap().global::<SettingsPageAdapter>().set_tolerance(val)
            }
        });

        ui.global::<SettingsPageAdapter>().on_edited_extension_type( {
            let ui_handle = ui.as_weak();
            move |text| {
                ui_handle.unwrap().global::<SettingsPageAdapter>().set_extension_type(text)
            }
        });

        ui.global::<HomePageAdapter>().on_toggle_tracker({
            let ui_handle = ui.as_weak();

            move || {
                let active = ui_handle.unwrap().global::<HomePageAdapter>().get_active();
                if active {
                    match service_manager_util::stop_and_uninstall() {
                        Ok(_) => { ui_handle.unwrap().global::<HomePageAdapter>().set_active(false); }
                        Err(err) => { window_spawn_util::pop_out_error_window(String::from("An error occurred stopping the mouse tracker service"), Some(err.to_string())) }
                    }
                } else {
                    match service_manager_util::start(false) {
                        Ok(_) => { ui_handle.unwrap().global::<HomePageAdapter>().set_active(true); }
                        Err(err) => { window_spawn_util::pop_out_error_window(String::from("An error occurred starting the mouse tracker service"), Some(err.to_string())); }
                    };
                }
            }
        });

    }

    pub fn open_file_dialog_and_get_path(mode: SharedString, source_selection: bool) -> anyhow::Result<SharedString> {
        debug!("Open file dialog START");
        debug!("source_selection: {:?}", source_selection);
        let result_path : Option<PathBuf>;

        if source_selection {
            result_path = match mode.to_string().to_lowercase().as_str() {
                "file" => {
                    debug!("File mode");
                    FileDialog::new().show_open_single_file()?
                },
                "folder" => {
                    debug!("Folder mode");
                    FileDialog::new().show_open_single_dir()?
                },
                _ => bail!(ModeSelectionError)
            };
        } else {
            result_path = FileDialog::new().show_open_single_dir()?;
        }

        match result_path {
            Some(path) => {
                Ok(SharedString::from(path.into_os_string().into_string().unwrap()))
            }
            None => { bail!(NoPathChosenError)  }
        }
    }

    pub fn apply_settings_ui(app_window: &AppWindow) -> anyhow::Result<()> {
        debug!("apply_settings_ui START");

        let settings = match load_settings() {
            Ok(settings) => { settings }
            Err(err) => { error!("Error loading settings: {:?}", err); bail!(LoadSettingsError)}
        };

        let mut config = match BackupConfig::field_checks(get_config_from_ui(app_window)) {
            Ok(config) => { config }
            Err(err) => { bail!(err) }
        };

        config.installation_dir = settings.installation_dir;

        match apply_settings(&config) {
            Ok(_) => { debug!("Settings applied: {:?}", config); Ok(())}
            Err(_) => { bail!(ApplySettingsError) }
        }
    }

    fn get_config_from_ui(app_window: &AppWindow) -> BackupConfig {
        BackupConfig {
            backup_source: app_window.global::<SettingsPageAdapter>().get_backup_source().parse().unwrap(),
            backup_destination: app_window.global::<SettingsPageAdapter>().get_backup_destination().parse().unwrap(),
            millis_update_frequency: app_window.global::<SettingsPageAdapter>().get_millis_update_frequency() as usize,
            tracking_window_sec: app_window.global::<SettingsPageAdapter>().get_tracking_window_sec() as usize,
            tolerance: app_window.global::<SettingsPageAdapter>().get_tolerance() as u32,
            extension_only: app_window.global::<SettingsPageAdapter>().get_extension_only(),
            extension_type: app_window.global::<SettingsPageAdapter>().get_extension_type().to_string().split("|").map(str::to_string).collect(),
            mode: app_window.global::<SettingsPageAdapter>().get_mode().parse().unwrap(),
            active: app_window.global::<HomePageAdapter>().get_active(),
            installation_dir: None
        }
    }
}