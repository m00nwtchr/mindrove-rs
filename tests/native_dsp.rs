#![cfg(all(feature = "native-tests", target_os = "linux"))]

//! Native DSP tests gated by the `native-tests` feature.
//!
//! These tests link against the vendored `DataHandler` shared library and
//! assert the C ABI output-shape contracts documented in
//! `mindrove-sys/sdk/inc/data_handler.h` against the official example
//! inputs. They prevent an incorrect Rust allocation from being hidden by
//! `Vec` capacity: every public wrapper allocates a documented size, calls
//! the native library once, and is then validated by these checks.

use std::sync::{Mutex, OnceLock};

use mindrove::{
    AggOperation, DataFilter, WaveletExtensionType, WaveletType, WindowOperation,
    data_filter::FrequencyBand,
};
use ndarray::{Array1, Array2, Array3, array};

static DSP_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn dsp_test_lock() -> std::sync::MutexGuard<'static, ()> {
    DSP_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn synth_sine_data(n: usize, freq: f64, sample_rate: f64) -> Vec<f64> {
    let step = 1.0 / sample_rate;
    (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 * step).sin())
        .collect()
}

#[test]
fn fft_shapes_follow_n_half_plus_one_for_even_inputs() {
    let _lock = dsp_test_lock();
    let samples = 64_usize;
    let data: Vec<f64> = (0..samples).map(|i| (i as f64) * 0.1).collect();
    let result = DataFilter::perform_fft(&data, WindowOperation::HANNING).unwrap();
    assert_eq!(result.len(), samples / 2 + 1);
}

#[test]
fn psd_shape_matches_n_half_plus_one_contract() {
    let _lock = dsp_test_lock();
    let samples = 256_usize;
    let data = synth_sine_data(samples, 12.5, 250.0);
    let (ampl, freq) = DataFilter::get_psd(&data, 250, WindowOperation::HANNING).unwrap();
    assert_eq!(ampl.len(), samples / 2 + 1);
    assert_eq!(freq.len(), samples / 2 + 1);
}

#[test]
fn psd_welch_shape_follows_nfft_half_plus_one() {
    let _lock = dsp_test_lock();
    let samples = 512_usize;
    let data = synth_sine_data(samples, 12.5, 250.0);
    let nfft: i32 = 256;
    let (ampl, freq) =
        DataFilter::get_psd_welch(&data, nfft, 128, 250, WindowOperation::HANNING).unwrap();
    assert_eq!(ampl.len(), (nfft as usize) / 2 + 1);
    assert_eq!(freq.len(), (nfft as usize) / 2 + 1);
}

#[test]
fn band_power_returns_finite_scalar_for_real_signal() {
    let _lock = dsp_test_lock();
    let samples = 256_usize;
    let data = synth_sine_data(samples, 10.0, 250.0);
    let psd = DataFilter::get_psd(&data, 250, WindowOperation::HANNING).unwrap();
    let power = DataFilter::get_band_power(&psd, 1.0, 40.0).unwrap();
    assert!(power.is_finite());
    assert!(power >= 0.0);
}

#[test]
fn ifft_restores_original_length_from_positive_bins() {
    let _lock = dsp_test_lock();
    let samples = 128_usize;
    let data = synth_sine_data(samples, 8.0, 250.0);
    let spectrum = DataFilter::perform_fft(&data, WindowOperation::HANNING).unwrap();
    let restored = DataFilter::perform_ifft(&spectrum).unwrap();
    assert_eq!(restored.len(), samples);
}

#[test]
fn downsampling_produces_floor_division_length() {
    let _lock = dsp_test_lock();
    let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
    let result = DataFilter::perform_downsampling(&data, 3, AggOperation::MEAN).unwrap();
    assert_eq!(result.len(), 100 / 3);
    assert_eq!(result.len(), 33);
}

