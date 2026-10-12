# MindRove Safe Rust Abstraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a safe, idiomatic Rust façade over the MindRove C ABI that mirrors the public `BoardShim`, `DataFilter`, and `MLModel` C++ SDK API.

**Architecture:** Keep `mindrove-sys` as a generated raw binding crate for the four C ABI headers. Implement the public façade in focused Rust modules with typed errors, serde JSON parameters, checked buffer helpers, RAII cleanup, and `ndarray` data values. Use feature-gated native tests against the SDK's synthetic board and official DSP examples.

**Tech Stack:** Rust 2024, bindgen 0.71, `ndarray` 0.16, `num-complex` 0.4, `serde` 1 with derive, `serde_json` 1, `thiserror` 2, vendored MindRove `BoardController`, `DataHandler`, and `MLModule` libraries.

**Spec:** `docs/superpowers/specs/2026-10-11-mindrove-safe-rust-abstraction-design.md`

## Global Constraints

- Use the SDK's C ABI as the FFI boundary. Do not bind the C++ classes directly.
- Implement the complete public `BoardShim`, `DataFilter`, and `MLModel` surface from the vendored headers and official examples.
- Omit JNI integration and the explicitly internal C function `get_num_elements_in_file`.
- Use `ndarray::Array1`, `Array2`, and `Array3` for public numeric arrays.
- Preserve the SDK's canonical data shapes: board data is channels × samples; epoch data is epochs × channels × samples.
- Keep raw pointers, C strings, native output buffers, and unsafe calls private to the FFI adapter.
- Every native nonzero status becomes a public `Result` error; no status is silently ignored.
- Public numeric results own their memory before crossing the safe API boundary.
- Board data and multidimensional DSP inputs must be standard-layout/contiguous before entering FFI.
- Native integration tests are opt-in through the `native-tests` feature and target Linux SDK libraries.
- Do not create commits unless the user explicitly requests them.

---

## File map

| Path | Responsibility |
| --- | --- |
| `Cargo.toml` | Safe-wrapper dependencies and `native-tests` feature. |
| `src/lib.rs` | Module declarations and public re-exports; remove the obsolete test. |
| `src/error.rs` | Typed native status and Rust-side error enum. |
| `src/types.rs` | SDK enums, board/preset value types, and serde parameter structs. |
| `src/internal/mod.rs` | Private shared adapter exports. |
| `src/internal/status.rs` | Status-code conversion and checked integer helpers. |
| `src/internal/json.rs` | Exact SDK JSON serialization and C-string conversion. |
| `src/internal/buffers.rs` | Probe/fill, checked allocation, native-to-ndarray conversion. |
| `src/board_shim.rs` | Safe board metadata, lifecycle, streaming, and data access. |
| `src/data_filter.rs` | Safe static DSP, spectral, PPG, file, CSP, and ICA methods. |
| `src/ml_model.rs` | Safe ML model lifecycle and prediction. |
| `mindrove-sys/wrapper.h` | The four C ABI includes only. |
| `mindrove-sys/build.rs` | Native link directives, bindgen invalidation, and Linux runtime search path. |
| `tests/types.rs` | Parameter defaults, exact JSON, and enum discriminants. |
| `tests/error.rs` | Native status conversion and unknown-code behavior. |
| `tests/internal.rs` | Checked shape/length and buffer-helper behavior. |
| `tests/native_board.rs` | Feature-gated synthetic-board lifecycle and board-data checks. |
| `tests/native_dsp.rs` | Feature-gated official-example DSP coverage. |
| `tests/native_ml.rs` | Feature-gated ML coverage with explicit asset skip behavior. |
| `examples/synthetic_acquisition.rs` | A documented Rust equivalent of the official synthetic C++ example. |

The safe implementation is intentionally split by public responsibility. The internal modules are the only place allowed to call `mindrove_sys` directly.

## Task 1: Expand and verify the raw C ABI crate

