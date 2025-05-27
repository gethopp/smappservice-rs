//! # SMAppService-RS
//!
//! A Rust wrapper for macOS's ServiceManagement framework, specifically the SMAppService API.
//!
//! ## Overview
//!
//! The ServiceManagement framework in macOS provides a way for applications to manage system services.
//! This library wraps the objc2-service-management API in a Rust-friendly interface, allowing to:
//!
//! - Register applications as login items
//! - Register and manage launch agents and daemons from the application bundle
//! - Check the status of registered services
//!
//! ## Example Usage
//!
//! ```rust
//! use smappservice_rs::{AppService, ServiceType, ServiceStatus};
//!
//! // Register the current application as a login item
//! let app_service = AppService::new(ServiceType::MainApp);
//! match app_service.register() {
//!     Ok(()) => println!("Application registered successfully as login item!"),
//!     Err(e) => eprintln!("Failed to register application: {}", e),
//! }
//!
//! // Check the registration status
//! let status = app_service.status();
//! println!("Service status: {}", status);
//!
//! // Register a LaunchAgent
//! let agent_service = AppService::new(ServiceType::Agent {
//!     plist_name: "com.example.myapp.agent.plist"
//! });
//! if let Err(e) = agent_service.register() {
//!     eprintln!("Failed to register agent: {}", e);
//! }
//!
//! // Open System Settings to manage login items
//! AppService::open_system_settings_login_items();
//!
//! // Register a LaunchDaemon
//! let daemon_service = AppService::new(ServiceType::Daemon {
//!     plist_name: "com.example.myapp.daemon.plist"
//! });
//! if let Err(e) = daemon_service.register() {
//!     eprintln!("Failed to register daemon: {}", e);
//! }
//!
//! // Register a helper application as a login item
//! let login_item = AppService::new(ServiceType::LoginItem {
//!     identifier: "com.example.helper"
//! });
//! if let Err(e) = login_item.register() {
//!     eprintln!("Failed to register login item: {}", e);
//! }
//!

use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_service_management::{
    kSMErrorAlreadyRegistered, kSMErrorAuthorizationFailure, kSMErrorInternalFailure,
    kSMErrorInvalidPlist, kSMErrorInvalidSignature, kSMErrorJobMustBeEnabled, kSMErrorJobNotFound,
    kSMErrorJobPlistNotFound, kSMErrorLaunchDeniedByUser, kSMErrorServiceUnavailable,
    kSMErrorToolNotValid, SMAppService, SMAppServiceStatus,
};
use thiserror::Error;

/// Represents the various types of services that can be registered with the ServiceManagement framework.
///
/// This enum is used to specify which kind of service you want to register when creating an `AppService`.
#[derive(Debug, Clone)]
pub enum ServiceType<'a> {
    /// An app service object that corresponds to the main application as a login item.
    ///
    /// This can be used to configure the main app to launch at login.
    MainApp,

    /// An app service object with a launch agent with the property list name you provide.
    ///
    /// The property list name must correspond to a property list in the calling app’s
    /// Contents/Library/LaunchAgents directory.
    ///
    /// # Parameters
    /// * `plist_name` - The name of the property list file (e.g., "com.example.myapp.agent.plist")
    Agent { plist_name: &'a str },

    /// An app service object with a launch daemon with the property list name you provide.
    ///
    /// The property list name must correspond to a property list in the calling app’s
    /// Contents/Library/LaunchDaemons directory
    ///
    /// # Parameters
    /// * `plist_name` - The name of the property list file (e.g., "com.example.myapp.daemon.plist")
    Daemon { plist_name: &'a str },

    /// An app service object for a login item corresponding to the bundle with the identifier you provide.
    ///
    /// The bundle name must correspond to a bundle in the calling app’s Contents/Library/LoginItems directory
    ///
    /// # Parameters
    /// * `identifier` - The bundle identifier of the helper application (e.g., "com.example.helper")
    LoginItem { identifier: &'a str },
}

/// Represents the status of a service registration.
///
/// This enum corresponds to the `SMAppServiceStatus` values in the ServiceManagement framework.
/// It provides information about the current state of a registered service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(isize)]
pub enum ServiceStatus {
    /// The service hasn't registered with the Service Management framework,
    /// or the service attempted to reregister after it was already registered.
    NotRegistered = SMAppServiceStatus::NotRegistered.0,

    /// The service has been successfully registered and is eligible to run.
    Enabled = SMAppServiceStatus::Enabled.0,

    /// The service has been successfully registered, but the user needs to take action in System Settings.
    ///
    /// The Service Management framework successfully registered this service, but the user needs to
    /// approve it in System Settings before the service is eligible to run. The framework also
    /// returns this status if the user revokes consent for the service to run.
    RequiresApproval = SMAppServiceStatus::RequiresApproval.0,

    /// An error occurred and the framework couldn't find this service.
    NotFound = SMAppServiceStatus::NotFound.0,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::NotRegistered => write!(f, "Not Registered"),
            ServiceStatus::Enabled => write!(f, "Enabled"),
            ServiceStatus::RequiresApproval => write!(f, "Requires Approval"),
            ServiceStatus::NotFound => write!(f, "Not Found"),
        }
    }
}

