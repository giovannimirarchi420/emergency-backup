# mouse_tracker - Emergency Backup Daemon

`mouse_tracker` is a system daemon application that serves as the backend for the emergency backup tool. It runs continuously in the background, monitoring user mouse movements to detect the specific gesture that triggers the emergency backup process.

## Key Features:

- **Daemon Mode**: Operates as a system daemon, ensuring continuous monitoring even after user login sessions end.
- **Mouse Gesture Detection**: Listens for specific mouse movements to trigger the backup initiation.
- **Confirmation Notification**: Displays a notification window to confirm the backup request, preventing accidental triggers.
- **U-Shape Gesture Confirmation**: Requires an additional U-shape gesture to proceed with the backup, ensuring user intention.
- **CPU Usage Logging**: Records CPU consumption data during the backup process for monitoring and analysis.
- **Backup Destination Management**: Saves backups to the user-defined destination folder, creating an emergency-backup subfolder for each backup.
- **Backup Summary Generation**: Generates a backup summary file containing CPU time and backup size information.

## Technical Details:

- **Operating System**: Compatible with Windows and Linux systems.
- **Logging**: Utilizes a dedicated log file (`cpu-consumption.log`) located in the directory provided in the installation phase to record CPU usage data.

### How it monitor the user mouse movements

The core functionality lies within the `mouse_tracker` executable, which must run continuously in the background for the Emergency Backup Application to function properly. The executable remains active until the user specifically disables it.

To achieve this persistent operation and automatic startup, different strategies are employed depending on the operating system:

#### Unix OS

On Unix-based systems, we leverage the operating system's service manager (such as `launchd` or `systemd`). This manager ensures the process runs continuously without our intervention.  Starting and stopping the process simply involves installing or uninstalling it as a system service.

#### Windows

The initial approach mirrored the Unix strategy, but complexities arose due to the differences between Windows Services and the sc.exe utility. Issues encountered with Windows Services included:

- **Sandboxing**: Each Windows Service runs within an isolated sandbox environment. This isolation prevents direct mouse movement tracking from within the service. An additional client application would be necessary to monitor the mouse and notify the service of relevant events.
- **Executable Restrictions**: sc.exe cannot directly execute arbitrary executables as Windows Services. Workarounds like [Shawl](https://github.com/mtkennerly/shawl) or [NSSM](https://nssm.cc/) would be required for functionality.

Therefore, the chosen solution for Windows utilizes the [Windows Registry Keys](https://en.wikipedia.org/wiki/Windows_Registry) to mimic the behavior of the Unix system manager. Upon user activation (Start), the mouse_tracker agent is registered in the following Windows Registry key:

```
HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
```

This ensures Windows automatically launches the executable during system startup. However, simply adding the registry entry isn't enough. To initiate the mouse_tracker immediately after user activation, the executable is also executed directly after its registration in the registry.

The user-initiated "Stop" action reverses this process. The registry entry under `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` is deleted, and any running instances of the mouse_tracker agent are terminated.
## Role in the Emergency Backup System:

`mouse_tracker` plays a crucial role in the emergency backup system by providing the real-time monitoring and gesture detection capabilities that enable users to initiate a backup with a simple mouse action. Its seamless integration with the system and logging features ensure a robust and reliable backup solution.

Interaction with `embctl` and `embgui`:

`mouse_tracker` communicates with the `embctl` command-line tool and `embgui` graphical user interface to receive configuration information and initiate backup processes. It acts as the backend service that handles the actual gesture detection and backup triggering based on the user's actions.

## Deployment:

`mouse_tracker` is typically installed by the `embctl` or `embgui` tool (on start command), which bundles the daemon along with the other components of the emergency backup tool. Once installed, it runs as a system daemon, automatically starting on system boot and remaining active in the background to monitor for user gestures.