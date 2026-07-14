// SPDX-License-Identifier: Apache-2.0

/// Clipboard operation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    /// Failed to open the clipboard.
    OpenFailed,

    /// The requested clipboard format is unavailable.
    FormatUnavailable,

    /// Failed to read clipboard contents.
    ReadFailed,

    /// Failed to write clipboard contents.
    WriteFailed,

    /// Failed to allocate memory.
    AllocationFailed,

    /// Failed to lock allocated memory.
    LockFailed,

    /// Clipboard access is not supported on this platform.
    Unsupported,

    /// Permission to access the clipboard was denied.
    PermissionDenied,

    /// Platform-specific error.
    PlatformError(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenFailed => write!(f, "failed to open clipboard"),
            Self::FormatUnavailable => write!(f, "clipboard format unavailable"),
            Self::ReadFailed => write!(f, "failed to read clipboard"),
            Self::WriteFailed => write!(f, "failed to write clipboard"),
            Self::AllocationFailed => write!(f, "failed to allocate memory"),
            Self::LockFailed => write!(f, "failed to lock memory"),
            Self::Unsupported => write!(f, "clipboard is unsupported on this platform"),
            Self::PermissionDenied => write!(f, "clipboard permission denied"),
            Self::PlatformError(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ClipboardError {}