impl TryFrom<isize> for ServiceStatus {
    type Error = ();

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        match value {
            x if x == SMAppServiceStatus::NotRegistered.0 => Ok(ServiceStatus::NotRegistered),
            x if x == SMAppServiceStatus::Enabled.0 => Ok(ServiceStatus::Enabled),
            x if x == SMAppServiceStatus::RequiresApproval.0 => Ok(ServiceStatus::RequiresApproval),
            x if x == SMAppServiceStatus::NotFound.0 => Ok(ServiceStatus::NotFound),
            _ => Err(()),
        }
    }
}

/// Represents errors that can occur when registering or unregistering services.
///
/// This enum wraps the error codes returned by the ServiceManagement framework.
#[derive(Debug, Error, PartialEq)]
#[repr(u32)]
pub enum ServiceManagementError {
    /// An internal failure has occurred in the ServiceManagement framework.
    #[error("an internal failure has occurred")]
    InternalFailure = kSMErrorInternalFailure,

    /// The app's code signature doesn't meet the requirements to perform the operation.
    ///
    /// This often occurs when the application is not properly signed or lacks the required entitlements.
    #[error("the app's code signature doesn't meet the requirements to perform the operation")]
    InvalidSignature = kSMErrorInvalidSignature,

    /// The authorization requested failed.
    #[error("the authorization requested failed")]
    AuthorizationFailure = kSMErrorAuthorizationFailure,

    /// The specified path doesn't exist or the helper tool at the specified path isn't valid.
    #[error(
        "the specified path doesn't exist or the helper tool at the specified path isn't valid"
    )]
    ToolNotValid = kSMErrorToolNotValid,

    /// The system can't find the specified job.
    #[error("the system can't find the specified job")]
    JobNotFound = kSMErrorJobNotFound,

    /// The service necessary to perform this operation is unavailable or is no longer accepting requests.
    #[error(
        "the service necessary to perform this operation is unavailable or is no longer accepting requests"
    )]
    ServiceUnavailable = kSMErrorServiceUnavailable,

    /// The system can't find the app's property list file.
    #[error("the system can't find the app's property list")]
    JobPlistNotFound = kSMErrorJobPlistNotFound,

    /// The job must be enabled before performing the requested operation.
    #[error("the job must be enabled")]
    JobMustBeEnabled = kSMErrorJobMustBeEnabled,

    /// The app's property list is invalid or contains errors.
    #[error("the app's property list is invalid")]
    InvalidPlist = kSMErrorInvalidPlist,

    /// The user denied the app's launch request through a system prompt.
    #[error("the user denied the app's launch request")]
    LaunchDeniedByUser = kSMErrorLaunchDeniedByUser,

    /// The application is already registered with the ServiceManagement framework.
    #[error("the application is already registered")]
    AlreadyRegistered = kSMErrorAlreadyRegistered,

    /// An unrecognized error code was returned by the ServiceManagement framework.
    #[error("unknown error {0}")]
    Unknown(u32),
}

