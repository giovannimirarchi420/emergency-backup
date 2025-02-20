use anyhow::bail;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::{Config};
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter};
use errors::Error::{LoadSettingsError};
use settings::settings::{get_config_dir};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub serviced_label: String,
    pub first_command_notification_summary: String,
    pub first_command_notification_body: String,
    pub second_command_notification_summary: String,
    pub second_command_notification_body: String,
    pub backup_done_summary: String,
    pub backup_done_body: String,
    pub backup_error_summary: String,
    pub backup_error_body: String,
    pub pop_up_window_size: (f32, f32),
    pub main_window_gui_size: (f32, f32),
    pub installation_window_gui_size: (f32, f32),
    pub cpu_consumption_log_interval_msec: u128
}

pub fn app_config() -> AppConfig {
    AppConfig {
        serviced_label: String::from("it.gmirarchi.mouse_tracker"),
        first_command_notification_summary: String::from("Mouse Tracker"),
        first_command_notification_body: String::from("Mouse command detected, perform the second step to start the backup"),
        second_command_notification_summary: String::from("Mouse Tracker"),
        second_command_notification_body: String::from("Mouse command detected, backup started"),
        backup_done_summary: String::from("Mouse Tracker"),
        backup_done_body: String::from("Backup successfully done."),
        backup_error_summary: String::from("Mouse Tracker Error."),
        backup_error_body: String::from("An error occurred during the backup, please try again."),
        pop_up_window_size: (400f32, 100f32),
        main_window_gui_size: (1024f32, 512f32),
        installation_window_gui_size: (512f32, 256f32),
        cpu_consumption_log_interval_msec: 120000
    }
}


/// Returns the configuration for the logger.

/// The logger configuration writes logs to the `.emergency-backup/logs` directory within the user's home directory.
/// When the log file reaches 1MB in size, it's compressed (gzip) and archived in a new directory called `.emergency-backup/logs/old`.
/// Up to two archived log files are stored using a rolling mechanism.
/// Once the limit of five archived files is reached, the oldest file is deleted to make space for the newest compressed log file.
pub fn log_config(debug: bool) -> anyhow::Result<Config> {
    let log_line_pattern = "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}";

    let config_dir = match get_config_dir() {
        Ok(config_dir) => { config_dir }
        Err(_) => { bail!(LoadSettingsError) }
    };

    let trigger_size = byte_unit::Byte::parse_str("1mb", false).unwrap().as_u64();
    let trigger_general = Box::new(SizeTrigger::new(trigger_size.clone()));
    let trigger_cpu_consumption = Box::new(SizeTrigger::new(trigger_size));

    let roller_pattern_general = config_dir.clone().join("logs/old/emergency_backup_{}.log.gz");
    let roller_pattern_cpu_consumption = config_dir.clone().join("logs/old/cpu_consumptions_{}.log.gz");

    let roller_count = 2; //How many old log files to keep
    let roller_base = 1;
    let roller_general = Box::new(
        FixedWindowRoller::builder()
            .base(roller_base)
            .build(roller_pattern_general.to_str().unwrap(), roller_count)
            .unwrap(),
    );

    let roller_cpu_consumption = Box::new(
        FixedWindowRoller::builder()
            .base(roller_base)
            .build(roller_pattern_cpu_consumption.to_str().unwrap(), roller_count)
            .unwrap(),
    );

    let compound_policy_general = Box::new(CompoundPolicy::new(trigger_general, roller_general));
    let compound_policy_cpu_consumption = Box::new(CompoundPolicy::new(trigger_cpu_consumption, roller_cpu_consumption));

    let general_ap = match RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_line_pattern)))
        .build(config_dir.clone().join("logs/emergency_backup.log"), compound_policy_general) {
            Ok(appender) => { appender },
            Err(err) => { eprintln!("RollingFileAppender 'general' creation failed: {:?}", err); bail!(err) }
        };

    let cpu_consumption_ap = match RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(log_line_pattern)))
        .build(config_dir.clone().join("logs/cpu_consumption.log"), compound_policy_cpu_consumption) {
            Ok(appender) => { appender },
            Err(err) => { eprintln!("RollingFileAppender 'cpu_consumption' creation failed: {:?}", err); bail!(err) }
    };


    let stdout = ConsoleAppender::builder().build();

    match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("general_ap", Box::new(general_ap)))
        .appender(Appender::builder().build("cpu_consumption_ap", Box::new(cpu_consumption_ap)))
        .logger(
            Logger::builder()
                .appender("general_ap")
                .build("general", if debug { LevelFilter::Debug } else { LevelFilter::Info }),
        )
        .logger(
            Logger::builder()
                .appender("cpu_consumption_ap")
                .build("cpu_consumption", LevelFilter::Trace),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug)) {
        Ok(build) => { Ok(build) },
        Err(err) => { eprintln!("Log Configuration creation failed."); bail!(err) }
    }
}
