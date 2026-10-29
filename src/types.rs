use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! sdk_enum {
    ($name:ident { $($variant:ident = $value:expr),+ $(,)? }) => {
        #[allow(non_camel_case_types)]
        #[repr(i32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $( $variant = $value, )+
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_i32(*self as i32)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = i32::deserialize(deserializer)?;
                match value {
                    $( $value => Ok(Self::$variant), )+
                    _ => Err(serde::de::Error::custom(concat!(stringify!($name), " has an unknown value"))),
                }
            }
        }
    };
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardId(i32);

impl BoardId {
    pub const NO_BOARD: Self = Self(-100);
    pub const PLAYBACK_FILE_BOARD: Self = Self(-3);
    pub const STREAMING_BOARD: Self = Self(-2);
    pub const SYNTHETIC_BOARD: Self = Self(-1);
    pub const MINDROVE_WIFI_BOARD: Self = Self(0);
    pub const MINDROVE_SYNCBOX_BOARD: Self = Self(1);

    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

sdk_enum! { IpProtocolType {
    NO_IP_PROTOCOL = 0, UDP = 1, TCP = 2
} }
sdk_enum! { FilterType {
    BUTTERWORTH = 0, CHEBYSHEV_TYPE_1 = 1, BESSEL = 2,
    BUTTERWORTH_ZERO_PHASE = 3, CHEBYSHEV_TYPE_1_ZERO_PHASE = 4, BESSEL_ZERO_PHASE = 5
} }
sdk_enum! { AggOperation { MEAN = 0, MEDIAN = 1, EACH = 2 } }
sdk_enum! { WindowOperation {
    NO_WINDOW = 0, HANNING = 1, HAMMING = 2, BLACKMAN_HARRIS = 3
} }
sdk_enum! { DetrendOperation { NO_DETREND = 0, CONSTANT = 1, LINEAR = 2 } }
sdk_enum! { MindRoveMetric { MINDFULNESS = 0, RESTFULNESS = 1, USER_DEFINED = 2 } }
sdk_enum! { MindRoveClassifier {
    DEFAULT_CLASSIFIER = 0, DYN_LIB_CLASSIFIER = 1, ONNX_CLASSIFIER = 2
} }
sdk_enum! { MindRovePreset {
    DEFAULT_PRESET = 0, AUXILIARY_PRESET = 1, ANCILLARY_PRESET = 2
} }
sdk_enum! { NoiseType { FIFTY = 0, SIXTY = 1, FIFTY_AND_SIXTY = 2 } }
sdk_enum! { WaveletDenoisingType { VISUSHRINK = 0, SURESHRINK = 1 } }
sdk_enum! { ThresholdType { SOFT = 0, HARD = 1 } }
sdk_enum! { WaveletExtensionType { SYMMETRIC = 0, PERIODIC = 1 } }
sdk_enum! { NoiseEstimationLevelType { FIRST_LEVEL = 0, ALL_LEVELS = 1 } }
sdk_enum! { WaveletType {
    HAAR = 0, DB1 = 1, DB2 = 2, DB3 = 3, DB4 = 4, DB5 = 5, DB6 = 6,
    DB7 = 7, DB8 = 8, DB9 = 9, DB10 = 10, DB11 = 11, DB12 = 12,
    DB13 = 13, DB14 = 14, DB15 = 15, BIOR1_1 = 16, BIOR1_3 = 17,
    BIOR1_5 = 18, BIOR2_2 = 19, BIOR2_4 = 20, BIOR2_6 = 21,
    BIOR2_8 = 22, BIOR3_1 = 23, BIOR3_3 = 24, BIOR3_5 = 25,
    BIOR3_7 = 26, BIOR3_9 = 27, BIOR4_4 = 28, BIOR5_5 = 29,
    BIOR6_8 = 30, COIF1 = 31, COIF2 = 32, COIF3 = 33, COIF4 = 34,
    COIF5 = 35, SYM2 = 36, SYM3 = 37, SYM4 = 38, SYM5 = 39,
    SYM6 = 40, SYM7 = 41, SYM8 = 42, SYM9 = 43, SYM10 = 44
} }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindRoveInputParams {
    pub serial_port: String,
    pub mac_address: String,
    pub ip_address: String,
    pub ip_address_aux: String,
    pub ip_address_anc: String,
    pub ip_port: i32,
    pub ip_port_aux: i32,
    pub ip_port_anc: i32,
    pub ip_protocol: IpProtocolType,
    pub other_info: String,
    pub timeout: i32,
    pub serial_number: String,
    pub file: String,
    pub file_aux: String,
    pub file_anc: String,
    pub master_board: BoardId,
}

impl Default for MindRoveInputParams {
    fn default() -> Self {
        Self {
            serial_port: String::new(),
            mac_address: String::new(),
            ip_address: String::new(),
            ip_address_aux: String::new(),
            ip_address_anc: String::new(),
            ip_port: -1,
            ip_port_aux: 0,
            ip_port_anc: 0,
            ip_protocol: IpProtocolType::NO_IP_PROTOCOL,
            other_info: String::new(),
            timeout: 0,
            serial_number: String::new(),
            file: String::new(),
            file_aux: String::new(),
            file_anc: String::new(),
            master_board: BoardId::NO_BOARD,
        }
    }
}

impl MindRoveInputParams {
    pub fn to_json(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindRoveModelParams {
    pub metric: MindRoveMetric,
    pub classifier: MindRoveClassifier,
    pub file: String,
    pub other_info: String,
    pub output_name: String,
    pub max_array_size: i32,
}

impl Default for MindRoveModelParams {
    fn default() -> Self {
        Self {
            metric: MindRoveMetric::MINDFULNESS,
            classifier: MindRoveClassifier::DEFAULT_CLASSIFIER,
            file: String::new(),
            other_info: String::new(),
            output_name: String::new(),
            max_array_size: 8192,
        }
    }
}

impl MindRoveModelParams {
    pub fn to_json(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
