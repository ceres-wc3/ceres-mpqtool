use std::io::Error as IoError;
use std::path::PathBuf;

use err_derive::Error;

use ceres_mpq::Error as MpqError;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error(display = "Failed to open file [{}]: {}", path, cause)]
    FileOpenError {
        path: String,
        cause: IoError
    },
    #[error(display = "Error while opening the MPQ archive: {}", cause)]
    MpqOpenError {
        cause: MpqError
    },
    #[error(display = "Error while reading file from MPQ archive: {}", cause)]
    MpqReadFileError {
        cause: MpqError
    },
    #[error(display = "Listfile not found in archive")]
    ListfileNotFound,
    #[error(display = "Could not create output directory [{:?}]: {}", path, cause)]
    OutDirCreationError {
        cause: IoError,
        path: PathBuf
    },
}
