use ndarray::{Array1, Array2, Array3};
use num_complex::Complex;

use crate::internal;
use crate::{
    AggOperation, DetrendOperation, Error, FilterType, NoiseEstimationLevelType, NoiseType, Result,
    ThresholdType, WaveletDenoisingType, WaveletExtensionType, WaveletType, WindowOperation,
};

pub const DEFAULT_OXYGEN_COEFS: (f64, f64, f64) = (1.5958422, -34.6596622, 112.6898759);

/// A frequency band used by [`DataFilter::get_custom_band_powers`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrequencyBand {
    pub start: f64,
    pub end: f64,
}

impl FrequencyBand {
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }
}

impl From<(f64, f64)> for FrequencyBand {
    fn from(value: (f64, f64)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

/// Result of [`DataFilter::get_custom_band_powers`] and the C++ average helper.
#[derive(Debug, Clone, PartialEq)]
pub struct BandPowerPair {
    pub average: Vec<f64>,
    pub stddev: Vec<f64>,
}

impl BandPowerPair {
    fn from_raw(average: Vec<f64>, stddev: Vec<f64>) -> Self {
        Self { average, stddev }
    }
}

/// Default SDK "brain wave" bands used by
/// [`DataFilter::get_avg_band_powers`]. Five fixed bands: delta, theta,
/// alpha, beta, gamma per the BrainFlow/MindRove reference implementation.
pub const DEFAULT_BANDS: [FrequencyBand; 5] = [
    FrequencyBand {
        start: 2.0,
        end: 4.0,
    },
    FrequencyBand {
        start: 4.0,
        end: 8.0,
    },
    FrequencyBand {
        start: 8.0,
        end: 13.0,
    },
    FrequencyBand {
        start: 13.0,
        end: 30.0,
    },
    FrequencyBand {
        start: 30.0,
        end: 45.0,
    },
];

/// Result of [`DataFilter::perform_wavelet_transform`].
#[derive(Debug, Clone, PartialEq)]
pub struct WaveletCoefficients {
    pub coefficients: Vec<f64>,
    pub decomposition_lengths: Vec<usize>,
}

/// Result of [`DataFilter::get_csp`].
#[derive(Debug, Clone, PartialEq)]
pub struct CspResult {
    pub filters: Array2<f64>,
    pub eigenvalues: Array1<f64>,
}

/// Result of [`DataFilter::perform_ica`].
#[derive(Debug, Clone, PartialEq)]
pub struct IcaResult {
    pub w: Array2<f64>,
    pub k: Array2<f64>,
    pub a: Array2<f64>,
    pub s: Array2<f64>,
}

pub struct DataFilter;

impl DataFilter {
    pub fn get_version() -> Result<String> {
        internal::ensure_logging_bridge();
        const VERSION_BUFFER_LEN: i32 = 64;
        let mut buffer = vec![0_u8; VERSION_BUFFER_LEN as usize];
        let mut len = VERSION_BUFFER_LEN;
        internal::check_status(internal::data_filter_version(
            buffer.as_mut_ptr().cast(),
            &mut len,
            VERSION_BUFFER_LEN,
        ))?;
        let len = usize::try_from(len).map_err(|_| Error::InvalidLength)?;
        if len > buffer.len() {
            return Err(Error::BufferContract(
                "data handler version fill exceeded bounded buffer".into(),
            ));
        }
        String::from_utf8(buffer[..len].to_vec()).map_err(|_| Error::InvalidUtf8)
    }

    pub fn perform_lowpass(
        data: &mut [f64],
        sampling_rate: i32,
        cutoff: f64,
        order: i32,
        filter_type: FilterType,
        ripple: f64,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        validate_positive_i32(order, "order")?;
        validate_finite(cutoff, "cutoff")?;
        validate_finite(ripple, "ripple")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::perform_lowpass(
            data.as_mut_ptr(),
            native_len,
            sampling_rate,
            cutoff,
            order,
            filter_type as i32,
            ripple,
        ))
    }

    pub fn perform_highpass(
        data: &mut [f64],
        sampling_rate: i32,
        cutoff: f64,
        order: i32,
        filter_type: FilterType,
        ripple: f64,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        validate_positive_i32(order, "order")?;
        validate_finite(cutoff, "cutoff")?;
        validate_finite(ripple, "ripple")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::perform_highpass(
            data.as_mut_ptr(),
            native_len,
            sampling_rate,
            cutoff,
            order,
            filter_type as i32,
            ripple,
        ))
    }

