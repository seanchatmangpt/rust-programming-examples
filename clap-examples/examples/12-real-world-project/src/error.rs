//! Error types for cloudctl

use thiserror::Error;

/// Main error type for cloudctl.
#[derive(Error, Debug)]
pub enum CloudError {
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Resource not found: {resource_type} '{name}'")]
    NotFound {
        resource_type: String,
        name: String,
    },

    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied {
        action: String,
        resource: String,
    },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("API error: {message}")]
    Api {
        message: String,
        code: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Resource already exists: {resource_type} '{name}'")]
    AlreadyExists {
        resource_type: String,
        name: String,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl CloudError {
    /// Get the exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            CloudError::Config { .. } => 1,
            CloudError::NotFound { .. } => 2,
            CloudError::PermissionDenied { .. } => 3,
            CloudError::InvalidArgument(_) => 4,
            CloudError::Api { .. } => 5,
            CloudError::Network(_) => 6,
            CloudError::Timeout(_) => 7,
            CloudError::AlreadyExists { .. } => 8,
            CloudError::Validation(_) => 9,
            CloudError::Internal(_) => 10,
        }
    }

    /// Get a suggestion for how to fix this error.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            CloudError::Config { .. } => {
                Some("Check your configuration file or run 'cloudctl config path'".to_string())
            }
            CloudError::NotFound { resource_type, .. } => {
                Some(format!("Use 'cloudctl {} list' to see available resources", resource_type))
            }
            CloudError::PermissionDenied { .. } => {
                Some("Check your credentials with 'cloudctl iam users get'".to_string())
            }
            CloudError::Network(_) => {
                Some("Check your network connection and proxy settings".to_string())
            }
            CloudError::Timeout(_) => {
                Some("Try again or increase timeout with --timeout flag".to_string())
            }
            _ => None,
        }
    }

    /// Should we show a help hint?
    pub fn show_help_hint(&self) -> bool {
        matches!(self, CloudError::InvalidArgument(_))
    }

    /// Create a config error.
    pub fn config(message: impl Into<String>) -> Self {
        CloudError::Config {
            message: message.into(),
            source: None,
        }
    }

    /// Create a not found error.
    pub fn not_found(resource_type: impl Into<String>, name: impl Into<String>) -> Self {
        CloudError::NotFound {
            resource_type: resource_type.into(),
            name: name.into(),
        }
    }

    /// Create an API error.
    pub fn api(message: impl Into<String>) -> Self {
        CloudError::Api {
            message: message.into(),
            code: None,
            source: None,
        }
    }
}

// Display is implemented by the #[derive(Error)] macro via #[error("...")] attributes
