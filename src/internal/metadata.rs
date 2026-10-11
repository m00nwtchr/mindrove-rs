use std::ffi::{c_char, c_int};

pub(crate) fn board_metadata(
    kind: u8,
    board_id: i32,
    preset: i32,
    buffer: *mut c_char,
    len: *mut i32,
) -> i32 {
    unsafe {
        match kind {
            0 => mindrove_sys::get_board_descr(board_id, preset, buffer, len),
            1 => mindrove_sys::get_eeg_names(board_id, preset, buffer, len),
            2 => mindrove_sys::get_device_name(board_id, preset, buffer, len),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn board_scalar(kind: u8, board_id: i32, preset: i32, value: *mut i32) -> i32 {
    unsafe {
        match kind {
            0 => mindrove_sys::get_sampling_rate(board_id, preset, value),
            1 => mindrove_sys::get_package_num_channel(board_id, preset, value),
            2 => mindrove_sys::get_timestamp_channel(board_id, preset, value),
            3 => mindrove_sys::get_marker_channel(board_id, preset, value),
            4 => mindrove_sys::get_battery_channel(board_id, preset, value),
            5 => mindrove_sys::get_num_rows(board_id, preset, value),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn board_channels(
    kind: u8,
    board_id: i32,
    preset: i32,
    values: *mut i32,
    len: *mut i32,
) -> i32 {
    unsafe {
        match kind {
            0 => mindrove_sys::get_exg_channels(board_id, preset, values, len),
            1 => mindrove_sys::get_eeg_channels(board_id, preset, values, len),
            2 => mindrove_sys::get_emg_channels(board_id, preset, values, len),
            3 => mindrove_sys::get_ecg_channels(board_id, preset, values, len),
            4 => mindrove_sys::get_eog_channels(board_id, preset, values, len),
            5 => mindrove_sys::get_ppg_channels(board_id, preset, values, len),
            6 => mindrove_sys::get_ppg_raw_channels(board_id, preset, values, len),
            7 => mindrove_sys::get_eda_channels(board_id, preset, values, len),
            8 => mindrove_sys::get_accel_channels(board_id, preset, values, len),
            9 => mindrove_sys::get_rotation_channels(board_id, preset, values, len),
            10 => mindrove_sys::get_analog_channels(board_id, preset, values, len),
            11 => mindrove_sys::get_gyro_channels(board_id, preset, values, len),
            12 => mindrove_sys::get_other_channels(board_id, preset, values, len),
            13 => mindrove_sys::get_temperature_channels(board_id, preset, values, len),
            14 => mindrove_sys::get_resistance_channels(board_id, preset, values, len),
            15 => mindrove_sys::get_magnetometer_channels(board_id, preset, values, len),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn board_presets(board_id: i32, values: *mut i32, len: *mut i32) -> i32 {
    unsafe { mindrove_sys::get_board_presets(board_id, values, len) }
}

pub(crate) fn board_version(buffer: *mut c_char, len: *mut i32, max_len: i32) -> i32 {
    unsafe { mindrove_sys::get_version_board_controller(buffer, len, max_len) }
}

pub(crate) fn logger_call(kind: u8, value: i32, text: *mut c_char) -> i32 {
    unsafe {
        match kind {
            0 => mindrove_sys::set_log_level_board_controller(value),
            1 => mindrove_sys::log_message_board_controller(value, text),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn set_log_file(path: *const c_char) -> i32 {
    unsafe { mindrove_sys::set_log_file_board_controller(path) }
}

pub(crate) fn release_all_sessions() -> i32 {
    unsafe { mindrove_sys::release_all_sessions() }
}

pub(crate) fn prepare_session(board_id: i32, params: *const c_char) -> i32 {
    unsafe { mindrove_sys::prepare_session(board_id, params) }
}

pub(crate) fn start_stream(
    buffer_size: c_int,
    streamer_params: *const c_char,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe { mindrove_sys::start_stream(buffer_size, streamer_params, board_id, params) }
}

pub(crate) fn stop_stream(board_id: i32, params: *const c_char) -> i32 {
    unsafe { mindrove_sys::stop_stream(board_id, params) }
}

pub(crate) fn release_session(board_id: i32, params: *const c_char) -> i32 {
    unsafe { mindrove_sys::release_session(board_id, params) }
}

pub(crate) fn is_prepared(result: *mut c_int, board_id: i32, params: *const c_char) -> i32 {
    unsafe { mindrove_sys::is_prepared(result, board_id, params) }
}

pub(crate) fn get_board_data_count(
    preset: i32,
    result: *mut c_int,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe { mindrove_sys::get_board_data_count(preset, result, board_id, params) }
}

pub(crate) fn get_current_board_data(
    num_samples: c_int,
    preset: i32,
    data: *mut f64,
    returned_samples: *mut c_int,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe {
        mindrove_sys::get_current_board_data(
            num_samples,
            preset,
            data,
            returned_samples,
            board_id,
            params,
        )
    }
}

pub(crate) fn get_board_data(
    data_count: c_int,
    preset: i32,
    data: *mut f64,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe { mindrove_sys::get_board_data(data_count, preset, data, board_id, params) }
}

pub(crate) fn config_board(
    config: *const c_char,
    response: *mut c_char,
    response_len: *mut c_int,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe { mindrove_sys::config_board(config, response, response_len, board_id, params) }
}

pub(crate) fn insert_marker(value: f64, preset: i32, board_id: i32, params: *const c_char) -> i32 {
    unsafe { mindrove_sys::insert_marker(value, preset, board_id, params) }
}

pub(crate) fn add_streamer(
    streamer: *const c_char,
    preset: i32,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe { mindrove_sys::add_streamer(streamer, preset, board_id, params) }
}

pub(crate) fn delete_streamer(
    streamer: *const c_char,
    preset: i32,
    board_id: i32,
    params: *const c_char,
) -> i32 {
    unsafe { mindrove_sys::delete_streamer(streamer, preset, board_id, params) }
}