**Files:**
- Modify: `mindrove-sys/wrapper.h`
- Modify: `mindrove-sys/build.rs`
- Verify: generated `OUT_DIR/bindings.rs` (not committed)

**Interfaces:**
- Produces raw bindings for `board_controller.h`, `board_info_getter.h`, `data_handler.h`, and `ml_module.h`.
- Produces linkable symbols named exactly as declared by those headers, including `prepare_session`, `get_board_descr`, `perform_fft`, `perform_ica`, `prepare`, and `predict`.

- [ ] **Step 1: Expand the bindgen header without importing C++ classes.**

Keep the existing board includes and add exactly:

```c
#include <data_handler.h>
#include <ml_module.h>
```

Do not include `board_shim.h`, `data_filter.h`, `ml_model.h`, `mindrove_array.h`, `mindrove_exception.h`, or `json.hpp`; those are C++ implementation types and are outside the FFI boundary.

- [ ] **Step 2: Add the three native shared-library link directives.**

In `mindrove-sys/build.rs`, retain the existing native search path and `BoardController` link, then add:

```rust
println!("cargo:rustc-link-lib=dylib=DataHandler");
println!("cargo:rustc-link-lib=dylib=MLModule");
```

Do not link `MindRoveWrapper` or the static archives speculatively: none of the four selected C headers requires their symbols. If `ldd` or the linker reports a missing transitive symbol, add only the specific archive proven necessary and record that native dependency in the build script comment.

- [ ] **Step 3: Make header changes invalidate bindgen.**

Add explicit build-script directives for the wrapper and four C headers:

```rust
println!("cargo:rerun-if-changed=wrapper.h");
println!("cargo:rerun-if-changed=sdk/inc/board_controller.h");
println!("cargo:rerun-if-changed=sdk/inc/board_info_getter.h");
println!("cargo:rerun-if-changed=sdk/inc/data_handler.h");
println!("cargo:rerun-if-changed=sdk/inc/ml_module.h");
```

- [ ] **Step 4: Verify the raw crate before writing safe code.**

Run:

```text
cargo clean -p mindrove-sys
cargo build -p mindrove-sys
cargo build
```

Expected: bindgen completes and the generated file contains the four C-ABI families without C++ `MindRoveArray` or class declarations. If linking fails, use `ldd mindrove-sys/sdk/lib/libDataHandler.so`, `ldd mindrove-sys/sdk/lib/libMLModule.so`, and `nm -D` on the reported symbol before changing link directives.

## Task 2: Add dependencies and implement core types/errors

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/types.rs`
- Create: `tests/types.rs`
- Create: `tests/error.rs`

**Interfaces:**
- Produces `pub enum Error` and `pub type Result<T> = std::result::Result<T, Error>`.
- Produces named `thiserror` variants for native SDK failures, `BoardId`, `MindRovePreset`, all SDK option enums, `MindRoveInputParams`, and `MindRoveModelParams`.
- Produces serde JSON strings whose keys exactly match the sixteen input/model parameter field names in the SDK headers.

- [ ] **Step 1: Declare the exact safe-wrapper dependencies and feature.**

Add:

```toml
[dependencies]
mindrove-sys = { path = "./mindrove-sys" }
ndarray = { version = "0.16", features = ["serde"] }
num-complex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[features]
native-tests = []
```

- [ ] **Step 2: Write failing type and serialization tests.**

Create tests asserting defaults, all native discriminants, and exact JSON:

```rust
#[test]
fn input_defaults_match_sdk() {
    let params = MindRoveInputParams::default();
    assert_eq!(params.ip_port, -1);
    assert_eq!(params.master_board, BoardId::NO_BOARD);
    assert_eq!(params.serial_port, "");
}

#[test]
fn input_json_uses_sdk_field_names() {
    let params = MindRoveInputParams::default();
    let value: serde_json::Value = serde_json::from_str(&params.to_json().unwrap()).unwrap();
    assert!(value.get("ip_address_aux").is_some());
    assert!(value.get("master_board").is_some());
}

