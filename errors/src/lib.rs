use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum Error {
    /********************************************
     *              Settings Error              *
     ********************************************/
    #[error("Failed to retrieve user's home directory")]
    HomeDirectoryError,

    #[error("Unable to determine the backup mode")]
    ModeSelectionError,

    #[error("No backup path selected. Please choose a valid path")]
    NoPathChosenError,

    #[error("Configuration loading failed. Please try again.")]
    LoadSettingsError,

    #[error("Configuration saving failed. Please try again.")]
    ApplySettingsError,

    #[error("Failed to retrieve application configuration path")]
    ParentPathError,

    /********************************************
    *                 Slint Errors              *
    *********************************************/
    #[error("Error building the error window")]
    BuildErrorWindowsError,

    #[error("Error building the success window")]
    BuildSuccessWindowsError,

    #[error("Error building the deny window")]
    BuildDenyWindowsError,

    /********************************************
    *            Mouse Tracker Errors           *
    *********************************************/
    #[error("Primary display could not be found")]
    ScreenSizeError,

    #[error("Mouse sampling frequency must be a positive value")]
    MillisUpdateFrequencyError,

    #[error("Mouse tracking window cannot be zero seconds")]
    ZeroTrackingWindowSecError,

    #[error("Backup mode set to 'file', but a folder was provided")]
    FolderProvidedFileRequiredError,

    #[error("Backup mode set to 'folder', but a file was provided")]
    FileProvidedFolderRequiredError,

    #[error("Tolerance value must be non-negative")]
    ToleranceValueError,

    #[error("At least one file extension must be provided when using extension type option")]
    ExtensionTypeArrayEmptyError,

    #[error("File extension list must be provided in the format: 'txt|pdf|png ...'")]
    ExtensionTypeFormatError,

    #[error("Failed to create mouse tracker")]
    MouseTrackerCreationError,

    #[error("'extension_only' attribute must be either 'true' or 'false'")]
    ExtensionOnlyValueError,

    #[error("Invalid backup mode. Must be 'file' or 'folder'")]
    BackupModeNotRecognizedError,

    #[error("Failed to retrieve backup source")]
    BackupSourceError,

    #[error("File transfer failed")]
    FileTransferError,

    /********************************************
    *            Service Daemon Errors          *
    *********************************************/
    #[error("Failed to start daemon service manager")]
    DaemonStartupError,

    #[error("Service manager does not support user-level services")]
    UserLeverNotSupportedError,

    #[error("Failed to install daemon service. Try running as administrator")]
    DaemonInstallationError,

    #[error("Failed to uninstall daemon service")]
    DaemonUninstallationError,

    #[error("Failed to start daemon service")]
    DaemonStartError,

    #[error("Failed to stop daemon service")]
    DaemonStopError,

    #[error("Installation directory not found")]
    InstallationDirectoryNotFound,

    /********************************************
    *                Other errors               *
    *********************************************/

    #[error("Error calculating CPU consumption logging time")]
    CpuConsumptionLoggingError,

    #[error("Backup source and/or destination path not configured")]
    BackupPathNotConfigured,
}