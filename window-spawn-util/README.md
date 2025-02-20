# window-spawn-util - Window Spawning Utility for Error, Success, and Backup Detection

window-spawn-util is a library that facilitates the creation, display, and management of graphical windows for error, success, and backup detection notifications. It leverages the power of the `slint-interpreter`, `slint`, and `tokio` libraries to seamlessly integrate with the `Slint windowing framework` and provide asynchronous runtime execution.

## Key Features:

- **Slint Window Generation**: Generates Slint window definitions from provided SLINT files at runtime.
- **Window Display and Management**: Displays and manages Slint windows for error, success, and backup detection notifications.
- **Asynchronous Execution**: Utilizes the tokio library for asynchronous execution, ensuring non-blocking operation and responsiveness.

## Benefits:

- **Simplified Window Creation**: Streamlines the process of creating Slint windows from SLINT files, reducing development effort.
- **Flexible Window Management**: Provides a centralized approach to managing error, success, and backup detection notification windows.
- **Non-Blocking Operation**: Ensures a responsive user interface by handling window operations asynchronously.

## Integration with embctl and embgui:

window-spawn-util is integrated into the embctl and embgui applications to handle the display of error, success, and backup detection notification windows. These applications utilize window-spawn-util to generate Slint window definitions from SLINT files, display the corresponding windows, and manage their lifecycle.