#[test]
fn discriminants_match_headers() {
    assert_eq!(FilterType::BUTTERWORTH as i32, 0);
    assert_eq!(FilterType::BESSEL_ZERO_PHASE as i32, 5);
    assert_eq!(WaveletType::SYM10 as i32, 44);
}
```

Also add an error test:

```rust
#[test]
fn unknown_status_becomes_typed_error() {
    assert!(matches!(
        Error::from_status(9999),
        Err(Error::UnknownNativeStatus(9999))
    ));
}
```

Run:

```text
cargo test --test types --test error
```

Expected: the new tests fail because the public types do not exist yet.

- [ ] **Step 3: Implement `src/error.rs`.**

Use `thiserror` and convert native statuses immediately into named errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // One named variant per SDK status code, plus UnknownNativeStatus.
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

pub type Result<T> = std::result::Result<T, Error>;
```

Implement `Error::from_status(i32)` as a complete match: `0` is success, statuses `1..=23` map to their named variants, and unknown integers map to `Error::UnknownNativeStatus(raw)`, preserving the original status code for diagnostics.

- [ ] **Step 4: Implement `src/types.rs` and `src/lib.rs` exports.**

Use exact SDK values for `BoardId`, `MindRovePreset`, `IpProtocolType`, `FilterType`, `AggOperation`, `WindowOperation`, `DetrendOperation`, `MindRoveMetric`, `MindRoveClassifier`, `LogLevel`, `NoiseType`, `WaveletDenoisingType`, `ThresholdType`, `WaveletExtensionType`, `NoiseEstimationLevelType`, and `WaveletType`. Use a board-ID newtype with these associated constants:

```rust
pub const NO_BOARD: BoardId = BoardId(-100);
pub const PLAYBACK_FILE_BOARD: BoardId = BoardId(-3);
pub const STREAMING_BOARD: BoardId = BoardId(-2);
pub const SYNTHETIC_BOARD: BoardId = BoardId(-1);
pub const MINDROVE_WIFI_BOARD: BoardId = BoardId(0);
pub const MINDROVE_SYNCBOX_BOARD: BoardId = BoardId(1);
```

Implement `Default` for both parameter structs with the header defaults, `Serialize`/`Deserialize`, and a `to_json() -> Result<String>` method. Declare the future public modules in `src/lib.rs` but keep their contents empty until their tasks:

```rust
pub mod board_shim;
pub mod data_filter;
pub mod error;
pub mod ml_model;
pub mod types;

pub use error::{Error, Result};
pub use types::*;
```

Remove the obsolete `add(2, 2)` test.

- [ ] **Step 5: Run the core tests and formatting.**

Run:

```text
cargo fmt --check
cargo test --test types --test error
```

Expected: PASS, with exact values and JSON keys matching `mindrove_constants.h`, `mindrove_input_params.h`, and `mindrove_model_params.h`.

## Task 3: Build private FFI status, JSON, and buffer helpers

**Files:**
- Create: `src/internal/mod.rs`
- Create: `src/internal/status.rs`
- Create: `src/internal/json.rs`
- Create: `src/internal/buffers.rs`
- Create: `tests/internal.rs`

**Interfaces:**
- Consumes: `Error`, `Result`, parameter structs, and generated `mindrove_sys` functions.
- Produces private `check_status`, `to_cstring`, checked integer/shape conversion, string probe/fill, integer-array fill, and native-array conversion helpers.

- [ ] **Step 1: Write failing helper tests.**

Cover overflow and native-length rejection before calling real SDK code:

```rust
#[test]
fn shape_overflow_is_rejected() {
    assert_eq!(checked_element_count(i32::MAX as usize, 2), Err(Error::IntegerOverflow));
}

#[test]
fn negative_native_length_is_rejected() {
    assert_eq!(checked_native_len(-1), Err(Error::InvalidLength));
}

#[test]
fn interior_nul_is_rejected() {
    assert!(matches!(to_cstring("bad\0value"), Err(Error::InteriorNul)));
}
```

Run `cargo test --test internal`; expected failure is missing helpers.

