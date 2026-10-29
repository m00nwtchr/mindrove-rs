// Forwarders mirror the data_handler C ABI exactly; parameter counts are
// part of the contract, so allow the FFI-signature-driven argument warnings.
#![allow(clippy::too_many_arguments)]

use std::os::raw::{c_char, c_int};

pub(crate) fn data_filter_version(buffer: *mut c_char, len: *mut c_int, max_len: c_int) -> c_int {
    unsafe { mindrove_sys::get_version_data_handler(buffer, len, max_len) }
}

pub(crate) fn perform_lowpass(
    data: *mut f64,
    data_len: c_int,
    sampling_rate: c_int,
    cutoff: f64,
    order: c_int,
    filter_type: c_int,
    ripple: f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_lowpass(
            data,
            data_len,
            sampling_rate,
            cutoff,
            order,
            filter_type,
            ripple,
        )
    }
}

pub(crate) fn perform_highpass(
    data: *mut f64,
    data_len: c_int,
    sampling_rate: c_int,
    cutoff: f64,
    order: c_int,
    filter_type: c_int,
    ripple: f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_highpass(
            data,
            data_len,
            sampling_rate,
            cutoff,
            order,
            filter_type,
            ripple,
        )
    }
}

pub(crate) fn perform_bandpass(
    data: *mut f64,
    data_len: c_int,
    sampling_rate: c_int,
    start_freq: f64,
    stop_freq: f64,
    order: c_int,
    filter_type: c_int,
    ripple: f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_bandpass(
            data,
            data_len,
            sampling_rate,
            start_freq,
            stop_freq,
            order,
            filter_type,
            ripple,
        )
    }
}

pub(crate) fn perform_bandstop(
    data: *mut f64,
    data_len: c_int,
    sampling_rate: c_int,
    start_freq: f64,
    stop_freq: f64,
    order: c_int,
    filter_type: c_int,
    ripple: f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_bandstop(
            data,
            data_len,
            sampling_rate,
            start_freq,
            stop_freq,
            order,
            filter_type,
            ripple,
        )
    }
}

pub(crate) fn remove_environmental_noise(
    data: *mut f64,
    data_len: c_int,
    sampling_rate: c_int,
    noise_type: c_int,
) -> c_int {
    unsafe { mindrove_sys::remove_environmental_noise(data, data_len, sampling_rate, noise_type) }
}

pub(crate) fn perform_rolling_filter(
    data: *mut f64,
    data_len: c_int,
    period: c_int,
    agg_operation: c_int,
) -> c_int {
    unsafe { mindrove_sys::perform_rolling_filter(data, data_len, period, agg_operation) }
}

pub(crate) fn detrend(data: *mut f64, data_len: c_int, detrend_operation: c_int) -> c_int {
    unsafe { mindrove_sys::detrend(data, data_len, detrend_operation) }
}

pub(crate) fn perform_wavelet_denoising(
    data: *mut f64,
    data_len: c_int,
    wavelet: c_int,
    decomposition_level: c_int,
    wavelet_denoising: c_int,
    threshold: c_int,
    extension_type: c_int,
    noise_level: c_int,
) -> c_int {
    unsafe {
        mindrove_sys::perform_wavelet_denoising(
            data,
            data_len,
            wavelet,
            decomposition_level,
            wavelet_denoising,
            threshold,
            extension_type,
            noise_level,
        )
    }
}

pub(crate) fn restore_data_from_wavelet_detailed_coeffs(
    data: *mut f64,
    data_len: c_int,
    wavelet: c_int,
    decomposition_level: c_int,
    level_to_restore: c_int,
    output: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::restore_data_from_wavelet_detailed_coeffs(
            data,
            data_len,
            wavelet,
            decomposition_level,
            level_to_restore,
            output,
        )
    }
}

pub(crate) fn detect_peaks_z_score(
    data: *mut f64,
    data_len: c_int,
    lag: c_int,
    threshold: f64,
    influence: f64,
    output: *mut f64,
) -> c_int {
    unsafe { mindrove_sys::detect_peaks_z_score(data, data_len, lag, threshold, influence, output) }
}

pub(crate) fn get_window(window_function: c_int, window_len: c_int, output: *mut f64) -> c_int {
    unsafe { mindrove_sys::get_window(window_function, window_len, output) }
}

pub(crate) fn get_nearest_power_of_two(value: c_int, output: *mut c_int) -> c_int {
    unsafe { mindrove_sys::get_nearest_power_of_two(value, output) }
}

pub(crate) fn calc_stddev(
    data: *mut f64,
    start_pos: c_int,
    end_pos: c_int,
    output: *mut f64,
) -> c_int {
    unsafe { mindrove_sys::calc_stddev(data, start_pos, end_pos, output) }
}

pub(crate) fn get_railed_percentage(
    data: *mut f64,
    data_len: c_int,
    gain: c_int,
    output: *mut f64,
) -> c_int {
    unsafe { mindrove_sys::get_railed_percentage(data, data_len, gain, output) }
}

pub(crate) fn get_oxygen_level(
    ppg_ir: *mut f64,
    ppg_red: *mut f64,
    data_size: c_int,
    sampling_rate: c_int,
    coef1: f64,
    coef2: f64,
    coef3: f64,
    oxygen_level: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_oxygen_level(
            ppg_ir,
            ppg_red,
            data_size,
            sampling_rate,
            coef1,
            coef2,
            coef3,
            oxygen_level,
        )
    }
}

