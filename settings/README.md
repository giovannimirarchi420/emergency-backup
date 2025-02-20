# settings - Configuration Management Library

settings is a crate that provides a structured and convenient way to manage the application configurations. It offers a simple API for reading, writing, and updating configuration values.

## Benefits:

- Simplified Configuration Management: Streamlines the process of managing application configurations, reducing boilerplate code.
- Type-Safe Configuration: Prevents runtime errors caused by invalid configuration values.
- Centralized Configuration Storage: Provides a centralized location for storing and accessing configuration data.
- Platform Independence: Works consistently across Windows and Unix environments.

## Integration with `embctl` and `embgui`:

settings plays a crucial role in the `embctl` and `embgui` applications by providing a unified interface for managing application configurations. These applications rely on `settings` to load, store, and update configuration settings related to various aspects of the emergency backup tool.

## Configuration File Locations:

- **Windows**: Configuration file and logs are saved in `C:\ProgramData\.emergency-backup`
- **Unix**: Configuration file and logs are saved in `$HOME/.emergency-backup`