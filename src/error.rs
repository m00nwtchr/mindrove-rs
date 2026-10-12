#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MindRove port is already open")]
    PortAlreadyOpen,
    #[error("MindRove was unable to open the port")]
    UnableToOpenPort,
    #[error("MindRove failed to set the port")]
    SetPort,
    #[error("MindRove failed to write to the board")]
    BoardWrite,
    #[error("MindRove received an incoming message error")]
    IncomingMessage,
    #[error("MindRove failed to receive the initial message")]
    InitialMessage,
    #[error("MindRove board is not ready")]
    BoardNotReady,
    #[error("MindRove stream is already running")]
    StreamAlreadyRunning,
    #[error("MindRove buffer size is invalid")]
    InvalidBufferSize,
    #[error("MindRove stream thread failed")]
    StreamThread,
    #[error("MindRove stream thread is not running")]
    StreamThreadNotRunning,
    #[error("MindRove buffer is empty")]
    EmptyBuffer,
    #[error("MindRove arguments are invalid")]
    InvalidArguments,
    #[error("MindRove board is unsupported")]
    UnsupportedBoard,
    #[error("MindRove board was not created")]
    BoardNotCreated,
    #[error("another MindRove board has been created")]
    AnotherBoardCreated,
    #[error("MindRove general error")]
    General,
    #[error("MindRove synchronization timed out")]
    SyncTimeout,
    #[error("MindRove JSON was not found")]
    JsonNotFound,
    #[error("MindRove data was not found in JSON")]
    NoSuchDataInJson,
    #[error("MindRove classifier is not prepared")]
    ClassifierNotPrepared,
    #[error("another MindRove classifier has been prepared")]
    AnotherClassifierPrepared,
    #[error("MindRove classifier and metric combination is unsupported")]
    UnsupportedClassifierMetricCombination,
    #[error("unknown MindRove native status: {0}")]
    UnknownNativeStatus(i32),
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
        match raw {
            0 => Ok(()),
            1 => Err(Self::PortAlreadyOpen),
            2 => Err(Self::UnableToOpenPort),
            3 => Err(Self::SetPort),
            4 => Err(Self::BoardWrite),
            5 => Err(Self::IncomingMessage),
            6 => Err(Self::InitialMessage),
            7 => Err(Self::BoardNotReady),
            8 => Err(Self::StreamAlreadyRunning),
            9 => Err(Self::InvalidBufferSize),
            10 => Err(Self::StreamThread),
            11 => Err(Self::StreamThreadNotRunning),
            12 => Err(Self::EmptyBuffer),
            13 => Err(Self::InvalidArguments),
            14 => Err(Self::UnsupportedBoard),
            15 => Err(Self::BoardNotCreated),
            16 => Err(Self::AnotherBoardCreated),
            17 => Err(Self::General),
            18 => Err(Self::SyncTimeout),
            19 => Err(Self::JsonNotFound),
            20 => Err(Self::NoSuchDataInJson),
            21 => Err(Self::ClassifierNotPrepared),
            22 => Err(Self::AnotherClassifierPrepared),
            23 => Err(Self::UnsupportedClassifierMetricCombination),
            _ => Err(Self::UnknownNativeStatus(raw)),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
