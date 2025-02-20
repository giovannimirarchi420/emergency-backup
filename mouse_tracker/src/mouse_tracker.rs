use std::cmp::PartialEq;
use std::path::{PathBuf};
use std::str::FromStr;
use std::thread::{sleep};
use std::time::{Duration, SystemTime};
use std::{fs, process,};
use std::fs::File;
use std::io::Write;
use anyhow::bail;
use cpu_time::ProcessTime;
use dircpy::{CopyBuilder};
use log::{debug, error, info, trace};

use perf_monitor::cpu::ProcessStat;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use config::app_config;
use errors::Error::{BackupModeNotRecognizedError, BackupPathNotConfigured, BackupSourceError, CpuConsumptionLoggingError, FileTransferError, MillisUpdateFrequencyError, ZeroTrackingWindowSecError};
use service_manager_util::notify;
use settings::settings::BackupConfig;
use window_spawn_util::pop_out_error_window;
use crate::data_type::screen_size::{ScreenSize};
use crate::data_type::mouse_position::{MousePosition};
use mouse_position::mouse_position::{Mouse};

#[cfg(target_family = "unix")] use std::{thread};

#[derive(Clone)]
pub struct MouseTracker {
    pub config: BackupConfig,
    pub screen_size: ScreenSize,
}

fn get_mouse_pos() -> MousePosition {

    let position = Mouse::get_mouse_position(); //Current mouse position

    match position {
        Mouse::Position { x, y } => {
            MousePosition::new((x, y))
        }
        Mouse::Error => {
            error!(target: "general", "Error detecting mouse position");
            MousePosition::new((-1, -1))
        }
    }
}
impl MouseTracker {
    pub fn from(config: BackupConfig) -> anyhow::Result<Self> {
        let _millis_update_frequency = match config.millis_update_frequency {
            0 => { bail!(MillisUpdateFrequencyError) }
            _ => { config.millis_update_frequency }
        };

        let _tracking_window_sec = match config.tracking_window_sec {
            0 => { bail!(ZeroTrackingWindowSecError) }
            _ => { config.tracking_window_sec }
        };

        let screen_size = match ScreenSize::new() {
            Ok(size) => {
                debug!(target: "general", "Screen corner size: {:?}", size);
                size
            }
            Err(screen_size_error) => { bail!(screen_size_error) }
        };

        let source_path = PathBuf::from_str(&*config.backup_source);
        let destination_path = PathBuf::from_str(&*config.backup_destination);

        if source_path.is_err() || destination_path.is_err() ||
            source_path.clone().unwrap().as_os_str().is_empty() || destination_path.clone().unwrap().as_os_str().is_empty() {
            error!(target: "general", "Source path: {:?}, Dest path; {:?}", source_path, destination_path);
            bail!(BackupPathNotConfigured)
        }

        Ok(MouseTracker {
            config,
            screen_size,
        })
    }

    pub fn start(self) -> () {
        match MouseTracker::tracking_loop(self.config.clone(), self.screen_size.clone(), false, None) {
            Ok(_) => { debug!(target: "general", "Tracking loop successfully started") }
            Err(err) => {
                error!(target: "general", "Error during tracking loop start: {:?}", err);
                pop_out_error_window(String::from("Error during tracking loop start"), Some(err.to_string()))
            }
        }
    }