    pub fn perform_bandpass(
        data: &mut [f64],
        sampling_rate: i32,
        start_freq: f64,
        stop_freq: f64,
        order: i32,
        filter_type: FilterType,
        ripple: f64,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        validate_positive_i32(order, "order")?;
        validate_band_order(start_freq, stop_freq)?;
        validate_finite(ripple, "ripple")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::perform_bandpass(
            data.as_mut_ptr(),
            native_len,
            sampling_rate,
            start_freq,
            stop_freq,
            order,
            filter_type as i32,
            ripple,
        ))
    }

    pub fn perform_bandstop(
        data: &mut [f64],
        sampling_rate: i32,
        start_freq: f64,
        stop_freq: f64,
        order: i32,
        filter_type: FilterType,
        ripple: f64,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        validate_positive_i32(order, "order")?;
        validate_band_order(start_freq, stop_freq)?;
        validate_finite(ripple, "ripple")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::perform_bandstop(
            data.as_mut_ptr(),
            native_len,
            sampling_rate,
            start_freq,
            stop_freq,
            order,
            filter_type as i32,
            ripple,
        ))
    }

    pub fn remove_environmental_noise(
        data: &mut [f64],
        sampling_rate: i32,
        noise_type: NoiseType,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::remove_environmental_noise(
            data.as_mut_ptr(),
            native_len,
            sampling_rate,
            noise_type as i32,
        ))
    }

    pub fn perform_rolling_filter(
        data: &mut [f64],
        period: i32,
        operation: AggOperation,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(period, "period")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::perform_rolling_filter(
            data.as_mut_ptr(),
            native_len,
            period,
            operation as i32,
        ))
    }

    pub fn detrend(data: &mut [f64], operation: DetrendOperation) -> Result<()> {
        validate_nonempty(data, "data")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::detrend(
            data.as_mut_ptr(),
            native_len,
            operation as i32,
        ))
    }

    pub fn get_nearest_power_of_two(value: i32) -> Result<i32> {
        let mut output: i32 = 0;
        internal::check_status(internal::get_nearest_power_of_two(value, &mut output))?;
        Ok(output)
    }

    pub fn calc_stddev(data: &[f64], start_pos: usize, end_pos: usize) -> Result<f64> {
        validate_nonempty(data, "data")?;
        validate_stddev_range(start_pos, end_pos, data.len())?;
        let mut output: f64 = 0.0;
        let native_start = internal::checked_native_capacity(start_pos)?;
        let native_end = internal::checked_native_capacity(end_pos)?;
        internal::check_status(internal::calc_stddev(
            data.as_ptr().cast_mut(),
            native_start,
            native_end,
            &mut output,
        ))?;
        Ok(output)
    }

    pub fn get_railed_percentage(data: &[f64], gain: i32) -> Result<f64> {
        validate_nonempty(data, "data")?;
        let native_len = checked_data_len(data.len(), "data")?;
        let mut output: f64 = 0.0;
        internal::check_status(internal::get_railed_percentage(
            data.as_ptr().cast_mut(),
            native_len,
            gain,
            &mut output,
        ))?;
        Ok(output)
    }

    pub fn get_oxygen_level_with_coefs(
        ppg_ir: &[f64],
        ppg_red: &[f64],
        sampling_rate: i32,
        coef1: f64,
        coef2: f64,
        coef3: f64,
    ) -> Result<f64> {
        validate_equal_ppg_lengths(ppg_ir, ppg_red, None)?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        validate_finite(coef1, "coef1")?;
        validate_finite(coef2, "coef2")?;
        validate_finite(coef3, "coef3")?;
        let native_len = checked_data_len(ppg_ir.len(), "ppg_ir")?;
        let mut output: f64 = 0.0;
        internal::check_status(internal::get_oxygen_level(
            ppg_ir.as_ptr().cast_mut(),
            ppg_red.as_ptr().cast_mut(),
            native_len,
            sampling_rate,
            coef1,
            coef2,
            coef3,
            &mut output,
        ))?;
        Ok(output)
    }

    pub fn get_oxygen_level(ppg_ir: &[f64], ppg_red: &[f64], sampling_rate: i32) -> Result<f64> {
        Self::get_oxygen_level_with_coefs(
            ppg_ir,
            ppg_red,
            sampling_rate,
            DEFAULT_OXYGEN_COEFS.0,
            DEFAULT_OXYGEN_COEFS.1,
            DEFAULT_OXYGEN_COEFS.2,
        )
    }

    pub fn get_heart_rate(
        ppg_ir: &[f64],
        ppg_red: &[f64],
        ppg_green: &[f64],
        sampling_rate: i32,
        fft_size: i32,
    ) -> Result<f64> {
        validate_equal_ppg_lengths(ppg_ir, ppg_red, Some(ppg_green))?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        validate_positive_i32(fft_size, "fft_size")?;
        let native_len = checked_data_len(ppg_ir.len(), "ppg_ir")?;
        let mut output: f64 = 0.0;
        internal::check_status(internal::get_heart_rate(
            ppg_ir.as_ptr().cast_mut(),
            ppg_red.as_ptr().cast_mut(),
            ppg_green.as_ptr().cast_mut(),
            native_len,
            sampling_rate,
            fft_size,
            &mut output,
        ))?;
        Ok(output)
    }

    pub fn get_rmssd_hrv(
        ppg_ir: &[f64],
        ppg_red: &[f64],
        ppg_green: &[f64],
        sampling_rate: i32,
    ) -> Result<f64> {
        validate_equal_ppg_lengths(ppg_ir, ppg_red, Some(ppg_green))?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        let native_len = checked_data_len(ppg_ir.len(), "ppg_ir")?;
        let mut output: f64 = 0.0;
        internal::check_status(internal::get_rmssd_hrv(
            ppg_ir.as_ptr(),
            ppg_red.as_ptr(),
            ppg_green.as_ptr(),
            native_len,
            sampling_rate,
            &mut output,
        ))?;
        Ok(output)
    }

    pub fn get_window(window_operation: WindowOperation, window_len: usize) -> Result<Vec<f64>> {
        validate_positive_usize(window_len, "window_len")?;
        let native_len = internal::checked_native_capacity(window_len)?;
        let mut output = vec![0.0_f64; window_len];
        internal::check_status(internal::get_window(
            window_operation as i32,
            native_len,
            output.as_mut_ptr(),
        ))?;
        Ok(output)
    }

    pub fn detect_peaks_z_score(
        data: &mut [f64],
        lag: i32,
        threshold: f64,
        influence: f64,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(lag, "lag")?;
        validate_finite(threshold, "threshold")?;
        validate_finite(influence, "influence")?;
        let native_len = checked_data_len(data.len(), "data")?;
        let mut output = vec![0.0_f64; data.len()];
        internal::check_status(internal::detect_peaks_z_score(
            data.as_mut_ptr(),
            native_len,
            lag,
            threshold,
            influence,
            output.as_mut_ptr(),
        ))?;
        copy_output_into(data, &output);
        Ok(())
    }

    pub fn perform_wavelet_denoising(
        data: &mut [f64],
        wavelet: WaveletType,
        decomposition_level: i32,
        denoising: WaveletDenoisingType,
        threshold: ThresholdType,
        extension: WaveletExtensionType,
        noise_level: NoiseEstimationLevelType,
    ) -> Result<()> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(decomposition_level, "decomposition_level")?;
        let native_len = checked_data_len(data.len(), "data")?;
        internal::check_status(internal::perform_wavelet_denoising(
            data.as_mut_ptr(),
            native_len,
            wavelet as i32,
            decomposition_level,
            denoising as i32,
            threshold as i32,
            extension as i32,
            noise_level as i32,
        ))
    }

    pub fn restore_data_from_wavelet_detailed_coeffs(
        data: &[f64],
        wavelet: WaveletType,
        decomposition_level: i32,
        level_to_restore: i32,
    ) -> Result<Vec<f64>> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(decomposition_level, "decomposition_level")?;
        validate_positive_i32(level_to_restore, "level_to_restore")?;
        if level_to_restore > decomposition_level {
            return Err(Error::InvalidArgument(format!(
                "level_to_restore ({level_to_restore}) must not exceed decomposition_level ({decomposition_level})"
            )));
        }
        let native_len = checked_data_len(data.len(), "data")?;
        let mut output = vec![0.0_f64; data.len()];
        internal::check_status(internal::restore_data_from_wavelet_detailed_coeffs(
            data.as_ptr().cast_mut(),
            native_len,
            wavelet as i32,
            decomposition_level,
            level_to_restore,
            output.as_mut_ptr(),
        ))?;
        Ok(output)
    }

    /// Direct FFT with even-length requirement and `n/2 + 1` complex bins.
    pub fn perform_fft(data: &[f64], window: WindowOperation) -> Result<Array1<Complex<f64>>> {
        validate_nonempty(data, "data")?;
        validate_even_length(data.len(), "data")?;
        let native_len = checked_data_len(data.len(), "data")?;
        let output_len = data.len() / 2 + 1;
        let mut re = vec![0.0_f64; output_len];
        let mut im = vec![0.0_f64; output_len];
        internal::check_status(internal::perform_fft(
            data.as_ptr().cast_mut(),
            native_len,
            window as i32,
            re.as_mut_ptr(),
            im.as_mut_ptr(),
        ))?;
        Ok(complex_array_from_parts(re, im))
    }

    /// Inverse FFT. The output length equals the original sample count
    /// (`fft_len` reported by [`DataFilter::perform_fft`], not `n/2 + 1`).
    pub fn perform_ifft(data: &Array1<Complex<f64>>) -> Result<Array1<f64>> {
        validate_nonempty_array1(data, "data")?;
        let (re, im) = complex_split(data);
        let output_len = re.len();
        if output_len < 2 {
            return Err(Error::InvalidArgument(
                "FFT input must contain at least two bins".into(),
            ));
        }
        let restored_len = (output_len - 1) * 2;
        let native_len = internal::checked_native_capacity(restored_len)?;
        let mut restored = vec![0.0_f64; restored_len];
        let mut re_mut = re;
        let mut im_mut = im;
        internal::check_status(internal::perform_ifft(
            re_mut.as_mut_ptr(),
            im_mut.as_mut_ptr(),
            native_len,
            restored.as_mut_ptr(),
        ))?;
        Ok(Array1::from_vec(restored))
    }

    /// Periodogram-style PSD with `n/2 + 1` amplitude/frequency bins.
    pub fn get_psd(
        data: &[f64],
        sampling_rate: i32,
        window: WindowOperation,
    ) -> Result<(Array1<f64>, Array1<f64>)> {
        validate_nonempty(data, "data")?;
        validate_even_length(data.len(), "data")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        let native_len = checked_data_len(data.len(), "data")?;
        let output_len = data.len() / 2 + 1;
        let mut ampl = vec![0.0_f64; output_len];
        let mut freq = vec![0.0_f64; output_len];
        internal::check_status(internal::get_psd(
            data.as_ptr().cast_mut(),
            native_len,
            sampling_rate,
            window as i32,
            ampl.as_mut_ptr(),
            freq.as_mut_ptr(),
        ))?;
        Ok((Array1::from_vec(ampl), Array1::from_vec(freq)))
    }

    /// Welch PSD. The output length follows `nfft / 2 + 1` per the SDK
    /// convention; we encode that directly because the C ABI does not
    /// expose the resulting length through an out-parameter.
    pub fn get_psd_welch(
        data: &[f64],
        nfft: i32,
        overlap: i32,
        sampling_rate: i32,
        window: WindowOperation,
    ) -> Result<(Array1<f64>, Array1<f64>)> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(nfft, "nfft")?;
        validate_even_length(nfft as usize, "nfft")?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        if overlap < 0 || overlap >= nfft {
            return Err(Error::InvalidArgument(format!(
                "overlap ({overlap}) must satisfy 0 <= overlap < nfft ({nfft})"
            )));
        }
        let native_len = checked_data_len(data.len(), "data")?;
        let output_len = (nfft as usize) / 2 + 1;
        let mut ampl = vec![0.0_f64; output_len];
        let mut freq = vec![0.0_f64; output_len];
        internal::check_status(internal::get_psd_welch(
            data.as_ptr().cast_mut(),
            native_len,
            nfft,
            overlap,
            sampling_rate,
            window as i32,
            ampl.as_mut_ptr(),
            freq.as_mut_ptr(),
        ))?;
        Ok((Array1::from_vec(ampl), Array1::from_vec(freq)))
    }

    /// Band power scalar from a previously computed PSD.
    pub fn get_band_power(
        psd: &(Array1<f64>, Array1<f64>),
        freq_start: f64,
        freq_end: f64,
    ) -> Result<f64> {
        let (ampl, freq) = psd;
        if ampl.len() != freq.len() {
            return Err(Error::InvalidLength);
        }
        if ampl.is_empty() {
            return Err(Error::InvalidArgument(
                "PSD amplitude array must not be empty".into(),
            ));
        }
        validate_finite(freq_start, "freq_start")?;
        validate_finite(freq_end, "freq_end")?;
        if freq_start >= freq_end {
            return Err(Error::InvalidArgument(
                "freq_end must exceed freq_start".into(),
            ));
        }
        let native_len = internal::checked_native_capacity(ampl.len())?;
        let mut output = 0.0_f64;
        internal::check_status(internal::get_band_power(
            ampl.as_ptr().cast_mut(),
            freq.as_ptr().cast_mut(),
            native_len,
            freq_start,
            freq_end,
            &mut output,
        ))?;
        Ok(output)
    }

    /// Custom band powers for an arbitrary `bands` list, computed across the
    /// selected channel rows of `data`. `data` is laid out as
    /// `(channels, samples)` and `channels` selects which rows contribute
    /// to the result; every channel index must satisfy
    /// `index < data.nrows()`.
    pub fn get_custom_band_powers(
        data: &Array2<f64>,
        channels: &[usize],
        bands: &[FrequencyBand],
        sampling_rate: i32,
        apply_filters: bool,
    ) -> Result<BandPowerPair> {
        if bands.is_empty() {
            return Err(Error::InvalidArgument(
                "bands list must not be empty".into(),
            ));
        }
        for band in bands {
            if !band.start.is_finite() || !band.end.is_finite() {
                return Err(Error::InvalidArgument(
                    "band frequencies must be finite".into(),
                ));
            }
            if band.start >= band.end {
                return Err(Error::InvalidArgument(format!(
                    "band end ({}) must exceed start ({})",
                    band.end, band.start
                )));
            }
        }
        let (selected_rows, samples) = select_channel_rows(data, channels)?;
        validate_positive_i32(sampling_rate, "sampling_rate")?;
        let num_bands = checked_data_len(bands.len(), "bands.len()")?;

        let mut start_freqs = Vec::with_capacity(bands.len());
        let mut stop_freqs = Vec::with_capacity(bands.len());
        for band in bands {
            start_freqs.push(band.start);
            stop_freqs.push(band.end);
        }
        let mut avg = vec![0.0_f64; bands.len()];
        let mut stddev = vec![0.0_f64; bands.len()];
        let rows_i = checked_data_len(channels.len(), "channels.len()")?;
        let cols_i = checked_data_len(samples, "samples")?;
        internal::check_status(internal::get_custom_band_powers(
            selected_rows.as_ptr().cast_mut(),
            rows_i,
            cols_i,
            start_freqs.as_mut_ptr(),
            stop_freqs.as_mut_ptr(),
            num_bands,
            sampling_rate,
            apply_filters as i32,
            avg.as_mut_ptr(),
            stddev.as_mut_ptr(),
        ))?;
        Ok(BandPowerPair::from_raw(avg, stddev))
    }

    /// Average and standard-deviation band powers across the selected
    /// channels of `data`, using the SDK's canonical default five bands.
    pub fn get_avg_band_powers(
        data: &Array2<f64>,
        channels: &[usize],
        sampling_rate: i32,
        apply_filters: bool,
    ) -> Result<BandPowerPair> {
        let bands = DEFAULT_BANDS.to_vec();
        Self::get_custom_band_powers(data, channels, &bands, sampling_rate, apply_filters)
    }

    /// Downsampling via the SDK's documented `floor(data_len / period)`
    /// aggregation length.
    pub fn perform_downsampling(
        data: &[f64],
        period: i32,
        operation: AggOperation,
    ) -> Result<Vec<f64>> {
        validate_nonempty(data, "data")?;
        validate_positive_i32(period, "period")?;
        let native_len = checked_data_len(data.len(), "data")?;
        let output_len = downsampled_len(data.len(), period as usize)?;
        let mut output = vec![0.0_f64; output_len];
        internal::check_status(internal::perform_downsampling(
            data.as_ptr().cast_mut(),
            native_len,
            period,
            operation as i32,
            output.as_mut_ptr(),
        ))?;
        validate_filled_length(output_len, output.len(), "downsampling")?;
        Ok(output)
    }

    /// Wavelet transform returning a coefficient array and `level + 1`
    /// decomposition block lengths.
    pub fn perform_wavelet_transform(
        data: &[f64],
        wavelet: WaveletType,
        decomposition_level: i32,
        extension: WaveletExtensionType,
    ) -> Result<WaveletCoefficients> {
        validate_nonempty(data, "data")?;
        if !(1..=100).contains(&decomposition_level) {
            return Err(Error::InvalidArgument(format!(
                "decomposition_level ({decomposition_level}) must be in 1..=100"
            )));
        }
        let level_usize = decomposition_level as usize;
        let native_len = checked_data_len(data.len(), "data")?;
        let level_count = level_usize + 1;
        let mut lengths = vec![0_i32; level_count];
        let capacity = internal::wavelet_coefficient_capacity(data.len(), level_usize)?;
        let mut coeffs = vec![0.0_f64; capacity];
        internal::check_status(internal::perform_wavelet_transform(
            data.as_ptr().cast_mut(),
            native_len,
            wavelet as i32,
            decomposition_level,
            extension as i32,
            coeffs.as_mut_ptr(),
            lengths.as_mut_ptr(),
        ))?;
        let total = internal::sum_decomposition_lengths(&lengths)?;
        if total > coeffs.len() {
            return Err(Error::BufferContract(format!(
                "native decomposition sum ({total}) exceeds proven allocation ({})",
                coeffs.len()
            )));
        }
        if total == 0 {
            return Err(Error::InvalidLength);
        }
        coeffs.truncate(total);
        let decomposition_lengths = lengths
            .into_iter()
            .map(|value| usize::try_from(value).map_err(|_| Error::InvalidLength))
            .collect::<Result<Vec<_>>>()?;
        Ok(WaveletCoefficients {
            coefficients: coeffs,
            decomposition_lengths,
        })
    }

    /// Inverse wavelet transform from the validated `(coefficients,
    /// decomposition_lengths)` pair returned by
    /// [`DataFilter::perform_wavelet_transform`].
    pub fn perform_inverse_wavelet_transform(
        coeffs: &[f64],
        decomposition_lengths: &[usize],
        original_data_len: usize,
        wavelet: WaveletType,
        decomposition_level: i32,
        extension: WaveletExtensionType,
    ) -> Result<Vec<f64>> {
        if decomposition_lengths.is_empty() {
            return Err(Error::InvalidArgument(
                "decomposition_lengths must not be empty".into(),
            ));
        }
        if coeffs.is_empty() {
            return Err(Error::InvalidArgument(
                "coefficients must not be empty".into(),
            ));
        }
        let sum: usize = decomposition_lengths
            .iter()
            .copied()
            .try_fold(0usize, |acc, value| acc.checked_add(value))
            .ok_or(Error::IntegerOverflow)?;
        if sum != coeffs.len() {
            return Err(Error::InvalidArgument(format!(
                "sum of decomposition_lengths ({sum}) must equal coefficient length ({})",
                coeffs.len()
            )));
        }
        if decomposition_lengths.len() != decomposition_level as usize + 1 {
            return Err(Error::InvalidArgument(format!(
                "decomposition_lengths.len() ({}) must equal decomposition_level + 1 ({})",
                decomposition_lengths.len(),
                decomposition_level as usize + 1
            )));
        }
        validate_positive_i32(decomposition_level, "decomposition_level")?;
        if original_data_len == 0 {
            return Err(Error::InvalidArgument(
                "original_data_len must be positive".into(),
            ));
        }
        let native_original = internal::checked_native_capacity(original_data_len)?;
        let mut native_lengths: Vec<i32> = Vec::with_capacity(decomposition_lengths.len());
        for &value in decomposition_lengths {
            native_lengths.push(i32::try_from(value).map_err(|_| Error::InvalidLength)?);
        }
        let mut output = vec![0.0_f64; original_data_len];
        let mut coeffs_owned = coeffs.to_vec();
        internal::check_status(internal::perform_inverse_wavelet_transform(
            coeffs_owned.as_mut_ptr(),
            native_original,
            wavelet as i32,
            decomposition_level,
            extension as i32,
            native_lengths.as_mut_ptr(),
            output.as_mut_ptr(),
        ))?;
        Ok(output)
    }

    /// Common Spatial Patterns filter/eigenvalue pair for two-class EEG
    /// epochs.
    pub fn get_csp(data: &Array3<f64>, labels: &Array1<f64>) -> Result<CspResult> {
        if data.is_empty() || labels.is_empty() {
            return Err(Error::InvalidArgument(
                "CSP data and labels must not be empty".into(),
            ));
        }
        let data = data.as_standard_layout();
        let (n_epochs, n_channels, n_times) = data.dim();
        if n_epochs < 2 {
            return Err(Error::InvalidArgument(
                "CSP requires at least two epochs".into(),
            ));
        }
        if n_channels == 0 || n_times == 0 {
            return Err(Error::InvalidArgument(
                "CSP requires positive channel and time dimensions".into(),
            ));
        }
        if labels.len() != n_epochs {
            return Err(Error::InvalidArgument(format!(
                "labels length ({}) must equal n_epochs ({})",
                labels.len(),
                n_epochs
            )));
        }
        let n_epochs_i = checked_data_len(n_epochs, "n_epochs")?;
        let n_channels_i = checked_data_len(n_channels, "n_channels")?;
        let n_times_i = checked_data_len(n_times, "n_times")?;
        let owned = data.into_owned();
        let mut w = vec![0.0_f64; n_channels * n_channels];
        let mut d = vec![0.0_f64; n_channels];
        let labels_owned = labels.to_vec();
        internal::check_status(internal::get_csp(
            owned.as_ptr(),
            labels_owned.as_ptr(),
            n_epochs_i,
            n_channels_i,
            n_times_i,
            w.as_mut_ptr(),
            d.as_mut_ptr(),
        ))?;
        let filters = internal::array2_from_native(w, n_channels, n_channels)?;
        let eigenvalues = Array1::from_vec(d);
        Ok(CspResult {
            filters,
            eigenvalues,
        })
    }

    /// ICA over the selected channel rows of `data`. The data layout is
    /// `(samples, channels)` so that `selected.len()` channels and the
    /// sample count are passed to the native `perform_ica` (which expects
    /// `(rows = samples, cols = selected_channels)`). Validates that
    /// `num_components >= 2`, `num_components <= selected_channels.len()`,
    /// and `samples >= 2`. Output shapes per the SDK contract: W `[nc,nc]`,
    /// K `[nc,selected_channels]`, A `[selected_channels,nc]`,
    /// S `[nc,samples]`.
    pub fn perform_ica_with_channels(
        data: &Array2<f64>,
        num_components: i32,
        channels: &[usize],
    ) -> Result<IcaResult> {
        let (selected, samples) = ica_selected_row_count(data, channels)?;
        if num_components < 2 {
            return Err(Error::InvalidArgument(format!(
                "num_components ({num_components}) must be >= 2"
            )));
        }
        let nc = num_components as usize;
        if nc > selected {
            return Err(Error::InvalidArgument(format!(
                "num_components ({num_components}) must be <= selected_channels.len() ({selected})"
            )));
        }
        let std_data = data.as_standard_layout().into_owned();
        let flat = build_selected_ica_buffer(&std_data, channels, selected, samples)?;
        let rows_i = checked_data_len(samples, "samples")?;
        let cols_i = checked_data_len(selected, "selected_channels")?;

        let w_size = nc * nc;
        let k_size = nc * selected;
        let a_size = selected * nc;
        let s_size = nc * samples;
        let mut w = vec![0.0_f64; w_size];
        let mut k = vec![0.0_f64; k_size];
        let mut a = vec![0.0_f64; a_size];
        let mut s = vec![0.0_f64; s_size];
        let mut data_owned = flat;
        internal::check_status(internal::perform_ica(
            data_owned.as_mut_ptr(),
            rows_i,
            cols_i,
            num_components,
            w.as_mut_ptr(),
            k.as_mut_ptr(),
            a.as_mut_ptr(),
            s.as_mut_ptr(),
        ))?;
        Ok(IcaResult {
            w: internal::array2_from_native(w, nc, nc)?,
            k: internal::array2_from_native(k, nc, selected)?,
            a: internal::array2_from_native(a, selected, nc)?,
            s: internal::array2_from_native(s, nc, samples)?,
        })
    }

    /// ICA over every column of a `(samples, channels)` 2D signal. This is
    /// a convenience wrapper around [`DataFilter::perform_ica_with_channels`]
    /// that selects all channels.
    pub fn perform_ica(data: &Array2<f64>, num_components: i32) -> Result<IcaResult> {
        let n_cols = data.ncols();
        let channels: Vec<usize> = (0..n_cols).collect();
        Self::perform_ica_with_channels(data, num_components, &channels)
    }

    /// Write a contiguous `(rows, cols)` table to disk. Rows/columns are
    /// stored transposed to match the on-disk layout the C++ helpers
    /// expect.
    pub fn write_file(data: &Array2<f64>, file_name: &str, file_mode: &str) -> Result<()> {
        if file_name.is_empty() {
            return Err(Error::InvalidArgument("file_name must not be empty".into()));
        }
        if file_mode.is_empty() {
            return Err(Error::InvalidArgument("file_mode must not be empty".into()));
        }
        let data = data.as_standard_layout();
        let (rows, cols) = data.dim();
        if rows == 0 || cols == 0 {
            return Err(Error::InvalidArgument(
                "write_file requires non-empty dimensions".into(),
            ));
        }
        let rows_i = checked_data_len(rows, "rows")?;
        let cols_i = checked_data_len(cols, "cols")?;
        let owned = data.into_owned();
        let file_name = internal::to_cstring(file_name)?;
        let file_mode = internal::to_cstring(file_mode)?;
        internal::check_status(internal::write_file(
            owned.as_ptr(),
            rows_i,
            cols_i,
            file_name.as_ptr(),
            file_mode.as_ptr(),
        ))
    }

    /// Read a file through the standard library's buffered text reader,
    /// count/validate numeric tokens with a bounded parser, then call the
    /// C ABI `read_file` with the exact element count. The shipped
    /// `read_file` rejects `num_elements = 0`, and iterative probing
    /// corrupts the SDK's std::stringstream state; using
    /// `std::fs::read_to_string` for the probe and parsing to doubles
    /// keeps the wrapper bounded without any internal helper or unsafe
    /// guess.
    pub fn read_file(file_name: &str) -> Result<Array2<f64>> {
        const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
        if file_name.is_empty() {
            return Err(Error::InvalidArgument("file_name must not be empty".into()));
        }
        let text = std::fs::read_to_string(file_name).map_err(|err| {
            Error::InvalidArgument(format!("read_to_string failed for {file_name}: {err}"))
        })?;
        if text.len() > MAX_FILE_BYTES {
            return Err(Error::BufferContract(format!(
                "read_file source exceeds {MAX_FILE_BYTES} bytes"
            )));
        }
        let (n_rows, n_cols) = count_numeric_tokens(&text)?;
        let total = internal::checked_element_count(n_rows, n_cols)?;
        if total > internal::MAX_NATIVE_FILE_ELEMENTS {
            return Err(Error::BufferContract(format!(
                "read_file would require {total} doubles, exceeding the native int bound"
            )));
        }
        let num_elements = total as i32;
        let mut data = vec![0.0_f64; total];
        let file_name_c = internal::to_cstring(file_name)?;
        let mut n_rows: i32 = 0;
        let mut n_cols: i32 = 0;
        internal::check_status(internal::read_file(
            data.as_mut_ptr(),
            &mut n_rows,
            &mut n_cols,
            file_name_c.as_ptr(),
            num_elements,
        ))?;
        if n_rows <= 0 || n_cols <= 0 {
            return Err(Error::InvalidLength);
        }
        let n_rows = usize::try_from(n_rows).map_err(|_| Error::InvalidLength)?;
        let n_cols = usize::try_from(n_cols).map_err(|_| Error::InvalidLength)?;
        let used = internal::checked_element_count(n_rows, n_cols)?;
        if used > data.len() {
            return Err(Error::BufferContract(format!(
                "read_file returned dimensions {n_rows}x{n_cols} exceeding the allocated {total} doubles"
            )));
        }
        data.truncate(used);
        internal::array2_from_native(data, n_rows, n_cols)
    }
}