impl ServiceManagementError {
    /// Returns the error code associated with this error.
    ///
    /// This method returns the underlying error code that corresponds to the
    /// ServiceManagement framework error constants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use smappservice_rs::ServiceManagementError;
    ///
    /// let error = ServiceManagementError::InvalidSignature;
    /// let code = error.code();
    /// println!("Error code: {}", code);
    /// ```
    pub fn code(&self) -> u32 {
        match self {
            ServiceManagementError::InternalFailure => kSMErrorInternalFailure,
            ServiceManagementError::InvalidSignature => kSMErrorInvalidSignature,
            ServiceManagementError::AuthorizationFailure => kSMErrorAuthorizationFailure,
            ServiceManagementError::ToolNotValid => kSMErrorToolNotValid,
            ServiceManagementError::JobNotFound => kSMErrorJobNotFound,
            ServiceManagementError::ServiceUnavailable => kSMErrorServiceUnavailable,
            ServiceManagementError::JobPlistNotFound => kSMErrorJobPlistNotFound,
            ServiceManagementError::JobMustBeEnabled => kSMErrorJobMustBeEnabled,
            ServiceManagementError::InvalidPlist => kSMErrorInvalidPlist,
            ServiceManagementError::LaunchDeniedByUser => kSMErrorLaunchDeniedByUser,
            ServiceManagementError::AlreadyRegistered => kSMErrorAlreadyRegistered,
            ServiceManagementError::Unknown(code) => *code,
        }
    }
}

impl TryFrom<u32> for ServiceManagementError {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            #[allow(non_upper_case_globals)]
            kSMErrorInternalFailure => Ok(ServiceManagementError::InternalFailure),
            #[allow(non_upper_case_globals)]
            kSMErrorInvalidSignature => Ok(ServiceManagementError::InvalidSignature),
            #[allow(non_upper_case_globals)]
            kSMErrorAuthorizationFailure => Ok(ServiceManagementError::AuthorizationFailure),
            #[allow(non_upper_case_globals)]
            kSMErrorToolNotValid => Ok(ServiceManagementError::ToolNotValid),
            #[allow(non_upper_case_globals)]
            kSMErrorJobNotFound => Ok(ServiceManagementError::JobNotFound),
            #[allow(non_upper_case_globals)]
            kSMErrorServiceUnavailable => Ok(ServiceManagementError::ServiceUnavailable),
            #[allow(non_upper_case_globals)]
            kSMErrorJobPlistNotFound => Ok(ServiceManagementError::JobPlistNotFound),
            #[allow(non_upper_case_globals)]
            kSMErrorJobMustBeEnabled => Ok(ServiceManagementError::JobMustBeEnabled),
            #[allow(non_upper_case_globals)]
            kSMErrorInvalidPlist => Ok(ServiceManagementError::InvalidPlist),
            #[allow(non_upper_case_globals)]
            kSMErrorLaunchDeniedByUser => Ok(ServiceManagementError::LaunchDeniedByUser),
            #[allow(non_upper_case_globals)]
            kSMErrorAlreadyRegistered => Ok(ServiceManagementError::AlreadyRegistered),
            _ => Err(()),
        }
    }
}

/// The main struct for interacting with macOS's ServiceManagement framework.
///
/// `AppService` provides methods to register, unregister, and check the status of various
/// types of services, such as login items, launch agents, and daemons.
pub struct AppService {
    service: Retained<SMAppService>,
}

