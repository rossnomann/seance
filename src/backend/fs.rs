use crate::{backend::SessionBackend, utils::now};
use async_trait::async_trait;
use snafu::{ResultExt, Snafu};
use std::{
    ffi::OsString,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    num::ParseIntError,
    path::{Path, PathBuf},
    string::FromUtf8Error,
    time::SystemTimeError,
};
use tokio::fs;

/// Filesystem session backend
#[derive(Clone)]
pub struct FilesystemBackend {
    root: PathBuf,
}

impl FilesystemBackend {
    /// Creates a new backend
    ///
    /// # Arguments
    ///
    /// * root - Path to sessions directory
    ///
    /// Note that you MUST create `root` directory before using this backend
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }
}

#[async_trait]
impl SessionBackend for FilesystemBackend {
    type Error = FilesystemError;

    async fn get_sessions(&mut self) -> Result<Vec<String>, Self::Error> {
        let mut result = Vec::new();
        let mut entries = match fs::read_dir(&self.root).await {
            Ok(entries) => entries,
            Err(error) => match error.kind() {
                IoErrorKind::NotFound => return Ok(result),
                _ => return Err(error).context(GetSessions),
            },
        };
        while let Some(entry) = entries.next_entry().await.context(GetSessions)? {
            let file_name = entry.file_name();
            result.push(match file_name.into_string() {
                Ok(file_name) => file_name,
                Err(file_name) => return GetSessionName { name: file_name }.fail(),
            })
        }
        Ok(result)
    }

    async fn get_session_age(&mut self, session_id: &str) -> Result<Option<u64>, Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if is_session_root_exists(&session_root).await? {
            Ok(Some(TimeMarker::read(session_root).await?))
        } else {
            Ok(None)
        }
    }

    async fn remove_session(&mut self, session_id: &str) -> Result<(), Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if is_session_root_exists(&session_root).await? {
            let mut entries = fs::read_dir(&session_root).await.context(RemoveSession)?;
            while let Some(entry) = entries.next_entry().await.context(RemoveSession)? {
                fs::remove_file(entry.path()).await.context(RemoveSession)?;
            }
            fs::remove_dir(session_root).await.context(RemoveSession)?;
        }
        Ok(())
    }

    async fn read_value(
        &mut self,
        session_id: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if is_session_root_exists(&session_root).await? {
            match fs::read(session_root.join(key)).await {
                Ok(data) => Ok(Some(data)),
                Err(error) => match error.kind() {
                    IoErrorKind::NotFound => Ok(None),
                    _ => Err(error).context(ReadValue),
                },
            }
        } else {
            Ok(None)
        }
    }

    async fn write_value(
        &mut self,
        session_id: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if !is_session_root_exists(&session_root).await? {
            fs::create_dir_all(&session_root)
                .await
                .context(WriteValue)?;
            TimeMarker::create(&session_root).await?;
        }
        fs::write(session_root.join(key), value)
            .await
            .context(WriteValue)?;
        Ok(())
    }

    async fn remove_value(&mut self, session_id: &str, key: &str) -> Result<(), Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if is_session_root_exists(&session_root).await? {
            if let Err(error) = fs::remove_file(session_root.join(key)).await {
                return match error.kind() {
                    IoErrorKind::NotFound => Ok(()),
                    _ => Err(error).context(RemoveValue),
                };
            }
        }
        Ok(())
    }
}

const TIME_MARKER: &str = ".__created";

struct TimeMarker;

impl TimeMarker {
    async fn create<P: AsRef<Path>>(root: P) -> Result<(), FilesystemError> {
        let timestamp = now().context(TimeMarkerInitValue)?;
        let timestamp = format!("{}", timestamp);
        fs::write(root.as_ref().join(TIME_MARKER), timestamp)
            .await
            .context(TimeMarkerCreate)?;
        Ok(())
    }

    async fn read<P: AsRef<Path>>(root: P) -> Result<u64, FilesystemError> {
        let data = fs::read(root.as_ref().join(TIME_MARKER))
            .await
            .context(TimeMarkerRead)?;
        let data = String::from_utf8(data).context(TimeMarkerGetString)?;
        let timestamp = data.parse::<u64>().context(TimeMarkerParseValue)?;
        Ok(timestamp)
    }
}

/// An error occurred in filesystem backend
#[derive(Debug, Snafu)]
pub enum FilesystemError {
    /// Failed to get sessions list
    #[snafu(display("failed to get sessions list: {}", source))]
    GetSessions {
        /// Source error
        source: IoError,
    },

    /// Failed to convert session directory name to string
    #[snafu(display("failed to get session name: {:?}", name))]
    GetSessionName {
        /// Session directory name
        name: OsString,
    },

    /// Failed to read a value
    #[snafu(display("failed to read a value: {}", source))]
    ReadValue {
        /// Source error
        source: IoError,
    },

    /// Failed to remove session
    #[snafu(display("failed to remove session: {}", source))]
    RemoveSession {
        /// Source error
        source: IoError,
    },

    /// Failed to remove a value
    #[snafu(display("failed to remove a value: {}", source))]
    RemoveValue {
        /// Source error
        source: IoError,
    },

    /// Failed to get session root metadata
    #[snafu(display("failed to get session root metadata: {}", source))]
    SessionRootMetadata {
        /// Source error
        source: IoError,
    },

    /// Session directory is occupied by a file
    #[snafu(display("session root '{}' is occupied", path.display()))]
    SessionRootOccupied {
        /// Path to a file
        path: PathBuf,
    },

    /// Failed to create time marker for a session
    #[snafu(display("failed to create time marker: {}", source))]
    TimeMarkerCreate {
        /// Source error
        source: IoError,
    },

    /// Failed to get current time for time marker
    #[snafu(display("failed to initialize value for time marker: {}", source))]
    TimeMarkerInitValue {
        /// Source error
        source: SystemTimeError,
    },

    /// Failed to read data from time marker
    #[snafu(display("time marker contains non UTF-8 string: {}", source))]
    TimeMarkerGetString {
        /// Source error
        source: FromUtf8Error,
    },

    /// Failed to parse value for a time marker
    #[snafu(display("failed to parse time marker value: {}", source))]
    TimeMarkerParseValue {
        /// Source error
        source: ParseIntError,
    },

    /// Failed to read time marker data from a file
    #[snafu(display("failed to read time marker data: {}", source))]
    TimeMarkerRead {
        /// Source error
        source: IoError,
    },

    /// Failed to write a value
    #[snafu(display("failed to write a value: {}", source))]
    WriteValue {
        /// Source error
        source: IoError,
    },
}

async fn is_session_root_exists<P: AsRef<Path>>(path: P) -> Result<bool, FilesystemError> {
    let path = path.as_ref();
    match fs::metadata(&path).await {
        Ok(meta) => {
            if meta.is_dir() {
                Ok(true)
            } else {
                SessionRootOccupied {
                    path: path.to_path_buf(),
                }
                .fail()
            }
        }
        Err(error) => match error.kind() {
            IoErrorKind::NotFound => Ok(false),
            _ => Err(error).context(SessionRootMetadata),
        },
    }
}