- [ ] **Step 2: Implement status and checked-size helpers.**

Use `usize::checked_mul`, `usize::checked_add`, and `i32::try_from`. Never cast a `usize` to the C `int` length without checking it. Convert negative output lengths to `Error::InvalidLength`.

- [ ] **Step 3: Implement JSON/C-string helpers.**

Serialize parameters with `serde_json::to_string`, then use `CString::new`. Convert `NulError` to `Error::InteriorNul`. Keep the resulting `CString` owned by `BoardShim`/`MLModel` so every FFI call receives a stable pointer for that call.

- [ ] **Step 4: Implement bounded probe/fill helpers.**

For SDK string getters, first call with a null output buffer when the C ABI supports its length-probe convention, then allocate and call again. Use a hard maximum of `1 << 20` bytes; return `Error::BufferContract` if a native length exceeds it, if the second call reports a larger length, or if the returned bytes are not UTF-8. Apply the same pattern to channel/preset integer arrays using a checked `i32` capacity.

Implement helpers with closures so public modules cannot accidentally bypass status conversion:

```rust
pub(crate) fn read_string<F>(call: F) -> Result<String>
where
    F: Fn(*mut c_char, *mut c_int) -> c_int;

pub(crate) fn read_i32_vec<F>(call: F) -> Result<Vec<i32>>
where
    F: Fn(*mut i32, *mut c_int) -> c_int;
```

- [ ] **Step 5: Implement ndarray conversion helpers.**

Provide checked constructors for row-major `Array2<f64>` and fixed-shape arrays. Validate that the native element count equals the requested shape before calling `Array2::from_shape_vec` or `Array3::from_shape_vec`.

- [ ] **Step 6: Run helper tests.**

Run:

```text
cargo fmt --check
cargo test --test internal
```

Expected: PASS without requiring hardware. Add a feature-gated `get_version_*` smoke test only after the native board task enables the runtime link path.

## Task 4: Implement `BoardShim` metadata and logging

**Files:**
- Create/modify: `src/board_shim.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `mindrove_sys` metadata/logging functions and internal probe/status helpers.
- Produces associated functions named after the C++ API: `get_board_descr`, `get_sampling_rate`, `get_package_num_channel`, `get_timestamp_channel`, `get_marker_channel`, `get_battery_channel`, `get_num_rows`, `get_device_name`, `get_eeg_names`, every channel getter, `get_board_presets`, `release_all_sessions`, `get_version`, and logger methods.

- [ ] **Step 1: Define the `BoardShim` state and metadata call pattern.**

Use:

```rust
pub struct BoardShim {
    board_id: BoardId,
    params_json: CString,
    prepared: bool,
    streaming: bool,
}
```

`new(board_id, params)` serializes parameters and returns `Result<Self>`. Metadata functions accept `BoardId` and an optional `MindRovePreset` defaulting to `DEFAULT_PRESET` through explicit `*_with_preset` methods where Rust cannot express C++ defaults.

- [ ] **Step 2: Write metadata tests against mocked helper closures.**

Test that a scalar native result is returned only for status zero, a string uses UTF-8 validation, and a negative channel index becomes `Error::InvalidLength`. Keep these tests independent from the hardware library.

- [ ] **Step 3: Implement scalar, string, vector, and JSON metadata.**

Use the exact raw functions from `board_info_getter.h`. Convert nonnegative channel/preset indices to `usize` or `MindRovePreset` only after validation. Return `serde_json::Value` from `get_board_descr` after UTF-8 and JSON parsing. Preserve the C++ names and default-preset behavior.

- [ ] **Step 4: Implement logger controls.**

Map `enable_board_logger`, `disable_board_logger`, `enable_dev_board_logger`, `set_log_file`, and `set_log_level` to the board-controller C functions. Implement `log_message(log_level, message: &str)` using the C ABI's message-string function, not C++ varargs.

- [ ] **Step 5: Run pure metadata tests and a raw-link smoke check.**

Run:

```text
cargo test --test types --test error --test internal
cargo check
```

Expected: all pure tests pass and the public module compiles against generated bindings.

## Task 5: Implement `BoardShim` lifecycle, streamers, and board data

**Files:**
- Modify: `src/board_shim.rs`
- Create: `tests/native_board.rs`

**Interfaces:**
- Consumes: `BoardShim` metadata and internal JSON/buffer helpers.
- Produces `prepare_session`, `start_stream`, `start_stream_with_options`, `stop_stream`, `is_prepared`, `release_session`, `add_streamer`, `delete_streamer`, `insert_marker`, `config_board`, `get_board_id`, `get_board_data_count`, `get_current_board_data`, `get_board_data`, and `get_board_data_n`.

- [ ] **Step 1: Write feature-gated synthetic-board tests.**

Use the official lifecycle and assert the channel×sample shape:

```rust
#![cfg(all(feature = "native-tests", target_os = "linux"))]

