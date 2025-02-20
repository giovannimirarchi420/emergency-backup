# config - Configuration Utility Library

This crate defines two functionalities: application configuration and logger configuration.

## Application Configuration (AppConfig struct and app_config function)

- Defines a struct named `AppConfig` to hold various configuration options for the application.
    - These options include labels, summaries, and body text for notifications, window sizes for different UI elements, and CPU consumption log interval.
- Provides a function named `app_config` that returns a default instance of `AppConfig` with pre-defined values for all the configuration options.

## Logger Configuration (log_config function)

- Defines a function named `log_config` that takes a boolean flag debug as input and returns a logger configuration object.
- The function configures two separate log files:
    - General log file: This logs general application events.
    - CPU consumption log file: This logs CPU consumption information.
- Both logs are written to the `.emergency-backup/logs` directory within the user's home directory.
- The configuration ensures:
    - When a log file reaches 1MB, it's compressed and archived in a new directory named `.emergency-backup/logs/old`.
    - Up to two archived files are kept using a rolling mechanism.
    - Once the limit of two archived files is reached, the oldest file is deleted.
- The function uses various crates to achieve this configuration:
    - `anyhow`: for error handling
    - `home-dir`: to get the user's home directory path
    - `log4rs`: for advanced logging configuration
    - `byte-unit`: to parse human-readable byte sizes