fn validate_nonempty(data: &[f64], name: &str) -> Result<()> {
    if data.is_empty() {
        Err(Error::InvalidArgument(format!("{name} must not be empty")))
    } else {
        Ok(())
    }
}

fn validate_positive_i32(value: i32, name: &str) -> Result<()> {
    if value <= 0 {
        Err(Error::InvalidArgument(format!(
            "{name} must be a positive integer"
        )))
    } else {
        Ok(())
    }
}

fn validate_positive_usize(value: usize, name: &str) -> Result<()> {
    if value == 0 {
        Err(Error::InvalidArgument(format!("{name} must be positive")))
    } else {
        Ok(())
    }
}

fn validate_finite(value: f64, name: &str) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidArgument(format!(
            "{name} must be a finite number"
        )))
    }
}

fn validate_band_order(start_freq: f64, stop_freq: f64) -> Result<()> {
    if !start_freq.is_finite() || !stop_freq.is_finite() {
        return Err(Error::InvalidArgument(
            "band frequencies must be finite".into(),
        ));
    }
    if start_freq >= stop_freq {
        return Err(Error::InvalidArgument(
            "stop frequency must exceed start frequency".into(),
        ));
    }
    Ok(())
}

fn validate_stddev_range(start_pos: usize, end_pos: usize, data_len: usize) -> Result<()> {
    if start_pos >= end_pos {
        return Err(Error::InvalidArgument(format!(
            "end_pos ({end_pos}) must exceed start_pos ({start_pos})"
        )));
    }
    if end_pos > data_len {
        return Err(Error::InvalidArgument(format!(
            "end_pos ({end_pos}) must not exceed data length ({data_len})"
        )));
    }
    Ok(())
}