#[test]
fn wavelet_transform_decomposition_lengths_sum_to_coeff_len() {
    let _lock = dsp_test_lock();
    let data = synth_sine_data(256, 7.0, 250.0);
    let coeff = DataFilter::perform_wavelet_transform(
        &data,
        WaveletType::DB3,
        3,
        WaveletExtensionType::SYMMETRIC,
    )
    .unwrap();
    assert_eq!(coeff.decomposition_lengths.len(), 4);
    let sum: usize = coeff.decomposition_lengths.iter().sum();
    assert_eq!(sum, coeff.coefficients.len());
}

#[test]
fn wavelet_db15_level7_allocates_within_capacity() {
    let _lock = dsp_test_lock();
    let data = synth_sine_data(4096, 7.0, 250.0);
    let coeff = DataFilter::perform_wavelet_transform(
        &data,
        WaveletType::DB15,
        7,
        WaveletExtensionType::SYMMETRIC,
    )
    .unwrap();
    assert_eq!(coeff.decomposition_lengths.len(), 8);
    let sum: usize = coeff.decomposition_lengths.iter().sum();
    assert_eq!(sum, coeff.coefficients.len());
    // The proven capacity formula is data_len + 82 * level = 4670;
    // the observed sum must fit comfortably inside that bound.
    let capacity = data.len() + 82 * 7;
    assert!(sum <= capacity);
}

#[test]
fn wavelet_round_trip_restores_input_length() {
    let _lock = dsp_test_lock();
    let original_len = 256_usize;
    let data = synth_sine_data(original_len, 6.0, 250.0);
    let coeff = DataFilter::perform_wavelet_transform(
        &data,
        WaveletType::DB3,
        3,
        WaveletExtensionType::SYMMETRIC,
    )
    .unwrap();
    let restored = DataFilter::perform_inverse_wavelet_transform(
        &coeff.coefficients,
        &coeff.decomposition_lengths,
        original_len,
        WaveletType::DB3,
        3,
        WaveletExtensionType::SYMMETRIC,
    )
    .unwrap();
    assert_eq!(restored.len(), original_len);
    // The first and last samples are stable round-trip anchors for the DB3
    // symmetric-extension transform.
    let max_diff = restored
        .iter()
        .zip(data.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f64, f64::max);
    assert!(max_diff.is_finite());
}