pub(crate) fn get_heart_rate(
    ppg_ir: *mut f64,
    ppg_red: *mut f64,
    ppg_green: *mut f64,
    data_size: c_int,
    sampling_rate: c_int,
    fft_size: c_int,
    rate: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_heart_rate(
            ppg_ir,
            ppg_red,
            ppg_green,
            data_size,
            sampling_rate,
            fft_size,
            rate,
        )
    }
}

pub(crate) fn get_rmssd_hrv(
    ppg_ir: *const f64,
    ppg_red: *const f64,
    ppg_green: *const f64,
    data_size: c_int,
    sampling_rate: c_int,
    out_rmssd: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_rmssd_hrv(
            ppg_ir,
            ppg_red,
            ppg_green,
            data_size,
            sampling_rate,
            out_rmssd,
        )
    }
}

pub(crate) fn perform_fft(
    data: *mut f64,
    data_len: c_int,
    window_function: c_int,
    output_re: *mut f64,
    output_im: *mut f64,
) -> c_int {
    unsafe { mindrove_sys::perform_fft(data, data_len, window_function, output_re, output_im) }
}

pub(crate) fn perform_ifft(
    input_re: *mut f64,
    input_im: *mut f64,
    data_len: c_int,
    restored_data: *mut f64,
) -> c_int {
    unsafe { mindrove_sys::perform_ifft(input_re, input_im, data_len, restored_data) }
}

pub(crate) fn get_psd(
    data: *mut f64,
    data_len: c_int,
    sampling_rate: c_int,
    window_function: c_int,
    output_ampl: *mut f64,
    output_freq: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_psd(
            data,
            data_len,
            sampling_rate,
            window_function,
            output_ampl,
            output_freq,
        )
    }
}

pub(crate) fn get_psd_welch(
    data: *mut f64,
    data_len: c_int,
    nfft: c_int,
    overlap: c_int,
    sampling_rate: c_int,
    window_function: c_int,
    output_ampl: *mut f64,
    output_freq: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_psd_welch(
            data,
            data_len,
            nfft,
            overlap,
            sampling_rate,
            window_function,
            output_ampl,
            output_freq,
        )
    }
}

pub(crate) fn get_band_power(
    ampl: *mut f64,
    freq: *mut f64,
    data_len: c_int,
    freq_start: f64,
    freq_end: f64,
    band_power: *mut f64,
) -> c_int {
    unsafe { mindrove_sys::get_band_power(ampl, freq, data_len, freq_start, freq_end, band_power) }
}

pub(crate) fn get_custom_band_powers(
    raw_data: *mut f64,
    rows: c_int,
    cols: c_int,
    start_freqs: *mut f64,
    stop_freqs: *mut f64,
    num_bands: c_int,
    sampling_rate: c_int,
    apply_filters: c_int,
    avg_band_powers: *mut f64,
    stddev_band_powers: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_custom_band_powers(
            raw_data,
            rows,
            cols,
            start_freqs,
            stop_freqs,
            num_bands,
            sampling_rate,
            apply_filters,
            avg_band_powers,
            stddev_band_powers,
        )
    }
}

pub(crate) fn perform_downsampling(
    data: *mut f64,
    data_len: c_int,
    period: c_int,
    agg_operation: c_int,
    output_data: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_downsampling(data, data_len, period, agg_operation, output_data)
    }
}

pub(crate) fn perform_wavelet_transform(
    data: *mut f64,
    data_len: c_int,
    wavelet: c_int,
    decomposition_level: c_int,
    extension: c_int,
    output_data: *mut f64,
    decomposition_lengths: *mut c_int,
) -> c_int {
    unsafe {
        mindrove_sys::perform_wavelet_transform(
            data,
            data_len,
            wavelet,
            decomposition_level,
            extension,
            output_data,
            decomposition_lengths,
        )
    }
}

pub(crate) fn perform_inverse_wavelet_transform(
    wavelet_coeffs: *mut f64,
    original_data_len: c_int,
    wavelet: c_int,
    decomposition_level: c_int,
    extension: c_int,
    decomposition_lengths: *mut c_int,
    output_data: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_inverse_wavelet_transform(
            wavelet_coeffs,
            original_data_len,
            wavelet,
            decomposition_level,
            extension,
            decomposition_lengths,
            output_data,
        )
    }
}

pub(crate) fn get_csp(
    data: *const f64,
    labels: *const f64,
    n_epochs: c_int,
    n_channels: c_int,
    n_times: c_int,
    output_w: *mut f64,
    output_d: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::get_csp(
            data, labels, n_epochs, n_channels, n_times, output_w, output_d,
        )
    }
}

pub(crate) fn perform_ica(
    data: *mut f64,
    rows: c_int,
    cols: c_int,
    num_components: c_int,
    w_mat: *mut f64,
    k_mat: *mut f64,
    a_mat: *mut f64,
    s_mat: *mut f64,
) -> c_int {
    unsafe {
        mindrove_sys::perform_ica(data, rows, cols, num_components, w_mat, k_mat, a_mat, s_mat)
    }
}

pub(crate) fn write_file(
    data: *const f64,
    num_rows: c_int,
    num_cols: c_int,
    file_name: *const c_char,
    file_mode: *const c_char,
) -> c_int {
    unsafe { mindrove_sys::write_file(data, num_rows, num_cols, file_name, file_mode) }
}

pub(crate) fn read_file(
    data: *mut f64,
    num_rows: *mut c_int,
    num_cols: *mut c_int,
    file_name: *const c_char,
    num_elements: c_int,
) -> c_int {
    unsafe { mindrove_sys::read_file(data, num_rows, num_cols, file_name, num_elements) }
}