fn validate_equal_ppg_lengths(ir: &[f64], red: &[f64], green: Option<&[f64]>) -> Result<()> {
    if ir.is_empty() {
        return Err(Error::InvalidArgument(
            "PPG slices must not be empty".into(),
        ));
    }
    if ir.len() != red.len() {
        return Err(Error::InvalidArgument(format!(
            "ppg_red length ({}) must equal ppg_ir length ({})",
            red.len(),
            ir.len()
        )));
    }
    if let Some(green) = green
        && ir.len() != green.len()
    {
        return Err(Error::InvalidArgument(format!(
            "ppg_green length ({}) must equal ppg_ir length ({})",
            green.len(),
            ir.len()
        )));
    }
    Ok(())
}

fn checked_data_len(len: usize, name: &str) -> Result<i32> {
    if len == 0 {
        return Err(Error::InvalidArgument(format!("{name} must not be empty")));
    }
    internal::checked_native_capacity(len).map_err(|_| {
        Error::BufferContract(format!("{name} length ({len}) exceeds native int capacity"))
    })
}

fn copy_output_into(target: &mut [f64], source: &[f64]) {
    let count = target.len().min(source.len());
    target[..count].copy_from_slice(&source[..count]);
}

/// Select channel rows from an `(channels, samples)` Array2 and copy them
/// into a contiguous `Vec<f64>` laid out as `[channels.len(), samples]`
/// matching the SDK's row-major expectation. Validates that indices are
/// strictly increasing (no duplicates) and in range, returns the selected
/// flat vector plus the unchanged sample count.
fn select_channel_rows(data: &Array2<f64>, channels: &[usize]) -> Result<(Vec<f64>, usize)> {
    let (n_channels_in, n_samples) = data.dim();
    if n_channels_in == 0 || n_samples == 0 {
        return Err(Error::InvalidArgument(
            "data array must be non-empty in both dims".into(),
        ));
    }
    if channels.is_empty() {
        return Err(Error::InvalidArgument(
            "channel indices must not be empty".into(),
        ));
    }
    let selected = channels.len();
    let mut flat = Vec::with_capacity(selected * n_samples);
    let mut last: Option<usize> = None;
    for &index in channels {
        if index >= n_channels_in {
            return Err(Error::InvalidArgument(format!(
                "channel index {index} must be less than {n_channels_in}"
            )));
        }
        if let Some(prev) = last {
            if index == prev {
                return Err(Error::InvalidArgument(format!(
                    "duplicate channel index {index} in channels list"
                )));
            }
            if index < prev {
                return Err(Error::InvalidArgument(format!(
                    "channel indices must be strictly increasing; saw {prev} then {index}"
                )));
            }
        }
        last = Some(index);
        // Copy the row by value so the resulting Vec owns its data; this
        // also avoids depending on the source Array2's stride/contiguity.
        let row = data.row(index).to_vec();
        if row.len() != n_samples {
            return Err(Error::InvalidLength);
        }
        flat.extend_from_slice(&row);
    }
    Ok((flat, n_samples))
}