#[test]
fn synthetic_board_returns_channel_sample_data() {
    let mut board = BoardShim::new(BoardId::SYNTHETIC_BOARD, MindRoveInputParams::default()).unwrap();
    board.prepare_session().unwrap();
    board.start_stream().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let data = board.get_current_board_data(2).unwrap();
    assert_eq!(data.nrows(), BoardShim::get_num_rows(BoardId::SYNTHETIC_BOARD).unwrap());
    assert!(data.ncols() <= 2);
    board.stop_stream().unwrap();
    board.release_session().unwrap();
}
```

Also test `get_board_data_count`, `get_board_data_n`, `get_board_id`, and cleanup after a failed operation.

- [ ] **Step 2: Implement lifecycle state transitions.**

Call the raw functions with `board_id.as_raw()` and `params_json.as_ptr()`. Set `prepared` only after successful preparation and `streaming` only after successful start. `release_session` clears both flags after a successful call. `Drop` calls `stop_stream` if tracked as streaming, then `release_session` if tracked as prepared, ignoring errors and never panicking.

- [ ] **Step 3: Implement board data allocation with the C ABI asymmetry.**

For `get_current_board_data`, obtain `rows`, allocate `rows * requested_samples`, call the raw function with `returned_samples`, validate `0 <= returned_samples <= requested_samples`, then construct `Array2::from_shape_vec((rows, returned_samples), data[..rows * returned_samples].to_vec())`.

For `get_board_data_n`, call `get_board_data_count` first when the requested count is not known, clamp the requested count to the available count, allocate `rows * count`, and call `get_board_data`. For `get_board_data`, use the complete available count. This explicitly handles the fact that `get_board_data` has no returned-sample out-parameter.

- [ ] **Step 4: Implement streamers, markers, and configuration.**

Pass `CString` streamer/config values for `add_streamer`, `delete_streamer`, and `config_board`. Use the string probe/fill helper for `config_board`. Validate the response as UTF-8 and return an empty `String` when the native response length is zero.

- [ ] **Step 5: Run native board tests with explicit environment.**

Run pure tests first:

```text
cargo test --all-targets
```

Then on Linux with the vendored libraries discoverable:

```text
LD_LIBRARY_PATH="$PWD/mindrove-sys/sdk/lib" cargo test --test native_board --features native-tests -- --nocapture
```

Expected: pure tests pass everywhere; the feature-gated synthetic test passes when the SDK libraries are available.

## Task 6: Implement `DataFilter` in-place and scalar operations

**Files:**
- Create/modify: `src/data_filter.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: generated `data_handler` functions, `Error`, enum conversions, and checked slice helpers.
- Produces `DataFilter` associated methods for logger controls, in-place filters, scalar metrics, PPG metrics, windows, and nearest-power calculations.

- [ ] **Step 1: Write pure input-validation tests.**

Cover empty slices, negative/overflowing lengths, invalid band ordering, invalid FFT parity, and mismatched PPG lengths. Example:

