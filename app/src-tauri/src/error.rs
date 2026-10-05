use std::sync::PoisonError;

use serde::Serialize;

macro_rules! from_error {
    ($ty:ty) => {
        impl From<$ty> for CommandError {
            fn from(value: $ty) -> Self {
                Self::from(value.to_string())
            }
        }
    };

    ($ty:ty $(, $generic:ident)*) => {
        impl< $($generic),* > From<$ty> for CommandError {
            fn from(value: $ty) -> Self {
                Self::from(value.to_string())
            }
        }
    };
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct CommandError {
    pub message: String,
}

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        Self::from(format!("{error:#}"))
    }
}

from_error!(PoisonError<T>, T);
from_error!(tauri_plugin_clipboard_manager::Error);

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for CommandError {
    fn from(message: &str) -> Self {
        Self::from(message.to_owned())
    }
}

pub(crate) fn cause_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(inner) = cause {
        let text = inner.to_string();
        if !message.contains(&text) {
            message.push_str(": ");
            message.push_str(&text);
        }
        cause = inner.source();
    }
    message
}

impl From<wf_core::CoreError> for CommandError {
    fn from(error: wf_core::CoreError) -> Self {
        Self::from(cause_chain(&error))
    }
}

impl From<wf_market::MarketError> for CommandError {
    fn from(error: wf_market::MarketError) -> Self {
        Self::from(cause_chain(&error))
    }
}

impl From<tauri::Error> for CommandError {
    fn from(error: tauri::Error) -> Self {
        Self::from(error.to_string())
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

pub type CommandResult<T> = std::result::Result<T, CommandError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anyhow_message() {
        let error = CommandError::from(anyhow::anyhow!("boom"));
        assert_eq!(error.message, "boom");
        assert_eq!(error.to_string(), "boom");
    }

    #[test]
    fn market_error_message() {
        let error = CommandError::from(wf_market::MarketError::Unauthorized);
        assert_eq!(error.message, "Unauthorized");
    }

    #[test]
    fn core_error_message() {
        let error = CommandError::from(wf_core::CoreError::SchemaVersion(7));
        assert_eq!(
            error.message,
            "Store schema version 7 is newer than this build understands"
        );
    }

    #[derive(Debug)]
    struct Outer(std::io::Error);

    impl std::fmt::Display for Outer {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("Reading the cache failed")
        }
    }

    impl std::error::Error for Outer {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn hidden_source_appended() {
        let inner = std::io::Error::other("disk full");
        assert_eq!(
            cause_chain(&Outer(inner)),
            "Reading the cache failed: disk full"
        );
    }

    #[test]
    fn displayed_source_not_repeated() {
        let error = wf_core::CoreError::Io {
            path: std::path::PathBuf::from("/data/merframe.sqlite"),
            source: std::io::Error::other("disk full"),
        };
        assert_eq!(
            cause_chain(&error),
            "Writing /data/merframe.sqlite failed: disk full"
        );
    }
}
