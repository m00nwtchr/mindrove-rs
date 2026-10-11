use std::ffi::{CString, c_char};

use ndarray::Array2;
use serde_json::Value;

use crate::internal;
use crate::{BoardId, Error, LogLevel, MindRoveInputParams, MindRovePreset, Result};

pub struct BoardShim {
    board_id: BoardId,
    params_json: CString,
    prepared: bool,
    streaming: bool,
}

macro_rules! scalar_metadata_methods {
    ($name:ident, $with_name:ident, $kind:expr, $output:ty) => {
        pub fn $name(board_id: BoardId) -> Result<$output> {
            Self::$with_name(board_id, MindRovePreset::DEFAULT_PRESET)
        }

        pub fn $with_name(board_id: BoardId, preset: MindRovePreset) -> Result<$output> {
            scalar_index_metadata($kind, board_id, preset)
        }
    };
}

macro_rules! channel_methods {
    ($name:ident, $with_name:ident, $kind:expr) => {
        pub fn $name(board_id: BoardId) -> Result<Vec<usize>> {
            Self::$with_name(board_id, MindRovePreset::DEFAULT_PRESET)
        }

        pub fn $with_name(board_id: BoardId, preset: MindRovePreset) -> Result<Vec<usize>> {
            channel_metadata($kind, board_id, preset)
        }
    };
}

impl BoardShim {
    pub fn new(board_id: BoardId, params: MindRoveInputParams) -> Result<Self> {
        let params_json = internal::to_json_cstring(&params)?;
        Ok(Self {
            board_id,
            params_json,
            prepared: false,
            streaming: false,
        })
    }

    pub fn get_board_descr(board_id: BoardId) -> Result<Value> {
        Self::get_board_descr_with_preset(board_id, MindRovePreset::DEFAULT_PRESET)
    }