```rust
#[test]
fn mismatched_ppg_inputs_are_rejected_before_ffi() {
    let ir = [1.0, 2.0];
    let red = [1.0];
    let green = [1.0, 2.0];
    assert!(matches!(
        DataFilter::get_heart_rate(&ir, &red, &green, 250, 8192),
        Err(Error::InvalidArgument(_))
    ));
}
```

- [ ] **Step 2: Implement `DataFilter` logger/version methods.**

Use `set_log_level_data_handler`, `set_log_file_data_handler`, `log_message_data_handler`, and `get_version_data_handler`. Keep logger messages as `&str` and convert them with the shared C-string helper.

- [ ] **Step 3: Implement in-place filters.**

Expose these exact methods with Rust slices and typed enums:

```rust
pub fn perform_lowpass(data: &mut [f64], sampling_rate: i32, cutoff: f64, order: i32, filter_type: FilterType, ripple: f64) -> Result<()>;
pub fn perform_highpass(data: &mut [f64], sampling_rate: i32, cutoff: f64, order: i32, filter_type: FilterType, ripple: f64) -> Result<()>;
pub fn perform_bandpass(data: &mut [f64], sampling_rate: i32, start_freq: f64, stop_freq: f64, order: i32, filter_type: FilterType, ripple: f64) -> Result<()>;
pub fn perform_bandstop(data: &mut [f64], sampling_rate: i32, start_freq: f64, stop_freq: f64, order: i32, filter_type: FilterType, ripple: f64) -> Result<()>;
pub fn remove_environmental_noise(data: &mut [f64], sampling_rate: i32, noise_type: NoiseType) -> Result<()>;
pub fn perform_rolling_filter(data: &mut [f64], period: i32, operation: AggOperation) -> Result<()>;
pub fn detrend(data: &mut [f64], operation: DetrendOperation) -> Result<()>;
```

Validate nonempty data and positive rates/periods before entering `unsafe`. Pass mutable slice pointers only for the duration of each call.

- [ ] **Step 4: Implement scalar and PPG methods.**

Implement `get_nearest_power_of_two`, `calc_stddev`, `get_railed_percentage`, `get_oxygen_level`, `get_heart_rate`, and `get_rmssd_hrv`. Validate same-length PPG slices, finite coefficient values, and valid sampling rates. Use the SDK defaults for oxygen coefficients in an explicit Rust overload helper.

- [ ] **Step 5: Implement windows and peak/wavelet in-place methods.**

Implement `get_window`, `detect_peaks_z_score`, `perform_wavelet_denoising`, and `restore_data_from_wavelet_detailed_coeffs`. Output arrays must be allocated from checked input dimensions; every native status is converted before returning.

- [ ] **Step 6: Run pure tests and clippy for this module.**

Run:

```text
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: validation tests pass without native hardware and no unsafe call is reachable from an invalid public input.

## Task 7: Implement `DataFilter` allocating, multidimensional, and file operations

**Files:**
- Modify: `src/data_filter.rs`
- Modify: `src/internal/buffers.rs`
- Create/modify: `tests/native_dsp.rs`

**Interfaces:**
- Produces allocating methods returning owned arrays: downsampling, wavelet transform/inverse, FFT/IFFT, PSD/Welch, band powers, CSP, ICA, and file read.
- Uses these result shapes:
  - FFT/PSD: `n / 2 + 1` values for even input `n`;
  - inverse FFT: original input length;
  - CSP filters: `n_channels × n_channels`, eigenvalues: `n_channels`;
  - ICA: the SDK-documented `W`, `K`, `A`, `S` shapes derived from rows, columns, and component count;
  - file read: rows/columns returned by the native probe;
  - wavelet transform: coefficient array plus `decomposition_level + 1` block lengths.

- [ ] **Step 1: Add native-contract tests before wrapping variable-size outputs.**

Use feature-gated tests with the official small arrays to assert the output-size contracts. Specifically assert `sum(decomposition_lengths) == coeffs.len()`, PSD/FFT lengths are `n / 2 + 1`, CSP dimensions match channel count, and ICA matrices match the documented dimensions. These tests prevent an incorrect Rust allocation from being hidden by `Vec` capacity.

- [ ] **Step 2: Implement fixed-size transforms.**

Implement:

```rust
pub fn perform_fft(data: &[f64], window: WindowOperation) -> Result<Array1<Complex<f64>>>;
pub fn perform_ifft(data: &Array1<Complex<f64>>) -> Result<Array1<f64>>;
pub fn get_psd(data: &[f64], sampling_rate: i32, window: WindowOperation) -> Result<(Array1<f64>, Array1<f64>)>;
pub fn get_psd_welch(data: &[f64], nfft: i32, overlap: i32, sampling_rate: i32, window: WindowOperation) -> Result<(Array1<f64>, Array1<f64>)>;
pub fn get_band_power(psd: &(Array1<f64>, Array1<f64>), freq_start: f64, freq_end: f64) -> Result<f64>;
```

Allocate `n / 2 + 1` for FFT/PSD outputs and `n` for inverse FFT; validate even input where required and validate that amplitude/frequency arrays have identical length.

- [ ] **Step 3: Implement downsampling and wavelets using checked SDK contracts.**

Allocate output capacity from the SDK’s documented C ABI contract, not from an arbitrary pointer guess. The adapter must validate `period > 0`, use the SDK-defined aggregation result length, and trim the returned `Vec` to that length. For wavelets, allocate the SDK-required coefficient capacity for the requested input length/wavelet/level/extension, allocate exactly `level + 1` decomposition lengths, and reject any reported block sum that exceeds the allocation or does not equal the copied coefficient length. The inverse call receives only the validated coefficient and length arrays.

If the shipped library's contract probe shows a different bound than the header documentation, encode that observed bound in a private checked helper and a feature-gated regression test before exposing the safe method; never call the native function with an undersized buffer.

- [ ] **Step 4: Implement band-power and file operations.**

Use row-major contiguous `Array2<f64>` for `get_avg_band_powers` and `get_custom_band_powers`; pass channel row indices as checked `i32` values. Allocate fixed five-element arrays for average band powers and `bands.len()` for custom powers. For `read_file`, use the C ABI probe convention with `num_elements = 0`, validate returned rows/columns, allocate `rows * columns`, then perform the fill call. `write_file` passes contiguous row-major data and validates the file mode string.

- [ ] **Step 5: Implement CSP and ICA.**

Accept `Array3<f64>` shaped `(epochs, channels, times)` plus `Array1<f64>` labels for CSP. Accept `Array2<f64>` plus `num_components` and optional channel indices for ICA. Validate all dimensions fit native `int`, allocate the exact documented matrices, call the C functions, and construct arrays only after status and element counts pass.

- [ ] **Step 6: Run DSP tests and inspect native output failures.**

Run pure tests, then:

```text
LD_LIBRARY_PATH="$PWD/mindrove-sys/sdk/lib" cargo test --test native_dsp --features native-tests -- --nocapture
```

Expected: official-example inputs pass shape/length assertions; a malformed native length produces `Error::InvalidLength` rather than a panic or out-of-bounds copy.

## Task 8: Implement `MLModel`

**Files:**
- Create: `src/ml_model.rs`
- Modify: `src/lib.rs`
- Create/modify: `tests/native_ml.rs`

**Interfaces:**
- Consumes: `MindRoveModelParams`, JSON/C-string helpers, and `prepare`, `predict`, `release`, logging, version, and `release_all` raw symbols.
- Produces `MLModel::new`, `prepare`, `predict`, `release`, `release_all`, logger methods, and `get_version`.

- [ ] **Step 1: Write the feature-gated lifecycle test.**

Use `MINDROVE_MODEL_PATH` for an optional model asset. If it is absent, print a clear skip message and return successfully; do not invoke `prepare` with an unknown asset and do not allow missing optional assets to become a link error.

- [ ] **Step 2: Implement model state.**

Use:

```rust
pub struct MLModel {
    params_json: CString,
    max_array_size: usize,
    prepared: bool,
}
```

`new` serializes the parameters. `prepare` sets `prepared` only on status zero. `release` clears it after success. `Drop` releases a prepared model best-effort and never panics.

- [ ] **Step 3: Implement prediction with the native length contract.**

Allocate exactly `max_array_size` doubles, pass its checked `i32` length to `predict`, validate `0 <= output_len <= max_array_size`, truncate the owned vector, and return it. Reject empty input or a zero `max_array_size` before FFI.

- [ ] **Step 4: Implement ML logging and version methods.**

Map the ML module logger functions and `get_version_ml_module` through the same private string/status helpers. `log_message` accepts a Rust `&str`, never a varargs format string.

- [ ] **Step 5: Run ML tests and clippy.**

Run:

```text
cargo test --test native_ml --features native-tests -- --nocapture
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: the test either exercises the model lifecycle with an available asset or reports a deterministic skip; invalid native output lengths always return an error.