    fn tracking_loop(config: BackupConfig, screen_size: ScreenSize, is_second_command: bool, first_command_time: Option<SystemTime>) -> anyhow::Result<()> {
        debug!(target: "general", "tracking_loop START");
        let buffer_size = 1000 / config.millis_update_frequency * config.tracking_window_sec;
        let mut mouse_position_buffer = AllocRingBuffer::new(buffer_size);
        let mut start_sys_time = SystemTime::now();
        let cpu_consumption_log_interval = app_config().cpu_consumption_log_interval_msec;
        let mut stat_p = ProcessStat::cur().unwrap();

        loop {
            let coordinates = get_mouse_pos();

            //Checking if the user started the backup mouse command
            if Self::is_touching_upper_left_corner(&coordinates, &mouse_position_buffer.to_vec()) {
                //Add new position to the position vector
                mouse_position_buffer.push(coordinates);
                let vec = mouse_position_buffer.to_vec();

                if Self::is_touching_lower_left_corner(&vec, &screen_size, config.tolerance) &&
                    Self::is_touching_lower_right_corner(&vec, &screen_size, config.tolerance) &&
                    Self::is_touching_upper_right_corner(&vec, &screen_size, config.tolerance) {
                    let app_config = app_config();

                    if is_second_command {
                        notify(app_config.second_command_notification_summary, app_config.second_command_notification_body);
                        info!(target: "general", "Backup starting..");

                        match Self::backup(config) {
                            Ok(_) => {
                                info!(target: "general", "Backup done.");
                                notify(app_config.backup_done_summary, app_config.backup_done_body);
                            }
                            Err(err) => {
                                error!(target: "general", "An error occurred during the Backup: {}", err);
                                notify(app_config.backup_error_summary, app_config.backup_error_body);
                            }
                        }

                        break Ok(());
                    } else {
                        info!(target: "general", "Mouse command detected: First time scenario, listening for the second one..");
                        notify(app_config.first_command_notification_summary, app_config.first_command_notification_body);
                        let config_bind = config.clone();
                        let screen_size_bind = screen_size.clone();
                        #[cfg(target_family = "unix")]
                        {
                            thread::spawn(move || {
                                match Self::tracking_loop(config_bind, screen_size_bind, true, Some(SystemTime::now())) {
                                    Ok(_) => { debug!(target: "general", "2nd tracking loop successfully started") }
                                    Err(err) => {
                                        error!(target: "general", "Error during 2nd tracking loop startup: {:?}", err);
                                        pop_out_error_window(String::from("Error during tracking loop start"), Some(err.to_string()))
                                    }
                                }
                            });
                            window_spawn_util::pop_out_deny_window(
                                String::from("Command detected, press cancel to stop the second command listening. \
                                The listening for the second command will be stopped anyway after".to_owned() + config.tracking_window_sec.to_string().as_str()));
                        }


                        #[cfg(target_family = "windows")]
                        {
                            match Self::tracking_loop(config_bind, screen_size_bind, true, Some(SystemTime::now())) {
                                Ok(_) => { debug!(target: "general", "2nd tracking loop successfully started") }
                                Err(err) => {
                                    error!(target: "general", "Error during 2nd tracking loop startup: {:?}", err);
                                    pop_out_error_window(String::from("Error during tracking loop start"), Some(err.to_string()))
                                }
                            }
                        }
                    }
                    mouse_position_buffer.clear();
                }
                //Checking if the user completed the backup mouse command
            }


            /***************************************
                      TRACKING WINDOW CHECK
            ****************************************/
            if is_second_command {
                let now_sys_time = SystemTime::now();
                match now_sys_time.duration_since(first_command_time.unwrap()) {
                    Ok(time_since) => {
                        if time_since.as_secs() > config.tracking_window_sec as u64 {
                            info!(target: "general", "Second command listening finish");
                            break Ok(());
                        }
                    }
                    Err(_) => { bail!(CpuConsumptionLoggingError) }
                }
            }

            /***************************************
                      CPU CONSUMPTION LOGGING
            ****************************************/
            let now_sys_time = SystemTime::now();
            match now_sys_time.duration_since(start_sys_time) {
                Ok(time_since) => {
                    if time_since.as_millis() > cpu_consumption_log_interval {
                        start_sys_time = SystemTime::now();
                        Self::log_cpu_consumption(&mut stat_p);
                    }
                }
                Err(_) => { bail!(CpuConsumptionLoggingError) }
            }

            //Implements mouse position sampling rate
            sleep(Duration::from_millis(config.millis_update_frequency as u64));
        }
    }

    fn is_touching_upper_left_corner(coordinates: &MousePosition, mouse_position_buffer: &Vec<MousePosition>) -> bool {
        coordinates.eq(&MousePosition::new((0, 0))) || mouse_position_buffer.contains(&MousePosition::new((0, 0)))
    }

    fn is_touching_lower_left_corner(mouse_position_buffer: &Vec<MousePosition>, screen_size: &ScreenSize, tolerance: u32) -> bool {
        for y in screen_size.max_height - tolerance..screen_size.max_height {
            if mouse_position_buffer.contains(&MousePosition::new((0, y as i32))) {
                return true;
            }
        }
        return false;
    }

    fn is_touching_lower_right_corner(mouse_position_buffer: &Vec<MousePosition>, screen_size: &ScreenSize, tolerance: u32) -> bool {
        for x in screen_size.max_width - tolerance..screen_size.max_width {
            for y in screen_size.max_height - tolerance..screen_size.max_height {
                if mouse_position_buffer.contains(&MousePosition::new((x as i32, y as i32))) {
                    return true;
                }
            }
        }
        return false;
    }