    pub fn get_board_descr_with_preset(board_id: BoardId, preset: MindRovePreset) -> Result<Value> {
        let text = read_string_metadata(|buffer, len| {
            internal::board_metadata(0, board_id.as_raw(), preset as i32, buffer, len)
        })?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn get_sampling_rate(board_id: BoardId) -> Result<i32> {
        Self::get_sampling_rate_with_preset(board_id, MindRovePreset::DEFAULT_PRESET)
    }

    pub fn get_sampling_rate_with_preset(board_id: BoardId, preset: MindRovePreset) -> Result<i32> {
        scalar_metadata(0, board_id, preset)
    }

    scalar_metadata_methods!(
        get_package_num_channel,
        get_package_num_channel_with_preset,
        1,
        usize
    );
    scalar_metadata_methods!(
        get_timestamp_channel,
        get_timestamp_channel_with_preset,
        2,
        usize
    );
    scalar_metadata_methods!(get_marker_channel, get_marker_channel_with_preset, 3, usize);
    scalar_metadata_methods!(
        get_battery_channel,
        get_battery_channel_with_preset,
        4,
        usize
    );
    scalar_metadata_methods!(get_num_rows, get_num_rows_with_preset, 5, usize);

    pub fn get_device_name(board_id: BoardId) -> Result<String> {
        Self::get_device_name_with_preset(board_id, MindRovePreset::DEFAULT_PRESET)
    }

    pub fn get_device_name_with_preset(
        board_id: BoardId,
        preset: MindRovePreset,
    ) -> Result<String> {
        read_string_metadata(|buffer, len| {
            internal::board_metadata(2, board_id.as_raw(), preset as i32, buffer, len)
        })
    }

    pub fn get_eeg_names(board_id: BoardId) -> Result<Vec<String>> {
        Self::get_eeg_names_with_preset(board_id, MindRovePreset::DEFAULT_PRESET)
    }

    pub fn get_eeg_names_with_preset(
        board_id: BoardId,
        preset: MindRovePreset,
    ) -> Result<Vec<String>> {
        let text = read_string_metadata(|buffer, len| {
            internal::board_metadata(1, board_id.as_raw(), preset as i32, buffer, len)
        })?;
        Ok(parse_eeg_names(&text))
    }

    channel_methods!(get_exg_channels, get_exg_channels_with_preset, 0);
    channel_methods!(get_eeg_channels, get_eeg_channels_with_preset, 1);
    channel_methods!(get_emg_channels, get_emg_channels_with_preset, 2);
    channel_methods!(get_ecg_channels, get_ecg_channels_with_preset, 3);
    channel_methods!(get_eog_channels, get_eog_channels_with_preset, 4);
    channel_methods!(get_ppg_channels, get_ppg_channels_with_preset, 5);
    channel_methods!(get_ppg_raw_channels, get_ppg_raw_channels_with_preset, 6);
    channel_methods!(get_eda_channels, get_eda_channels_with_preset, 7);
    channel_methods!(get_accel_channels, get_accel_channels_with_preset, 8);
    channel_methods!(get_rotation_channels, get_rotation_channels_with_preset, 9);
    channel_methods!(get_analog_channels, get_analog_channels_with_preset, 10);
    channel_methods!(get_gyro_channels, get_gyro_channels_with_preset, 11);
    channel_methods!(get_other_channels, get_other_channels_with_preset, 12);
    channel_methods!(
        get_temperature_channels,
        get_temperature_channels_with_preset,
        13
    );
    channel_methods!(
        get_resistance_channels,
        get_resistance_channels_with_preset,
        14
    );
    channel_methods!(
        get_magnetometer_channels,
        get_magnetometer_channels_with_preset,
        15
    );

    pub fn get_board_presets(board_id: BoardId) -> Result<Vec<i32>> {
        internal::read_i32_vec(|values, len| {
            internal::board_presets(board_id.as_raw(), values, len)
        })
    }

    pub fn release_all_sessions() -> Result<()> {
        internal::check_status(internal::release_all_sessions())
    }

    pub fn get_version() -> Result<String> {
        let max_len = 256_i32;
        let mut buffer = vec![0_u8; max_len as usize];
        let mut len = max_len;
        internal::check_status(internal::board_version(
            buffer.as_mut_ptr().cast(),
            &mut len,
            max_len,
        ))?;
        let len = usize::try_from(len).map_err(|_| Error::InvalidLength)?;
        if len > buffer.len() {
            return Err(Error::InvalidLength);
        }
        String::from_utf8(buffer[..len].to_vec()).map_err(|_| Error::InvalidUtf8)
    }

    pub fn disable_board_logger() -> Result<()> {
        set_log_level(LogLevel::LEVEL_OFF)
    }
    pub fn enable_board_logger() -> Result<()> {
        set_log_level(LogLevel::LEVEL_INFO)
    }
    pub fn enable_dev_board_logger() -> Result<()> {
        set_log_level(LogLevel::LEVEL_TRACE)
    }

    pub fn set_log_file(path: &str) -> Result<()> {
        let path = internal::to_cstring(path)?;
        internal::check_status(internal::set_log_file(path.as_ptr()))
    }

    pub fn set_log_level(level: LogLevel) -> Result<()> {
        set_log_level(level)
    }

    pub fn log_message(level: LogLevel, message: &str) -> Result<()> {
        let mut message = internal::to_cstring(message)?.into_bytes_with_nul();
        internal::check_status(internal::logger_call(
            1,
            level as i32,
            message.as_mut_ptr().cast(),
        ))
    }

    pub fn prepare_session(&mut self) -> Result<()> {
        internal::check_status(internal::prepare_session(
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        self.prepared = true;
        Ok(())
    }

    pub fn start_stream(&mut self) -> Result<()> {
        self.start_stream_with_options(450_000, "")
    }

    pub fn start_stream_with_options(
        &mut self,
        buffer_size: usize,
        streamer_params: &str,
    ) -> Result<()> {
        let buffer_size = internal::checked_native_capacity(buffer_size)?;
        let streamer_params = internal::to_cstring(streamer_params)?;
        internal::check_status(internal::start_stream(
            buffer_size,
            streamer_params.as_ptr(),
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        self.streaming = true;
        Ok(())
    }

    pub fn stop_stream(&mut self) -> Result<()> {
        internal::check_status(internal::stop_stream(
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        self.streaming = false;
        Ok(())
    }

    pub fn is_prepared(&self) -> Result<bool> {
        let mut prepared = 0;
        internal::check_status(internal::is_prepared(
            &mut prepared,
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        Ok(prepared != 0)
    }

    pub fn release_session(&mut self) -> Result<()> {
        internal::check_status(internal::release_session(
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        self.prepared = false;
        self.streaming = false;
        Ok(())
    }

    pub fn add_streamer(&self, streamer_params: &str) -> Result<()> {
        self.add_streamer_with_preset(streamer_params, MindRovePreset::DEFAULT_PRESET)
    }

    pub fn add_streamer_with_preset(
        &self,
        streamer_params: &str,
        preset: MindRovePreset,
    ) -> Result<()> {
        let streamer_params = internal::to_cstring(streamer_params)?;
        internal::check_status(internal::add_streamer(
            streamer_params.as_ptr(),
            preset as i32,
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))
    }

    pub fn delete_streamer(&self, streamer_params: &str) -> Result<()> {
        self.delete_streamer_with_preset(streamer_params, MindRovePreset::DEFAULT_PRESET)
    }

    pub fn delete_streamer_with_preset(
        &self,
        streamer_params: &str,
        preset: MindRovePreset,
    ) -> Result<()> {
        let streamer_params = internal::to_cstring(streamer_params)?;
        internal::check_status(internal::delete_streamer(
            streamer_params.as_ptr(),
            preset as i32,
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))
    }

    pub fn insert_marker(&self, value: f64) -> Result<()> {
        internal::check_status(internal::insert_marker(
            value,
            MindRovePreset::DEFAULT_PRESET as i32,
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))
    }

    pub fn config_board(&self, config: &str) -> Result<String> {
        let config = internal::to_cstring(config)?;
        internal::read_string(|response, response_len| {
            internal::config_board(
                config.as_ptr(),
                response,
                response_len,
                self.board_id.as_raw(),
                self.params_json.as_ptr(),
            )
        })
    }

    pub fn get_board_id(&self) -> Result<BoardId> {
        Ok(self.board_id)
    }

    pub fn get_board_data_count(&self) -> Result<usize> {
        let mut count = 0;
        internal::check_status(internal::get_board_data_count(
            MindRovePreset::DEFAULT_PRESET as i32,
            &mut count,
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        usize::try_from(count).map_err(|_| Error::InvalidLength)
    }

    pub fn get_current_board_data(&self, requested_samples: usize) -> Result<Array2<f64>> {
        let rows = Self::get_num_rows(self.board_id)?;
        let element_count = internal::checked_element_count(rows, requested_samples)?;
        let native_samples = internal::checked_native_capacity(requested_samples)?;
        let mut data = vec![0.0; element_count];
        let mut returned_samples = 0;
        internal::check_status(internal::get_current_board_data(
            native_samples,
            MindRovePreset::DEFAULT_PRESET as i32,
            data.as_mut_ptr(),
            &mut returned_samples,
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        let returned_samples = validate_returned_samples(returned_samples, requested_samples)?;
        let used = internal::checked_element_count(rows, returned_samples)?;
        internal::array2_from_native(data[..used].to_vec(), rows, returned_samples)
    }

    pub fn get_board_data(&self) -> Result<Array2<f64>> {
        let count = self.get_board_data_count()?;
        self.read_board_data(count)
    }

    pub fn get_board_data_n(&self, requested_count: usize) -> Result<Array2<f64>> {
        let available = self.get_board_data_count()?;
        self.read_board_data(clamp_data_count(requested_count, available))
    }

    fn read_board_data(&self, count: usize) -> Result<Array2<f64>> {
        let rows = Self::get_num_rows(self.board_id)?;
        let element_count = internal::checked_element_count(rows, count)?;
        let native_count = internal::checked_native_capacity(count)?;
        let mut data = vec![0.0; element_count];
        internal::check_status(internal::get_board_data(
            native_count,
            MindRovePreset::DEFAULT_PRESET as i32,
            data.as_mut_ptr(),
            self.board_id.as_raw(),
            self.params_json.as_ptr(),
        ))?;
        internal::array2_from_native(data, rows, count)
    }
}

impl Drop for BoardShim {
    fn drop(&mut self) {
        if self.streaming {
            let _ = self.stop_stream();
        }
        if self.prepared {
            let _ = self.release_session();
        }
    }
}

fn set_log_level(level: LogLevel) -> Result<()> {
    internal::check_status(internal::logger_call(0, level as i32, std::ptr::null_mut()))
}

fn validate_returned_samples(returned_samples: i32, requested_samples: usize) -> Result<usize> {
    let returned_samples = usize::try_from(returned_samples).map_err(|_| Error::InvalidLength)?;
    if returned_samples > requested_samples {
        return Err(Error::InvalidLength);
    }
    Ok(returned_samples)
}

fn clamp_data_count(requested_count: usize, available_count: usize) -> usize {
    requested_count.min(available_count)
}

fn scalar_metadata(kind: u8, board_id: BoardId, preset: MindRovePreset) -> Result<i32> {
    read_scalar(|| {
        let mut value = 0;
        (
            internal::board_scalar(kind, board_id.as_raw(), preset as i32, &mut value),
            value,
        )
    })
}

fn scalar_index_metadata(kind: u8, board_id: BoardId, preset: MindRovePreset) -> Result<usize> {
    usize::try_from(scalar_metadata(kind, board_id, preset)?).map_err(|_| Error::InvalidLength)
}

fn channel_metadata(kind: u8, board_id: BoardId, preset: MindRovePreset) -> Result<Vec<usize>> {
    let values = internal::read_i32_vec(|values, len| {
        internal::board_channels(kind, board_id.as_raw(), preset as i32, values, len)
    })?;
    read_channel_indices(|| (0, values))
}

fn read_scalar<F>(call: F) -> Result<i32>
where
    F: FnOnce() -> (i32, i32),
{
    let (status, value) = call();
    internal::check_status(status)?;
    Ok(value)
}

fn read_string_metadata<F>(call: F) -> Result<String>
where
    F: Fn(*mut c_char, *mut i32) -> i32,
{
    internal::read_string(call)
}

fn read_channel_indices<F>(call: F) -> Result<Vec<usize>>
where
    F: FnOnce() -> (i32, Vec<i32>),
{
    let (status, values) = call();
    internal::check_status(status)?;
    values
        .into_iter()
        .map(|value| usize::try_from(value).map_err(|_| Error::InvalidLength))
        .collect()
}

fn parse_eeg_names(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn scalar_metadata_requires_success_status() {
        assert_eq!(read_scalar(|| (0, 128)).unwrap(), 128);
        assert!(matches!(read_scalar(|| (17, 128)), Err(Error::Native(_))));
    }

    #[test]
    fn string_metadata_rejects_invalid_utf8() {
        let result = read_string_metadata(|buffer, length| {
            if buffer.is_null() {
                unsafe {
                    *length = 1;
                }
            } else {
                unsafe { *buffer.cast::<u8>() = 0xff };
                unsafe {
                    *length = 1;
                }
            }
            0
        });

        assert!(matches!(result, Err(Error::InvalidUtf8)));
    }

    #[test]
    fn channel_metadata_rejects_negative_indices() {
        assert!(matches!(
            read_channel_indices(|| (0, vec![-1])),
            Err(Error::InvalidLength)
        ));
    }

    #[test]
    fn eeg_names_parser_splits_native_comma_separated_names() {
        assert_eq!(
            parse_eeg_names("Fz,C3,Cz,C4,Pz,PO7,Oz,PO8,F5,F7,F3,F1,F2,F4,F6,F8"),
            vec![
                "Fz", "C3", "Cz", "C4", "Pz", "PO7", "Oz", "PO8", "F5", "F7", "F3", "F1", "F2",
                "F4", "F6", "F8"
            ]
        );
    }

    #[test]
    fn eeg_names_parser_returns_empty_for_empty_native_string() {
        assert!(parse_eeg_names("").is_empty());
    }

    #[test]
    fn returned_samples_must_fit_requested_capacity() {
        assert_eq!(validate_returned_samples(2, 5).unwrap(), 2);
        assert!(matches!(
            validate_returned_samples(-1, 5),
            Err(Error::InvalidLength)
        ));
        assert!(matches!(
            validate_returned_samples(6, 5),
            Err(Error::InvalidLength)
        ));
    }

    #[test]
    fn requested_board_data_is_clamped_to_available_samples() {
        assert_eq!(clamp_data_count(3, 10), 3);
        assert_eq!(clamp_data_count(10, 3), 3);
    }
}
