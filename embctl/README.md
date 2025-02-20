# embctl - Emergency Backup Tool Command Line Interface

`embctl` is a command-line tool for managing emergency backups triggered by a specific mouse action. It provides functionalities to start, stop, and check the status of the emergency backup daemon process. Additionally, it allows configuring various aspects of the backup behavior.

## Features:

- Start/stop/check status of the emergency backup daemon
- View current configuration
- Set mouse position sampling frequency (affects CPU usage)
- Set time window for performing the mouse command to trigger backup
- Set tolerance for corner detection on the screen
- Define backup source path
- Define backup destination path (a new emergency-backup folder will be created)
- Configure extension-based backup (only files with specified extensions are copied)
- Set backup target file extensions
- Choose between file or folder backup mode

## Usage:
```bash
embctl [OPTIONS] [COMMAND]
```
## Commands:

- `start` - Starts the emergency backup daemon process.
- `stop` - Stops the emergency backup daemon process.
- `status` - Checks the status of the mouse tracker daemon service.
- `show-config` - Prints the current configuration settings.
- `set-millis-update-time` - Sets the mouse position sampling frequency in milliseconds (default: 200ms, higher values increase CPU usage).
- `set-tracking-window-sec` - Sets the time window (in seconds) within which the user must perform the mouse command to trigger the backup action (default: 15 seconds).
- `set-tolerance` - Sets the tolerance for touching in the corners of the display (default: 5 pixels).
- `set-source` - Defines the path to the source directory for backups.
- `set-destination` - Defines the path to the backup destination directory. A new emergency-backup folder will be created within this path to store the backed-up content.
- `set-extension-only` - Enables or disables extension-based backup (default: false). When enabled, only files with extensions specified in set-extension-type are copied.
- `set-extension-type` - Sets a comma or space-separated list of file extensions to be included in the backup (if set-extension-only is enabled).
- `set-mode` - Sets the backup mode to either 'file' or 'folder' (default: folder).
- `help` - Prints the help message or the help for a specific subcommand.

## Options:

- `-d, --debug` - Enables debug mode for more verbose logging.
- `-h, --help` - Prints this help message.
- `-V, --version` - Prints the version information of `embctl`.

## Technical Details

This Command Line Tool has been built leveraging on the [clap](https://docs.rs/clap/latest/clap/) Rust crate using the derive pattern.

## Notes for Windows Users

In order to be able to use the `embctl` tool on Windows, you need to start the tool directly from the installation folder. To avoid this, and be able to call the `embctl` tool from everywhere in your filesystem, you need to set you PATH environment variable adding the `embctl` folder in it.