/// Same semantics as [`select_channel_rows`] for the ICA path that needs
/// to validate `num_components` and samples together.
fn ica_selected_row_count(data: &Array2<f64>, channels: &[usize]) -> Result<(usize, usize)> {
    let data = data.as_standard_layout();
    let (n_channels_in, n_samples) = data.dim();
    if n_channels_in == 0 || n_samples < 2 {
        return Err(Error::InvalidArgument(format!(
            "ICA requires non-empty data with at least two samples per channel (samples={n_samples})"
        )));
    }
    if channels.is_empty() {
        return Err(Error::InvalidArgument(
            "channel indices must not be empty".into(),
        ));
    }
    let mut last: Option<usize> = None;
    for &index in channels {
        if index >= n_channels_in {
            return Err(Error::InvalidArgument(format!(
                "channel index {index} must be less than {n_channels_in}"
            )));
        }
        if let Some(prev) = last
            && index <= prev
        {
            return Err(Error::InvalidArgument(format!(
                "channel indices must be strictly increasing; saw {prev} then {index}"
            )));
        }
        last = Some(index);
    }
    Ok((channels.len(), n_samples))
}

fn build_selected_ica_buffer(
    data: &Array2<f64>,
    channels: &[usize],
    selected: usize,
    samples: usize,
) -> Result<Vec<f64>> {
    let total = selected
        .checked_mul(samples)
        .ok_or(Error::IntegerOverflow)?;
    let mut flat = Vec::with_capacity(total);
    for &index in channels {
        let row = data.row(index);
        let row_slice = row.as_slice().ok_or(Error::InvalidLength)?;
        if row_slice.len() != samples {
            return Err(Error::InvalidLength);
        }
        flat.extend_from_slice(row_slice);
    }
    Ok(flat)
}

