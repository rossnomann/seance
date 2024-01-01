use std::{
    error::Error,
    ffi::OsString,
    fmt,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    num::ParseIntError,
    path::{Path, PathBuf},
    string::FromUtf8Error,
    time::SystemTimeError,
};

use tokio::fs;

use crate::{backend::SessionBackend, utils::now};

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

impl SessionBackend for FilesystemBackend {
    type Error = FilesystemBackendError;

    async fn get_sessions(&mut self) -> Result<Vec<String>, Self::Error> {
        let mut result = Vec::new();
        let mut entries = match fs::read_dir(&self.root).await {
            Ok(entries) => entries,
            Err(error) => {
                return match error.kind() {
                    IoErrorKind::NotFound => Ok(result),
                    _ => Err(FilesystemBackendError::GetSessions(error)),
                }
            }
        };
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(FilesystemBackendError::GetSessions)?
        {
            let file_name = entry.file_name();
            result.push(match file_name.into_string() {
                Ok(file_name) => file_name,
                Err(file_name) => return Err(FilesystemBackendError::GetSessionName(file_name)),
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
            let mut entries = fs::read_dir(&session_root)
                .await
                .map_err(FilesystemBackendError::RemoveSession)?;
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(FilesystemBackendError::RemoveSession)?
            {
                fs::remove_file(entry.path())
                    .await
                    .map_err(FilesystemBackendError::RemoveSession)?;
            }
            fs::remove_dir(session_root)
                .await
                .map_err(FilesystemBackendError::RemoveSession)?;
        }
        Ok(())
    }

    async fn read_value(&mut self, session_id: &str, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if is_session_root_exists(&session_root).await? {
            match fs::read(session_root.join(key)).await {
                Ok(data) => Ok(Some(data)),
                Err(error) => match error.kind() {
                    IoErrorKind::NotFound => Ok(None),
                    _ => Err(FilesystemBackendError::ReadValue(error)),
                },
            }
        } else {
            Ok(None)
        }
    }

    async fn write_value(&mut self, session_id: &str, key: &str, value: &[u8]) -> Result<(), Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if !is_session_root_exists(&session_root).await? {
            fs::create_dir_all(&session_root)
                .await
                .map_err(FilesystemBackendError::WriteValue)?;
            TimeMarker::create(&session_root).await?;
        }
        fs::write(session_root.join(key), value)
            .await
            .map_err(FilesystemBackendError::WriteValue)?;
        Ok(())
    }

    async fn remove_value(&mut self, session_id: &str, key: &str) -> Result<(), Self::Error> {
        let session_root = self.root.clone().join(session_id);
        if is_session_root_exists(&session_root).await? {
            if let Err(error) = fs::remove_file(session_root.join(key)).await {
                return match error.kind() {
                    IoErrorKind::NotFound => Ok(()),
                    _ => Err(FilesystemBackendError::RemoveValue(error)),
                };
            }
        }
        Ok(())
    }
}

const TIME_MARKER: &str = ".__created";

struct TimeMarker;

impl TimeMarker {
    async fn create<P: AsRef<Path>>(root: P) -> Result<(), FilesystemBackendError> {
        let timestamp = now().map_err(FilesystemBackendError::TimeMarkerInitValue)?;
        let timestamp = format!("{timestamp}");
        fs::write(root.as_ref().join(TIME_MARKER), timestamp)
            .await
            .map_err(FilesystemBackendError::TimeMarkerCreate)?;
        Ok(())
    }

