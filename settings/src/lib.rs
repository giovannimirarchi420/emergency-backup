
pub mod settings {
    use std::{fs};
    use std::fs::File;
    use std::io::{BufWriter, Read};
    use std::path::{Path, PathBuf};
    use anyhow::{bail};
    use homedir::{get_my_home};
    use log::{debug, error};
    use serde::{Deserialize, Serialize};
    use errors::Error::{ApplySettingsError, ExtensionTypeArrayEmptyError, ExtensionTypeFormatError, FileProvidedFolderRequiredError, FolderProvidedFileRequiredError, HomeDirectoryError, LoadSettingsError, MillisUpdateFrequencyError, ParentPathError, ZeroTrackingWindowSecError};
    use regex::Regex;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct BackupConfig {
        /// Mouse position sampling frequency. Higher frequency results in higher CPU usage.
        /// A higher value increases tracking precision but may also negatively impact performance.
        pub millis_update_frequency: usize,

        /// Time window (in seconds) within which the user must perform the mouse command to trigger the backup action.
        /// The value should be long enough to allow the user to complete the action but not so long as to cause unwanted delays.
        pub tracking_window_sec: usize,

        /// Tolerance for touching in the corners of the display.
        /// The maximum display size may not be perfectly "clickable", so this tolerance allows the user to activate the backup action even if the position is not perfectly precise.
        /// Even with tolerance enabled, it is unlikely that unintentional commands will occur.
        pub tolerance: u32,

        /// Backup source path
        pub backup_source: String,

        /// Backup destination path, a new emergency-backup/ folder will be created containing the content of the backup_source.
        pub backup_destination: String,

        /// If true, only files with an extension contained in extension_type vector will be copied.
        pub extension_only: bool,
        pub extension_type: Vec<String>,

        /// 'Folder' or 'File', if the backup has as objective a file or an entire directory
        pub mode: String,

        /// If the mouse_tracker daemon service is active
        pub active: bool,

        pub installation_dir: Option<String>
    }

    impl Default for BackupConfig {
        fn default() -> Self {
            BackupConfig {
                backup_source: "".to_string(),
                backup_destination: "".to_string(),
                millis_update_frequency: 200,
                tracking_window_sec: 15,
                tolerance: 5,
                extension_only: false,
                extension_type: vec![],
                active: false,
                mode: "Folder".to_string(),
                installation_dir: None
            }
        }
    }
    pub fn ensure_config_dir() -> anyhow::Result<()>{
        match get_config_dir() {
            Ok(config_path) => { Ok(fs::create_dir_all(config_path)?) }
            Err(_) => { bail!(ParentPathError) }
        }
    }

    impl BackupConfig {
        pub fn field_checks(backup_config: BackupConfig) -> anyhow::Result<Self> {
            if backup_config.millis_update_frequency <= 0 {
                bail!(MillisUpdateFrequencyError)
            }

            if backup_config.extension_only {
                let regex = Regex::new(r"^(\w+|\|)+$").unwrap();
                if !regex.is_match(backup_config.extension_type.join("|").as_str()) {
                    bail!(ExtensionTypeFormatError)
                }
                if backup_config.extension_type.is_empty() {
                    bail!(ExtensionTypeArrayEmptyError)
                }
            }

            if backup_config.tracking_window_sec <= 0 {
                bail!(ZeroTrackingWindowSecError)
            }

            match backup_config.mode.to_lowercase().as_str() {
                "file" => {
                    if PathBuf::from(backup_config.clone().backup_source).is_dir() {
                        bail!(FolderProvidedFileRequiredError)
                    }
                },
                "folder" => {
                    if !PathBuf::from(backup_config.clone().backup_source).is_dir() {
                        bail!(FileProvidedFolderRequiredError)
                    }
                },
                _ => {}
            }

            Ok(backup_config)
        }
    }

    pub fn load_settings() -> anyhow::Result<BackupConfig> {
        debug!(target: "general", "Loading settings START");

        let config_path = get_config_path()?;
        match read_or_create_config(&config_path) {
            Ok(config) => { debug!(target: "general", "Loading settings OK"); Ok( config ) }
            Err(_) => { error!(target: "general", "Loading settings KO"); bail!(LoadSettingsError) }
        }
    }

    ///This function return the absolute path to the config yaml
    pub(crate) fn get_config_path() -> anyhow::Result<PathBuf> {
        debug!(target: "general", "get_config_path START");

        if cfg!(target_family = "windows"){
            Ok(PathBuf::from("C:\\ProgramData\\.emergency-backup\\config.yaml"))
        } else {
            let homedir_path = get_my_home()?;

            if homedir_path.is_none() {
                bail!(HomeDirectoryError)
            }
            debug!(target: "general", "homedir_path_option: {:?}", &homedir_path);

            let homedir_string = homedir_path.unwrap().into_os_string().into_string();

            if homedir_string.is_err() {
                bail!(HomeDirectoryError)
            }
            debug!(target: "general", "homedir_string: {:?}", &homedir_string);
            let config_path = Path::new(&homedir_string.unwrap()).join(".emergency-backup/config.yaml");
            debug!(target: "general", "config_path: {:?}", &config_path);
            Ok(config_path)
        }


    }