impl AppService {
    /// Creates a new `AppService` instance for the specified service type.
    ///
    /// This method creates a new service handle but does not register it.
    /// To register the service, call the [`register`](#method.register) method.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use smappservice_rs::{AppService, ServiceType};
    ///
    /// // Create a service for the main application
    /// let main_app = AppService::new(ServiceType::MainApp);
    ///
    /// // Create a service for a LaunchAgent
    /// let agent = AppService::new(ServiceType::Agent {
    ///     plist_name: "com.example.myapp.agent.plist"
    /// });
    ///
    /// // Create a service for a LaunchDaemon
    /// let daemon = AppService::new(ServiceType::Daemon {
    ///     plist_name: "com.example.myapp.daemon.plist"
    /// });
    ///
    /// // Create a service for a login item
    /// let login_item = AppService::new(ServiceType::LoginItem {
    ///     identifier: "com.example.helper"
    /// });
    /// ```
    pub fn new(service_type: ServiceType) -> Self {
        let service = match service_type {
            ServiceType::MainApp => unsafe { SMAppService::mainAppService() },
            ServiceType::Agent { plist_name } => unsafe {
                let input_arg = NSString::from_str(plist_name);
                SMAppService::agentServiceWithPlistName(&input_arg)
            },
            ServiceType::Daemon { plist_name } => unsafe {
                let input_arg = NSString::from_str(plist_name);
                SMAppService::daemonServiceWithPlistName(&input_arg)
            },
            ServiceType::LoginItem { identifier } => unsafe {
                let input_arg = NSString::from_str(identifier);
                SMAppService::loginItemServiceWithIdentifier(&input_arg)
            },
        };
        Self { service }
    }

    /// Registers the service so it can begin launching according to its configuration.
    ///
    /// The behavior and requirements differ based on the service type:
    ///
    /// - **Login Item**: The helper starts immediately and on subsequent logins.
    ///   If it crashes or exits with a non-zero status, the system relaunches it.
    ///
    /// - **Main Application**: The application launches on subsequent logins.
    ///
    /// - **LaunchAgent**: The agent is immediately bootstrapped and may begin running.
    ///   LaunchAgents registered with this method bootstrap on each subsequent login.
    ///   If registering a LaunchAgent for multiple users, you must call this method
    ///   once per user while that user is running the app.
    ///
    /// - **LaunchDaemon**: The system won't bootstrap the daemon until an admin
    ///   approves it in System Settings. After approval, the system bootstraps
    ///   the daemon on each subsequent boot.
    ///
    /// # Errors
    ///
    /// Returns a `ServiceManagementError` if:
    /// - The service is already registered (`AlreadyRegistered`)
    /// - The user denies the launch request (`LaunchDeniedByUser`)
    /// - The app's code signature is invalid (`InvalidSignature`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use smappservice_rs::{AppService, ServiceType};
    ///
    /// let app_service = AppService::new(ServiceType::MainApp);
    /// match app_service.register() {
    ///     Ok(()) => println!("Application registered successfully!"),
    ///     Err(e) => eprintln!("Failed to register application: {}", e),
    /// }
    /// ```
    pub fn register(&self) -> Result<(), ServiceManagementError> {
        match unsafe { self.service.registerAndReturnError() } {
            Ok(()) => Ok(()),
            Err(error) => {
                let error_code = error.code() as u32;
                Err(ServiceManagementError::try_from(error_code)
                    .unwrap_or(ServiceManagementError::Unknown(error_code)))
            }
        }
    }

    /// Un registers the service, preventing it from launching automatically in the future.
    ///
    /// This is the opposite operation of the [`register`](#method.register) method.
    ///
    /// The behavior depends on the service type:
    ///
    /// - **Login Item**, **LaunchAgent**, or **LaunchDaemon**: If the service is currently
    ///   running, the system terminates it and prevents future launches.
    ///
    /// - **Main Application**: The application continues running if it's already running,
    ///   but becomes unregistered to prevent future launches at login.
    ///
    /// # Errors
    ///
    /// Returns a `ServiceManagementError` if:
    /// - The service is not registered (`JobNotFound`)
    /// - Any other error occurs during unregistration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use smappservice_rs::{AppService, ServiceType};
    ///
    /// let app_service = AppService::new(ServiceType::MainApp);
    /// match app_service.unregister() {
    ///     Ok(()) => println!("Application unregistered successfully!"),
    ///     Err(e) => eprintln!("Failed to unregister application: {}", e),
    /// }
    /// ```
    pub fn unregister(&self) -> Result<(), ServiceManagementError> {
        match unsafe { self.service.unregisterAndReturnError() } {
            Ok(()) => Ok(()),
            Err(error) => {
                let error_code = error.code() as u32;
                Err(ServiceManagementError::try_from(error_code)
                    .unwrap_or(ServiceManagementError::Unknown(error_code)))
            }
        }
    }