    async fn read<P: AsRef<Path>>(root: P) -> Result<u64, FilesystemBackendError> {
        let data = fs::read(root.as_ref().join(TIME_MARKER))
            .await
            .map_err(FilesystemBackendError::TimeMarkerRead)?;
        let data = String::from_utf8(data).map_err(FilesystemBackendError::TimeMarkerGetString)?;
        let timestamp = data
            .parse::<u64>()
            .map_err(FilesystemBackendError::TimeMarkerParseValue)?;
        Ok(timestamp)
    }
}

async fn is_session_root_exists<P: AsRef<Path>>(path: P) -> Result<bool, FilesystemBackendError> {
    let path = path.as_ref();
    match fs::metadata(&path).await {
        Ok(meta) => {
            if meta.is_dir() {
                Ok(true)
            } else {
                Err(FilesystemBackendError::SessionRootOccupied(path.to_path_buf()))
            }
        }
        Err(error) => match error.kind() {
            IoErrorKind::NotFound => Ok(false),
            _ => Err(FilesystemBackendError::SessionRootMetadata(error)),
        },
    }
}

/// An error occurred in filesystem backend
#[derive(Debug)]
pub enum FilesystemBackendError {
    /// Failed to get sessions list
    // #[snafu(display("failed to get sessions list: {}", source))]
    GetSessions(IoError),
    /// Failed to convert session directory name to string
    // #[snafu(display("failed to get session name: {:?}", name))]
    GetSessionName(OsString),
    /// Failed to read a value
    // #[snafu(display("failed to read a value: {}", source))]
    ReadValue(IoError),
    /// Failed to remove session
    // #[snafu(display("failed to remove session: {}", source))]
    RemoveSession(IoError),
    /// Failed to remove a value
    // #[snafu(display("failed to remove a value: {}", source))]
    RemoveValue(IoError),
    /// Failed to get session root metadata
    // #[snafu(display("failed to get session root metadata: {}", source))]
    SessionRootMetadata(IoError),
    /// Session directory is occupied by a file
    // #[snafu(display("session root '{}' is occupied", path.display()))]
    SessionRootOccupied(PathBuf),
    /// Failed to create time marker for a session
    // #[snafu(display("failed to create time marker: {}", source))]
    TimeMarkerCreate(IoError),
    /// Failed to get current time for time marker
    // #[snafu(display("failed to initialize value for time marker: {}", source))]
    TimeMarkerInitValue(SystemTimeError),
    /// Failed to read data from time marker
    // #[snafu(display("time marker contains non UTF-8 string: {}", source))]
    TimeMarkerGetString(FromUtf8Error),
    /// Failed to parse value for a time marker
    // #[snafu(display("failed to parse time marker value: {}", source))]
    TimeMarkerParseValue(ParseIntError),
    /// Failed to read time marker data from a file
    // #[snafu(display("failed to read time marker data: {}", source))]
    TimeMarkerRead(IoError),
    /// Failed to write a value
    // #[snafu(display("failed to write a value: {}", source))]
    WriteValue(IoError),
}

impl fmt::Display for FilesystemBackendError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::FilesystemBackendError::*;
        match self {
            GetSessions(err) => write!(out, "failed to get sessions list: {err}"),
            GetSessionName(name) => write!(out, "failed to get session name: {name:?}"),
            ReadValue(err) => write!(out, "failed to read a value: {err}"),
            RemoveSession(err) => write!(out, "failed to remove session: {err}"),
            RemoveValue(err) => write!(out, "failed to remove a value: {err}"),
            SessionRootMetadata(err) => {
                write!(out, "failed to get session root metadata: {err}")
            }
            SessionRootOccupied(path) => {
                write!(out, "session root '{}' is occupied", path.display())
            }
            TimeMarkerCreate(err) => write!(out, "failed to create time marker: {err}"),
            TimeMarkerInitValue(err) => {
                write!(out, "failed to initialize value for time marker: {err}")
            }
            TimeMarkerGetString(err) => {
                write!(out, "time marker contains non UTF-8 string: {err}")
            }
            TimeMarkerParseValue(err) => {
                write!(out, "failed to parse time marker value: {err}")
            }
            TimeMarkerRead(err) => write!(out, "failed to read time marker data: {err}"),
            WriteValue(err) => write!(out, "failed to write a value: {err}"),
        }
    }
}

impl Error for FilesystemBackendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::FilesystemBackendError::*;
        Some(match self {
            GetSessions(err) => err,
            GetSessionName(_) => return None,
            ReadValue(err) => err,
            RemoveSession(err) => err,
            RemoveValue(err) => err,
            SessionRootMetadata(err) => err,
            SessionRootOccupied(_) => return None,
            TimeMarkerCreate(err) => err,
            TimeMarkerInitValue(err) => err,
            TimeMarkerGetString(err) => err,
            TimeMarkerParseValue(err) => err,
            TimeMarkerRead(err) => err,
            WriteValue(err) => err,
        })
    }
}