    /// This function return the absolute path of where the config yaml is stored (its parent folder)
    pub fn get_config_dir() -> anyhow::Result<PathBuf> {
        debug!(target: "general", "get_config_dir START");

        if cfg!(target_family = "windows") {
            Ok(PathBuf::from("C:\\ProgramData\\.emergency-backup"))
        } else {
            let homedir_path = get_my_home()?;
            if homedir_path.is_none() {
                bail!(HomeDirectoryError)
            }
            debug!(target: "general", "homedir_path_option: {:?}", &homedir_path);

            let homedir_string = homedir_path.unwrap().into_os_string().into_string();

            if homedir_string.is_err() {
                bail!(HomeDirectoryError)
            }
            debug!(target: "general", "homedir_string: {:?}", &homedir_string);

            Ok(Path::new(&homedir_string.unwrap()).join(".emergency-backup"))
        }

    }

    fn read_or_create_config(config_path: &PathBuf) -> anyhow::Result<BackupConfig> {
        debug!(target: "general", "read_or_create_config START");
        if config_path.exists() {
            debug!(target: "general", "Config file already exists");
            read_config(&config_path)
        } else {
            debug!(target: "general", "Config file NOT exists");
            create_config(&config_path)
        }
    }

    fn read_config(path: &Path) -> anyhow::Result<BackupConfig> {
        debug!(target: "general", "read_config START");
        let mut file = File::open(path)?;
        let mut yaml_string = String::new();
        let _ = file.read_to_string(&mut yaml_string);
        //let reader = BufReader::new(file);

        match serde_yaml::from_str(&*yaml_string) {
            Ok(config) => { debug!(target: "general", "Config read: {:?}", config); Ok(config) }
            Err(_) => { bail!(LoadSettingsError) }
        }
    }
    fn create_config(path: &Path) -> anyhow::Result<BackupConfig> {
        debug!(target: "general", "create_config START");
        let parent_path = path.parent();

        if parent_path.is_none() {
            bail!(ParentPathError)
        }
        fs::create_dir_all(parent_path.unwrap())?;
        let config = BackupConfig::default();

        match File::create(path) {
            Ok(_) => { debug!(target: "general", "Config file created")}
            Err(err) => { error!(target: "general", "Error creating configuration file: {}", err)}
        };

        match apply_settings(&config) {
            Ok(_) => { debug!(target: "general", "Configuration file created"); Ok(config) }
            Err(_) => { error!(target: "general", "ERROR creating configuration file"); bail!(ApplySettingsError) }
        }
    }

    pub fn set_tracker_on() -> anyhow::Result<()> {
        let mut config = match load_settings() {
            Ok(config) => { config }
            Err(_) => { bail!(LoadSettingsError) }
        };
        config.active = true;
        apply_settings(&config)
    }

    pub fn set_tracker_off() -> anyhow::Result<()> {
        let mut config = match load_settings() {
            Ok(config) => { config }
            Err(_) => { bail!(LoadSettingsError) }
        };
        config.active = false;
        apply_settings(&config)
    }

    pub fn apply_settings(config: &BackupConfig) -> anyhow::Result<()> {
        debug!(target: "general", "apply_settings START");
        let path = get_config_path()?;
        debug!(target: "general", "Saving settings into config file: {:?}", path);

        //1. Open file and write the settings
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        match serde_yaml::to_writer(writer, &config) {
            Ok(_) => { debug!(target: "general", "Configuration saved"); Ok(()) }
            Err(_err) => { bail!(ApplySettingsError) }
        }
    }

    pub fn set_millis_update_time(time: usize) -> anyhow::Result<()>{
        if time <= 0 {
            bail!(MillisUpdateFrequencyError)
        }
        match load_settings() {
            Ok(mut config) => {
                config.millis_update_frequency = time;
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_tolerance(tolerance: u32) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                config.tolerance = tolerance;
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_source(source: PathBuf) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                match source.canonicalize() {
                    Ok(full_path) => {
                        config.backup_source = full_path.to_string_lossy().to_string();
                        apply_settings(&config)
                    }
                    Err(err) => { bail!(err)}
                }
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_destination(destination: PathBuf) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                match destination.canonicalize() {
                    Ok(full_path) => {
                        config.backup_destination = full_path.to_string_lossy().to_string();
                        apply_settings(&config)
                    }
                    Err(err) => { bail!(err) }
                }

            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_extension_only(extension_only: bool) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                config.extension_only = extension_only;
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_mode(mode: String) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                config.mode = mode;
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_tracking_window_sec(secs: usize) -> anyhow::Result<()>{
        if secs <= 0 {
            bail!(ZeroTrackingWindowSecError)
        }
        match load_settings() {
            Ok(mut config) => {
                config.tracking_window_sec = secs;
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_extension_types(types: Vec<String>) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                config.extension_type = types;
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }

    pub fn set_installation_dir(path: String) -> anyhow::Result<()>{
        match load_settings() {
            Ok(mut config) => {
                config.installation_dir = Some(path);
                apply_settings(&config)
            }
            Err(err) => { bail!(err) }
        }
    }
}