    fn is_touching_upper_right_corner(mouse_position_buffer: &Vec<MousePosition>, screen_size: &ScreenSize, tolerance: u32) -> bool {
        for x in screen_size.max_width - tolerance..screen_size.max_width {
            if mouse_position_buffer.contains(&MousePosition::new((x as i32, 0))) {
                return true;
            }
        }
        return false;
    }

    fn log_cpu_consumption(process_stat: &mut ProcessStat) {
        match process_stat.cpu() {
            Ok(stat) => {
                debug!(target: "general", "CPU consumption logged");
                trace!(target: "cpu_consumption", "[{}] [mouse_tracker] -> {:.9}% CPU", process::id(), stat * 100f64);
            }
            Err(err) => { error!("An error occurred reading process stats: {}", err) }
        }
    }

    fn backup(settings: BackupConfig) -> anyhow::Result<()> {
        let start = ProcessTime::now();

        let result = match settings.mode.to_lowercase().as_str() {
            "file" => { Self::file_backup(&settings) }
            "folder" => { Self::folder_backup(settings.clone()) }
            _ => { bail!(BackupModeNotRecognizedError) }
        };

        return if result.is_ok() {
            let cpu_time = start.elapsed();
            match Self::log_backup_cpu_time(cpu_time, PathBuf::from(&settings.backup_destination), PathBuf::from(&settings.backup_source)) {
                Ok(_) => { info!("Backup logs successfully written.")}
                Err(err) => { pop_out_error_window(String::from("Error writing backup logs"), Some(err.to_string()))}
            }
            result
        } else {
            Err(result.err().unwrap())
        }
    }

    fn log_backup_cpu_time(duration: Duration, target_path: PathBuf, source_path: PathBuf) -> anyhow::Result<()>{
        let target_file = target_path.join("backup_log_info.log");
        let mut file = match File::create(target_file) {
            Ok(file) => { debug!("Backup log file opened"); file }
            Err(err) => { error!("Error writing backup logs on target directory: {:?}", err); bail!(err) }
        };
        debug!(target: "general", "Backup path: {:?}", source_path);

        let backup_size = match fs_extra::dir::get_size(source_path) {
            Ok(size) => { (size as f64)/1024f64/1024f64 }
            Err(err) => { error!("Error calculating backup size, returning 0. \nError: {}", err); 0f64 }
        };
        let mut backup_message = String::from("Backup CPU time: ");

        backup_message.push_str(&*duration.as_millis().to_string());
        backup_message.push_str(" ms");
        backup_message.push_str("\n");
        backup_message.push_str("Backup size: ");
        backup_message.push_str(format!("{:.2}", backup_size).as_str());
        backup_message.push_str(" Mb");

        match file.write(backup_message.as_bytes()) {
            Ok(_) => { debug!("Backup log saved."); Ok(()) }
            Err(err) => { error!("An error occurred writing backup log: {:?}", err); bail!(err) }
        }
    }

    /// Function responsible to back up the entire folder (applying filters if needed, if extension_only is true
    fn folder_backup(settings: BackupConfig) -> anyhow::Result<()> {
        let destination_path = PathBuf::from(&settings.backup_destination).join("emergency-backup");
        let mut builder = CopyBuilder::new(&settings.backup_source, &destination_path);
        if settings.extension_only {
            debug!(target: "general", "Target extensions: {:?}", settings.extension_type);
            for ext in settings.extension_type {
                let mut dotted_ext = ext.clone();
                dotted_ext.insert(0, '.');
                builder = builder.with_include_filter(&dotted_ext);
            }
        }

        match builder
            .run() {
            Ok(_) => { Ok(()) }
            Err(err) => { error!(target: "general", "Ext-only directory copy error: {}", err.to_string()); bail!(FileTransferError) }
        }

    }

    fn file_backup(settings: &BackupConfig) -> anyhow::Result<()> {

        let binding = PathBuf::from(&settings.backup_source);

        let source_file_name = match binding.file_name() {
            None => { error!("Error to retrieve backup source file name"); bail!(BackupSourceError) }
            Some(file_name) => { file_name }
        };

        let destination_path = PathBuf::from(&settings.backup_destination)
            .join("emergency-backup");

        match fs::create_dir_all(&destination_path) {
            Ok(_) => {
                match fs::copy(PathBuf::from(&settings.backup_source), destination_path.join(source_file_name)) {
                    Ok(_) => { Ok(()) }
                    Err(err) => { error!("{:?}", err); bail!(err) }
                }
            }
            Err(err) => { error!("{:?}", err); bail!(err) }
        }
    }
}