## Task 9: Add the official-style example and complete verification

**Files:**
- Create: `examples/synthetic_acquisition.rs`
- Modify: `README.md` (create if absent)
- Modify: `Cargo.toml` only if an explicit `[[example]]` target is needed

**Interfaces:**
- Produces a runnable `synthetic_acquisition` example using the safe API and the official C++ lifecycle.
- Documents Linux library discovery, native test gating, data shape, and explicit cleanup.

- [ ] **Step 1: Write the example against the finished public API.**

The example must follow this sequence:

```rust
BoardShim::enable_dev_board_logger()?;
let mut board = BoardShim::new(BoardId::SYNTHETIC_BOARD, MindRoveInputParams::default())?;
board.prepare_session()?;
board.start_stream()?;
std::thread::sleep(std::time::Duration::from_secs(1));
board.stop_stream()?;
let data = board.get_current_board_data(10)?;
println!("{} channels × {} samples", data.nrows(), data.ncols());
board.release_session()?;
```

Adapt logger methods if they are intentionally infallible, but preserve the same lifecycle and explicit release.

- [ ] **Step 2: Document native runtime setup.**

Document:

```text
LD_LIBRARY_PATH="$PWD/mindrove-sys/sdk/lib" cargo run --example synthetic_acquisition --features native-tests
LD_LIBRARY_PATH="$PWD/mindrove-sys/sdk/lib" cargo test --all-targets --features native-tests
```

Explain that pure unit tests run without hardware, while synthetic/native tests require the vendored Linux libraries.

- [ ] **Step 3: Run the complete verification matrix.**

Run:

```text
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
LD_LIBRARY_PATH="$PWD/mindrove-sys/sdk/lib" cargo test --all-targets --features native-tests -- --nocapture
```

Expected: formatting, pure tests, and clippy pass; native tests pass or explicitly skip only the ML asset-dependent portion. Review `git diff --check` and `git status --short` before reporting completion. Do not commit unless requested.

## Plan self-review

- **Spec coverage:** raw C ABI expansion is Task 1; public types/errors are Task 2; safe FFI helpers are Task 3; BoardShim metadata/lifecycle/data are Tasks 4–5; DataFilter is Tasks 6–7; MLModel is Task 8; examples and required commands are Task 9.
- **Safety coverage:** checked lengths, C strings, status conversion, ndarray contiguity, output contracts, Drop cleanup, and native-test gating are explicitly assigned to Tasks 3, 5, 7, and 8.
- **Completeness check:** every task has concrete files, interfaces, test code, and verification commands. Conditional native behavior is limited to explicit SDK/model availability checks and linker diagnostics.
- **Type consistency:** all public modules consume the `Error`/`Result`, enum, parameter, and internal-helper interfaces established in Tasks 2–3. Board data is always `Array2<f64>` channels × samples; multidimensional DSP shapes are fixed in Task 7.
- **Repository fit:** the existing obsolete `add(2, 2)` test is removed, generated bindings remain uncommitted, and no C++ headers or native binaries are modified.