    /// Opens the Login Items section in System Settings.
    ///
    /// Use this method to direct the user to the system UI where they can manually
    /// enable, disable, or view registered login items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use smappservice_rs::AppService;
    ///
    /// // Open System Settings to the Login Items section
    /// AppService::open_system_settings_login_items();
    /// ```
    pub fn open_system_settings_login_items() {
        unsafe { SMAppService::openSystemSettingsLoginItems() }
    }

    /// Checks the current registration status of the service.
    ///
    /// Returns a [`ServiceStatus`] enum value indicating whether the service is:
    /// - `Enabled`: Registered and eligible to run
    /// - `RequiresApproval`: Registered but requires user approval
    /// - `NotRegistered`: Not registered or attempted to reregister
    /// - `NotFound`: Cannot be found by the framework
    ///
    /// # Examples
    ///
    /// ```rust
    /// use smappservice_rs::{AppService, ServiceType, ServiceStatus};
    ///
    /// let app_service = AppService::new(ServiceType::MainApp);
    /// let status = app_service.status();
    /// println!("Service status: {}", status);
    ///
    /// if status == ServiceStatus::RequiresApproval {
    ///     println!("Please approve the service in System Settings");
    ///     AppService::open_system_settings_login_items();
    /// }
    /// ```
    pub fn status(&self) -> ServiceStatus {
        let status = unsafe { self.service.status() };
        match ServiceStatus::try_from(status.0) {
            Ok(status) => status,
            Err(_) => ServiceStatus::NotFound,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_status() {
        let service_types: [ServiceType; 4] = [
            ServiceType::MainApp,
            ServiceType::Agent {
                plist_name: "com.example.smappservice-test-app.plist",
            },
            ServiceType::Daemon {
                plist_name: "com.example.smappservice-test-app.plist",
            },
            ServiceType::LoginItem {
                identifier: "com.example.loginitem",
            },
        ];
        for service_type in service_types {
            let service = AppService::new(service_type);
            let status = service.status();
            assert!(
                status == ServiceStatus::NotFound,
                "Service status should be NotFound"
            );
        }
    }

    #[test]
    fn test_service_management_error_code() {
        // Test known error variants
        assert_eq!(
            ServiceManagementError::InternalFailure.code(),
            kSMErrorInternalFailure
        );
        assert_eq!(
            ServiceManagementError::InvalidSignature.code(),
            kSMErrorInvalidSignature
        );
        assert_eq!(
            ServiceManagementError::AuthorizationFailure.code(),
            kSMErrorAuthorizationFailure
        );
        assert_eq!(
            ServiceManagementError::ToolNotValid.code(),
            kSMErrorToolNotValid
        );
        assert_eq!(
            ServiceManagementError::JobNotFound.code(),
            kSMErrorJobNotFound
        );
        assert_eq!(
            ServiceManagementError::ServiceUnavailable.code(),
            kSMErrorServiceUnavailable
        );
        assert_eq!(
            ServiceManagementError::JobPlistNotFound.code(),
            kSMErrorJobPlistNotFound
        );
        assert_eq!(
            ServiceManagementError::JobMustBeEnabled.code(),
            kSMErrorJobMustBeEnabled
        );
        assert_eq!(
            ServiceManagementError::InvalidPlist.code(),
            kSMErrorInvalidPlist
        );
        assert_eq!(
            ServiceManagementError::LaunchDeniedByUser.code(),
            kSMErrorLaunchDeniedByUser
        );
        assert_eq!(
            ServiceManagementError::AlreadyRegistered.code(),
            kSMErrorAlreadyRegistered
        );

        // Test unknown error variant
        let unknown_code = 9999u32;
        assert_eq!(
            ServiceManagementError::Unknown(unknown_code).code(),
            unknown_code
        );
    }
}