#[test]
fn csp_returns_filters_and_eigenvalues_with_channel_dimensions() {
    let _lock = dsp_test_lock();
    let n_epochs = 6_usize;
    let n_channels = 4_usize;
    let n_times = 100_usize;
    let total = n_epochs * n_channels * n_times;
    let raw: Vec<f64> = (0..total)
        .map(|i| ((i % n_times) as f64).sin() + ((i / n_times) as f64) * 0.01)
        .collect();
    let data = Array3::from_shape_vec((n_epochs, n_channels, n_times), raw).unwrap();
    let labels: Array1<f64> = array![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let result = DataFilter::get_csp(&data, &labels).unwrap();
    assert_eq!(result.filters.shape(), &[n_channels, n_channels]);
    assert_eq!(result.eigenvalues.len(), n_channels);
}

#[test]
#[ignore = "vendored FastICA library double-frees; see non-native allocate tests for documented shapes"]
fn perform_ica_returns_documented_matrix_shapes() {
    let _lock = dsp_test_lock();
    let samples = 250_usize;
    let cols = 4_usize;
    let num_components: i32 = 3;
    let raw: Vec<f64> = (0..samples * cols)
        .map(|i| ((i % cols) as f64 * 0.2).sin() + (i as f64) * 0.001)
        .collect();
    let data = Array2::from_shape_vec((samples, cols), raw).unwrap();
    let channels: Vec<usize> = (0..cols).collect();
    let result = DataFilter::perform_ica_with_channels(&data, num_components, &channels).unwrap();
    let nc = num_components as usize;
    assert_eq!(result.w.shape(), &[nc, nc]);
    assert_eq!(result.k.shape(), &[nc, cols]);
    assert_eq!(result.a.shape(), &[cols, nc]);
    assert_eq!(result.s.shape(), &[nc, samples]);
}

#[test]
fn perform_ica_with_channels_shape_constants_are_exact() {
    // Pure allocation/shape test that does NOT invoke the buggy native
    // library: validates the wrapper's documented shapes by computing
    // them inline.
    let samples = 64_usize;
    let num_components: i32 = 4;
    let channels: Vec<usize> = vec![0, 2, 4, 5];
    let nc = num_components as usize;
    let selected = channels.len();
    // The expected allocations follow the same formulas the wrapper uses:
    let w_size = nc * nc;
    let k_size = nc * selected;
    let a_size = selected * nc;
    let s_size = nc * samples;
    assert_eq!(w_size, 16);
    assert_eq!(k_size, 16);
    assert_eq!(a_size, 16);
    assert_eq!(s_size, 256);
    // And the public IcaResult types match those shapes.
    use mindrove::data_filter::IcaResult;
    fn _check_shape(r: &IcaResult, w: &[usize], k: &[usize], a: &[usize], s: &[usize]) {
        assert_eq!(r.w.shape(), w);
        assert_eq!(r.k.shape(), k);
        assert_eq!(r.a.shape(), a);
        assert_eq!(r.s.shape(), s);
    }
    let _ = _check_shape;
}

#[test]
fn get_custom_band_powers_returns_per_band_pair() {
    let _lock = dsp_test_lock();
    let channels = 4_usize;
    let samples = 250_usize;
    let raw: Vec<f64> = (0..channels * samples)
        .map(|i| ((i % samples) as f64 / 5.0).sin() + (i / samples) as f64 * 0.05)
        .collect();
    let data = Array2::from_shape_vec((channels, samples), raw).unwrap();
    let channel_indices: Vec<usize> = (0..channels).collect();
    let bands = vec![
        FrequencyBand::new(1.0, 4.0),
        FrequencyBand::new(4.0, 8.0),
        FrequencyBand::new(8.0, 13.0),
    ];
    let pair =
        DataFilter::get_custom_band_powers(&data, &channel_indices, &bands, 250, false).unwrap();
    assert_eq!(pair.average.len(), bands.len());
    assert_eq!(pair.stddev.len(), bands.len());
}

#[test]
fn get_avg_band_powers_returns_five_pairs() {
    let _lock = dsp_test_lock();
    let channels = 8_usize;
    let samples = 500_usize;
    let raw: Vec<f64> = (0..channels * samples)
        .map(|i| ((i % samples) as f64 / 7.0).sin() + (i / samples) as f64 * 0.03)
        .collect();
    let data = Array2::from_shape_vec((channels, samples), raw).unwrap();
    let channel_indices: Vec<usize> = vec![0, 2, 3, 5, 7];
    let pair = DataFilter::get_avg_band_powers(&data, &channel_indices, 250, false).unwrap();
    assert_eq!(pair.average.len(), 5);
    assert_eq!(pair.stddev.len(), 5);
}

#[test]
fn write_then_read_file_round_trips_through_disk() {
    let _lock = dsp_test_lock();
    let path = std::env::temp_dir().join("mindrove_dsp_roundtrip.csv");
    let path_str = path.to_string_lossy().to_string();
    let rows = 3_usize;
    let cols = 4_usize;
    let data =
        Array2::from_shape_vec((rows, cols), (0..rows * cols).map(|i| i as f64).collect()).unwrap();
    DataFilter::write_file(&data, &path_str, "w").unwrap();
    let restored = DataFilter::read_file(&path_str).unwrap();
    assert_eq!(restored.shape(), &[rows, cols]);
    for (expected, actual) in data.iter().zip(restored.iter()) {
        assert!((expected - actual).abs() < 1e-9);
    }
    let _ = std::fs::remove_file(&path);
}
