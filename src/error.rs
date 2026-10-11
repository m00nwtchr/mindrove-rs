#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MindRoveExitCode(i32);

impl MindRoveExitCode {
    pub const STATUS_OK: Self = Self(0);
    pub const PORT_ALREADY_OPEN_ERROR: Self = Self(1);
    pub const UNABLE_TO_OPEN_PORT_ERROR: Self = Self(2);
    pub const SET_PORT_ERROR: Self = Self(3);
    pub const BOARD_WRITE_ERROR: Self = Self(4);
    pub const INCOMMING_MSG_ERROR: Self = Self(5);
    pub const INITIAL_MSG_ERROR: Self = Self(6);
    pub const BOARD_NOT_READY_ERROR: Self = Self(7);
    pub const STREAM_ALREADY_RUN_ERROR: Self = Self(8);
    pub const INVALID_BUFFER_SIZE_ERROR: Self = Self(9);
    pub const STREAM_THREAD_ERROR: Self = Self(10);
    pub const STREAM_THREAD_IS_NOT_RUNNING: Self = Self(11);
    pub const EMPTY_BUFFER_ERROR: Self = Self(12);
    pub const INVALID_ARGUMENTS_ERROR: Self = Self(13);
    pub const UNSUPPORTED_BOARD_ERROR: Self = Self(14);
    pub const BOARD_NOT_CREATED_ERROR: Self = Self(15);
    pub const ANOTHER_BOARD_IS_CREATED_ERROR: Self = Self(16);
    pub const GENERAL_ERROR: Self = Self(17);
    pub const SYNC_TIMEOUT_ERROR: Self = Self(18);
    pub const JSON_NOT_FOUND_ERROR: Self = Self(19);
    pub const NO_SUCH_DATA_IN_JSON_ERROR: Self = Self(20);
    pub const CLASSIFIER_IS_NOT_PREPARED_ERROR: Self = Self(21);
    pub const ANOTHER_CLASSIFIER_IS_PREPARED_ERROR: Self = Self(22);
    pub const UNSUPPORTED_CLASSIFIER_AND_METRIC_COMBINATION_ERROR: Self = Self(23);

    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MindRove native error: {0:?}")]
    Native(MindRoveExitCode),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("integer or allocation size overflow")]
    IntegerOverflow,
    #[error("string contains an interior NUL")]
    InteriorNul,
    #[error("native string is not valid UTF-8")]
    InvalidUtf8,
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("native output length is invalid")]
    InvalidLength,
    #[error("native buffer contract failed: {0}")]
    BufferContract(String),
}

impl Error {
    pub fn from_status(raw: i32) -> Result<()> {
        if raw == MindRoveExitCode::STATUS_OK.as_raw() {
            Ok(())
        } else {
            Err(Self::Native(MindRoveExitCode::from_raw(raw)))
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
