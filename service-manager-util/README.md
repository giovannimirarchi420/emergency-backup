# service-manager-util - Service Management Utility Library

service-manager-util is a Rust library that provides a unified interface for interacting with various service management platforms, including systemd, Windows Service Manager, and Launchd. It is designed to simplify the management of system services, particularly for the embctl and embgui applications.

## Key Features:

- **Cross-Platform Support**: Works seamlessly with different service management systems on Linux, and macOS. Windows systems uses Window Registry Keys.
- **Service Installation**: Installs services with the appropriate configuration for each platform.
- **Service Control**: Starts, stops, and uninstalls services using platform-specific APIs.
- **Notification Support**: Sends system notifications when backup trigger events are detected.

## Technical Details:

- **System Service Interface**: Leverages the service-manager crate to interact with system services in unix systems. Windows systems leverages on a Windows Registry Key. 
- **Notification Mechanism**: Employs the notify crate to deliver system notifications.

## Benefits:

- **Simplified Service Management**: Provides a consistent interface for managing services across different platforms.
- **Reduced Development Effort**: Abstracts away the complexities of platform-specific service management APIs.
- **Enhanced User Experience**: Enables embctl and embgui to deliver a more consistent and user-friendly experience for service management tasks.