fn validate_even_length(len: usize, name: &str) -> Result<()> {
    if !len.is_multiple_of(2) {
        return Err(Error::InvalidArgument(format!(
            "{name} length ({len}) must be even"
        )));
    }
    Ok(())
}

fn validate_nonempty_array1<T>(data: &Array1<T>, name: &str) -> Result<()> {
    if data.is_empty() {
        Err(Error::InvalidArgument(format!(
            "{name} array must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn complex_array_from_parts(re: Vec<f64>, im: Vec<f64>) -> Array1<Complex<f64>> {
    let mut buffer = Vec::with_capacity(re.len());
    for (real, imag) in re.into_iter().zip(im) {
        buffer.push(Complex::new(real, imag));
    }
    Array1::from_vec(buffer)
}

fn complex_split(data: &Array1<Complex<f64>>) -> (Vec<f64>, Vec<f64>) {
    let mut re = Vec::with_capacity(data.len());
    let mut im = Vec::with_capacity(data.len());
    for value in data.iter() {
        re.push(value.re);
        im.push(value.im);
    }
    (re, im)
}

/// Floor division that rejects zero output length before any FFI call.
fn downsampled_len(data_len: usize, period: usize) -> Result<usize> {
    let value = data_len.checked_div(period).ok_or(Error::IntegerOverflow)?;
    if value == 0 {
        return Err(Error::InvalidArgument(
            "downsampling produces an empty output".into(),
        ));
    }
    Ok(value)
}

/// Count numeric tokens in a bounded whitespace-separated text buffer, enforcing
/// that every non-empty line has the same column count. Used by
/// [`DataFilter::read_file`] before allocating the native output buffer. The
/// SDK writes files with arbitrary whitespace separators (tabs by default and
/// commas for the BrainFlow/MindRove Python examples), so we accept any run
/// of ` `, `\t`, `\n` (already split), `,`, `;`, or `|` between tokens.
fn count_numeric_tokens(text: &str) -> Result<(usize, usize)> {
    let mut rows = 0_usize;
    let mut cols = 0_usize;
    let mut first_columns: Option<usize> = None;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line
            .split(|c: char| c == ',' || c == ';' || c == '|' || c.is_whitespace())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if tokens.is_empty() {
            return Err(Error::InvalidLength);
        }
        let mut count = 0_usize;
        for token in &tokens {
            if token.parse::<f64>().is_err() {
                return Err(Error::InvalidLength);
            }
            count = count.checked_add(1).ok_or(Error::IntegerOverflow)?;
        }
        match first_columns {
            None => {
                first_columns = Some(count);
                cols = count;
            }
            Some(expected) => {
                if count != expected {
                    return Err(Error::InvalidLength);
                }
            }
        }
        rows = rows.checked_add(1).ok_or(Error::IntegerOverflow)?;
    }
    if rows == 0 || cols == 0 {
        return Err(Error::InvalidLength);
    }
    Ok((rows, cols))
}

fn validate_filled_length(reported: usize, expected: usize, operation: &str) -> Result<()> {
    if reported != expected {
        return Err(Error::BufferContract(format!(
            "{operation} native output length ({reported}) does not match computed length ({expected})"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn existing_sampling_rate() -> i32 {
        250
    }

    #[test]
    fn empty_data_is_rejected_before_ffi() {
        let mut data: [f64; 0] = [];
        assert!(matches!(
            DataFilter::perform_lowpass(
                &mut data,
                existing_sampling_rate(),
                40.0,
                4,
                FilterType::BUTTERWORTH,
                0.0
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::detrend(&mut data, DetrendOperation::LINEAR),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_railed_percentage(&data, 1),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::calc_stddev(&data, 0, 0),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_wavelet_denoising(
                &mut data,
                WaveletType::DB3,
                1,
                WaveletDenoisingType::SURESHRINK,
                ThresholdType::SOFT,
                WaveletExtensionType::SYMMETRIC,
                NoiseEstimationLevelType::FIRST_LEVEL,
            ),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn non_positive_sampling_rates_and_periods_are_rejected() {
        let mut data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::perform_lowpass(&mut data, 0, 40.0, 4, FilterType::BUTTERWORTH, 0.0),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_lowpass(
                &mut data,
                existing_sampling_rate(),
                40.0,
                0,
                FilterType::BUTTERWORTH,
                0.0
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_rolling_filter(&mut data, 0, AggOperation::MEAN),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::remove_environmental_noise(&mut data, -1, NoiseType::FIFTY),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn invalid_band_ordering_is_rejected() {
        let mut data = [1.0, 2.0, 3.0, 4.0];
        let result = DataFilter::perform_bandpass(
            &mut data,
            existing_sampling_rate(),
            40.0,
            10.0,
            4,
            FilterType::BUTTERWORTH,
            0.0,
        );
        assert!(matches!(result, Err(Error::InvalidArgument(_))));

        let result = DataFilter::perform_bandstop(
            &mut data,
            existing_sampling_rate(),
            f64::NAN,
            40.0,
            4,
            FilterType::BUTTERWORTH,
            0.0,
        );
        assert!(matches!(result, Err(Error::InvalidArgument(_))));
    }

    #[test]
    fn finite_coefficient_validation_blocks_nan_and_infinity() {
        let mut data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::perform_lowpass(
                &mut data,
                existing_sampling_rate(),
                f64::NAN,
                4,
                FilterType::BUTTERWORTH,
                0.0
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_bandpass(
                &mut data,
                existing_sampling_rate(),
                1.0,
                f64::INFINITY,
                4,
                FilterType::BUTTERWORTH,
                0.0,
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::detect_peaks_z_score(&mut data, 4, f64::NAN, 0.5),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn mismatched_ppg_inputs_are_rejected_before_ffi() {
        let ir = [1.0, 2.0];
        let red = [1.0];
        let green = [1.0, 2.0];
        assert!(matches!(
            DataFilter::get_heart_rate(&ir, &red, &green, 250, 1024),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_rmssd_hrv(&ir, &red, &green, 250),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_oxygen_level(&ir, &red, 250),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_oxygen_level_with_coefs(&ir, &red, 250, 0.0, 0.0, 0.0),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn empty_ppg_inputs_are_rejected_before_ffi() {
        let empty: [f64; 0] = [];
        assert!(matches!(
            DataFilter::get_oxygen_level(&empty, &empty, 250),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_heart_rate(&empty, &empty, &empty, 250, 1024),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn equal_ppg_inputs_with_mismatched_green_are_rejected() {
        let ir = [1.0, 2.0, 3.0];
        let red = [1.0, 2.0, 3.0];
        let green = [1.0, 2.0];
        assert!(matches!(
            DataFilter::get_rmssd_hrv(&ir, &red, &green, 250),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn invalid_fft_size_for_heart_rate_is_rejected() {
        let ir = [1.0; 4];
        let red = [1.0; 4];
        let green = [1.0; 4];
        assert!(matches!(
            DataFilter::get_heart_rate(&ir, &red, &green, 250, 0),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_heart_rate(&ir, &red, &green, -1, 1024),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn calc_stddev_range_is_validated() {
        let data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::calc_stddev(&data, 0, 0),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::calc_stddev(&data, 2, 1),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::calc_stddev(&data, 0, 5),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn window_length_must_be_positive() {
        assert!(matches!(
            DataFilter::get_window(WindowOperation::HANNING, 0),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn peak_detection_validates_inputs() {
        let mut data: [f64; 0] = [];
        assert!(matches!(
            DataFilter::detect_peaks_z_score(&mut data, 4, 3.5, 0.5),
            Err(Error::InvalidArgument(_))
        ));

        let mut data = [1.0, 2.0, 3.0];
        assert!(matches!(
            DataFilter::detect_peaks_z_score(&mut data, 0, 3.5, 0.5),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn wavelet_restoration_validates_levels() {
        let data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::restore_data_from_wavelet_detailed_coeffs(&data, WaveletType::DB3, 0, 1,),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::restore_data_from_wavelet_detailed_coeffs(&data, WaveletType::DB3, 2, 0,),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::restore_data_from_wavelet_detailed_coeffs(&data, WaveletType::DB3, 2, 3,),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn wavelet_denoising_rejects_zero_levels() {
        let mut data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::perform_wavelet_denoising(
                &mut data,
                WaveletType::DB3,
                0,
                WaveletDenoisingType::SURESHRINK,
                ThresholdType::SOFT,
                WaveletExtensionType::SYMMETRIC,
                NoiseEstimationLevelType::FIRST_LEVEL,
            ),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn oxygen_default_helper_uses_sdk_default_coefs() {
        assert_eq!(DEFAULT_OXYGEN_COEFS, (1.5958422, -34.6596622, 112.6898759));
    }

    #[test]
    fn finite_coef_validation_rejects_non_finite_for_oxygen_level() {
        let ppg_ir = [1.0, 2.0];
        let ppg_red = [1.0, 2.0];
        assert!(matches!(
            DataFilter::get_oxygen_level_with_coefs(&ppg_ir, &ppg_red, 250, f64::NAN, 0.0, 0.0),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn fft_rejects_empty_inputs() {
        let data: [f64; 0] = [];
        assert!(matches!(
            DataFilter::perform_fft(&data, WindowOperation::HANNING),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn fft_rejects_odd_inputs() {
        let data = [1.0, 2.0, 3.0];
        assert!(matches!(
            DataFilter::perform_fft(&data, WindowOperation::HANNING),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn psd_rejects_invalid_sampling_rate_and_odd_length() {
        let data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::get_psd(&data, 0, WindowOperation::NO_WINDOW),
            Err(Error::InvalidArgument(_))
        ));
        let odd = [1.0, 2.0, 3.0];
        assert!(matches!(
            DataFilter::get_psd(&odd, 250, WindowOperation::NO_WINDOW),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn psd_welch_rejects_inconsistent_overlap() {
        let data: Vec<f64> = (0..512).map(|i| i as f64).collect();
        assert!(matches!(
            DataFilter::get_psd_welch(&data, 0, 0, 250, WindowOperation::NO_WINDOW),
            Err(Error::InvalidArgument(_))
        ));
        let odd_nfft_odd: Vec<f64> = (0..32).map(|i| i as f64).collect();
        assert!(matches!(
            DataFilter::get_psd_welch(&odd_nfft_odd, 33, 0, 250, WindowOperation::NO_WINDOW),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_psd_welch(&data, 256, 256, 250, WindowOperation::NO_WINDOW),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_psd_welch(&data, 256, -1, 250, WindowOperation::NO_WINDOW),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn band_power_rejects_mismatched_or_band_issues() {
        let ampl = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        let freq = Array1::from_vec(vec![0.0, 1.0, 2.0, 3.0]);
        assert!(matches!(
            DataFilter::get_band_power(&(ampl.clone(), freq.clone()), 1.0, 2.0),
            Err(Error::InvalidLength)
        ));
        let empty = Array1::from_vec(Vec::<f64>::new());
        assert!(matches!(
            DataFilter::get_band_power(&(empty.clone(), empty), 1.0, 2.0),
            Err(Error::InvalidArgument(_))
        ));
        let short_ampl = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        let short_freq = Array1::from_vec(vec![0.0, 1.0, 2.0]);
        assert!(matches!(
            DataFilter::get_band_power(&(short_ampl.clone(), short_freq.clone()), 2.0, 1.0),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_band_power(&(short_ampl, short_freq), 1.0, f64::NAN),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn custom_band_powers_reject_invalid_inputs() {
        let data = ndarray::Array2::<f64>::zeros((2, 4));
        let channels = [0_usize, 1];
        let bands = [FrequencyBand::new(1.0, 4.0)];
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &channels, &[], 250, false),
            Err(Error::InvalidArgument(_))
        ));
        let bad_band = [FrequencyBand::new(4.0, 1.0)];
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &channels, &bad_band, 250, false),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &channels, &bands, 0, false),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &[], &bands, 250, false),
            Err(Error::InvalidArgument(_))
        ));
        let out_of_range = [5_usize];
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &out_of_range, &bands, 250, false),
            Err(Error::InvalidArgument(_))
        ));
        let duplicates = [0_usize, 0];
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &duplicates, &bands, 250, false),
            Err(Error::InvalidArgument(_))
        ));
        let unordered = [1_usize, 0];
        assert!(matches!(
            DataFilter::get_custom_band_powers(&data, &unordered, &bands, 250, false),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::get_avg_band_powers(&data, &[], 250, false),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn downsampling_validates_period_and_data() {
        let data = [1.0, 2.0, 3.0];
        assert!(matches!(
            DataFilter::perform_downsampling(&data, 0, AggOperation::MEAN),
            Err(Error::InvalidArgument(_))
        ));
        let empty: [f64; 0] = [];
        assert!(matches!(
            DataFilter::perform_downsampling(&empty, 2, AggOperation::MEAN),
            Err(Error::InvalidArgument(_))
        ));
        let short: [f64; 2] = [1.0, 2.0];
        assert!(matches!(
            DataFilter::perform_downsampling(&short, 3, AggOperation::MEAN),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn wavelet_transform_validates_inputs() {
        let empty: [f64; 0] = [];
        assert!(matches!(
            DataFilter::perform_wavelet_transform(
                &empty,
                WaveletType::DB3,
                1,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
        let data = [1.0, 2.0, 3.0, 4.0];
        assert!(matches!(
            DataFilter::perform_wavelet_transform(
                &data,
                WaveletType::DB3,
                0,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_wavelet_transform(
                &data,
                WaveletType::DB3,
                101,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn inverse_wavelet_validates_block_consistency() {
        let coefficients = [1.0, 2.0, 3.0, 4.0];
        let mismatched = [3usize, 4usize];
        assert!(matches!(
            DataFilter::perform_inverse_wavelet_transform(
                &coefficients,
                &mismatched,
                4,
                WaveletType::DB3,
                1,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
        let wrong_levels = [4usize];
        assert!(matches!(
            DataFilter::perform_inverse_wavelet_transform(
                &coefficients,
                &wrong_levels,
                4,
                WaveletType::DB3,
                1,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_inverse_wavelet_transform(
                &[],
                &[1usize],
                4,
                WaveletType::DB3,
                1,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_inverse_wavelet_transform(
                &coefficients,
                &[1usize, 2usize, 1usize],
                0,
                WaveletType::DB3,
                2,
                WaveletExtensionType::SYMMETRIC,
            ),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn csp_validates_input_shape_and_labels() {
        let data = Array3::<f64>::zeros((0, 2, 4));
        let labels = Array1::<f64>::zeros(0);
        assert!(matches!(
            DataFilter::get_csp(&data, &labels),
            Err(Error::InvalidArgument(_))
        ));
        let data = Array3::<f64>::zeros((2, 4, 8));
        let labels = Array1::<f64>::zeros(4);
        assert!(matches!(
            DataFilter::get_csp(&data, &labels),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn ica_validates_components_and_data() {
        let empty = Array2::<f64>::zeros((0, 4));
        let channels = [0_usize, 1, 2, 3];
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&empty, 3, &channels),
            Err(Error::InvalidArgument(_))
        ));
        let data = Array2::<f64>::zeros((100, 4));
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&data, 0, &channels),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&data, 6, &channels),
            Err(Error::InvalidArgument(_))
        ));
        let one_sample = Array2::<f64>::zeros((1, 4));
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&one_sample, 3, &channels),
            Err(Error::InvalidArgument(_))
        ));
        let empty_channels: [usize; 0] = [];
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&data, 3, &empty_channels),
            Err(Error::InvalidArgument(_))
        ));
        let bad = [0_usize, 7];
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&data, 3, &bad),
            Err(Error::InvalidArgument(_))
        ));
        let duplicates = [0_usize, 0];
        assert!(matches!(
            DataFilter::perform_ica_with_channels(&data, 1, &duplicates),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn file_methods_validate_input_strings() {
        assert!(matches!(
            DataFilter::write_file(&Array2::<f64>::zeros((2, 3)), "", "w",),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            DataFilter::read_file(""),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn read_file_rejects_missing_or_empty() {
        let unique = std::env::temp_dir().join("mindrove_empty.csv");
        let path_str = unique.to_string_lossy().to_string();
        std::fs::write(&unique, b"").unwrap();
        assert!(matches!(
            DataFilter::read_file(&path_str),
            Err(Error::InvalidLength)
        ));
        let _ = std::fs::remove_file(&unique);
    }

    #[test]
    fn read_file_rejects_malformed_csv() {
        let unique = std::env::temp_dir().join("mindrove_malformed.csv");
        let path_str = unique.to_string_lossy().to_string();
        std::fs::write(&unique, b"1.0,oops\n4.0,5.0\n").unwrap();
        assert!(matches!(
            DataFilter::read_file(&path_str),
            Err(Error::InvalidLength)
        ));
        let _ = std::fs::remove_file(&unique);
    }

    #[test]
    fn read_file_rejects_uneven_columns() {
        let unique = std::env::temp_dir().join("mindrove_uneven.csv");
        let path_str = unique.to_string_lossy().to_string();
        std::fs::write(&unique, b"1.0,2.0,3.0\n4.0,5.0\n").unwrap();
        assert!(matches!(
            DataFilter::read_file(&path_str),
            Err(Error::InvalidLength)
        ));
        let _ = std::fs::remove_file(&unique);
    }

    #[test]
    fn write_file_validates_dimensions() {
        let empty = Array2::<f64>::zeros((0, 4));
        assert!(matches!(
            DataFilter::write_file(&empty, "/tmp/empty.csv", "w"),
            Err(Error::InvalidArgument(_))
        ));
        let empty_rows = Array2::<f64>::zeros((2, 0));
        assert!(matches!(
            DataFilter::write_file(&empty_rows, "/tmp/empty_rows.csv", "w"),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn ifft_rejects_empty_and_short_input() {
        let empty = Array1::<num_complex::Complex<f64>>::from_vec(Vec::new());
        assert!(matches!(
            DataFilter::perform_ifft(&empty),
            Err(Error::InvalidArgument(_))
        ));
        let tiny = Array1::<num_complex::Complex<f64>>::from_vec(vec![num_complex::Complex::new(
            1.0, 0.0,
        )]);
        assert!(matches!(
            DataFilter::perform_ifft(&tiny),
            Err(Error::InvalidArgument(_))
        ));
    }
}
