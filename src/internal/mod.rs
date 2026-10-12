mod buffers;
mod data_filter;
mod json;
mod metadata;
mod ml_model;
mod status;

pub(crate) use crate::logging_bridge::ensure_logging_bridge;
pub(crate) use buffers::{
    MAX_NATIVE_FILE_ELEMENTS, array2_from_native, checked_element_count, checked_native_capacity,
    read_i32_vec, read_string, sum_decomposition_lengths, wavelet_coefficient_capacity,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use buffers::{array3_from_native, checked_buffer_size, checked_native_len};
pub(crate) use data_filter::{
    calc_stddev, data_filter_log_file, data_filter_log_level, data_filter_log_message,
    data_filter_version, detect_peaks_z_score, detrend, get_band_power, get_csp,
    get_custom_band_powers, get_heart_rate, get_nearest_power_of_two, get_oxygen_level, get_psd,
    get_psd_welch, get_railed_percentage, get_rmssd_hrv, get_window, perform_bandpass,
    perform_bandstop, perform_downsampling, perform_fft, perform_highpass, perform_ica,
    perform_ifft, perform_inverse_wavelet_transform, perform_lowpass, perform_rolling_filter,
    perform_wavelet_denoising, perform_wavelet_transform, read_file, remove_environmental_noise,
    restore_data_from_wavelet_detailed_coeffs, write_file,
};
pub(crate) use json::{to_cstring, to_json_cstring};
pub(crate) use metadata::{
    add_streamer, board_channels, board_metadata, board_presets, board_scalar, board_version,
    config_board, delete_streamer, get_board_data, get_board_data_count, get_current_board_data,
    insert_marker, is_prepared, logger_call, prepare_session, release_all_sessions,
    release_session, set_log_file, set_log_file_raw, set_log_level_raw, start_stream, stop_stream,
};
pub(crate) use ml_model::{
    log_file as ml_log_file, log_level as ml_log_level, log_message as ml_log_message,
    predict as ml_predict, prepare as ml_prepare, release as ml_release,
    release_all as ml_release_all, version as ml_version,
};
pub(crate) use status::check_status;
