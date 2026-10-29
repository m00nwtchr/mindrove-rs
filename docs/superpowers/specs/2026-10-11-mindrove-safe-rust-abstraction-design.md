# MindRove Safe Rust Abstraction Design

## Status

Design approved in conversation; implementation has not started.

## Goal

Add a higher-level safe Rust abstraction over the vendored `mindrove-sys` bindings. The public API will mirror the public C++ SDK façade represented by `BoardShim`, `DataFilter`, and `MLModel`, while using idiomatic Rust ownership, `Result`-based errors, and `ndarray` values instead of exposing C++ containers or raw pointers.

The official MindRoveSDK examples are the behavioral and public-surface reference, especially the examples under `examples/cpp`, `examples/cs`, and `examples/python` in <https://github.com/MindRove/MindRoveSDK>.

## Settled decisions

- Use the SDK's C ABI as the FFI boundary. Do not bind the C++ classes directly.
- Implement the complete public `BoardShim`, `DataFilter`, and `MLModel` surface from the vendored headers and official examples.
- Omit JNI integration and the explicitly internal C function `get_num_elements_in_file`.
- Use `ndarray::Array1`, `Array2`, and `Array3` for public numeric arrays.
- Preserve the SDK's canonical data shapes:
  - board data: channels × samples;
  - epoch data: epochs × channels × samples.
- Keep raw pointers, C strings, native output buffers, and unsafe calls private to the FFI adapter.

## Architecture

### `mindrove-sys`

`mindrove-sys` remains a raw, generated binding crate. Its wrapper header and build script will include the C ABI headers for:

- board control and lifecycle;
- board metadata and channel indices;
- signal processing and file operations;
- machine-learning model operations.

The build script will link the native `BoardController`, `DataHandler`, and `MLModule` libraries in addition to the existing board-controller library path. Generated symbols are not reinterpreted as safe Rust APIs.

### `mindrove`

The root crate will expose focused modules:

- `board_shim`: board metadata, session lifecycle, streamers, markers, board configuration, and data retrieval;
- `data_filter`: static signal-processing, spectral, PPG, file, CSP, and ICA operations;
- `ml_model`: owned model parameters and prepare/predict/release lifecycle;
- `types`: Rust representations of SDK enums and JSON parameter structs;
- `error`: the public error type and native status conversion;
- a private FFI/buffer layer shared by the public modules.

The public module organization will retain the C++ class concepts. `BoardShim` and `MLModel` are owned Rust structs; `DataFilter` is represented by a namespace-like unit type with associated functions, matching the C++ static-method usage.

## Public types and API mapping

### Enums and constants

SDK integer enums will be represented with exact discriminants for board IDs, presets, IP protocols, filters, aggregation, windows, detrending, metrics, classifiers, noise, wavelet, threshold, extension, and noise-estimation choices. Board IDs will expose the named SDK constants and a raw-ID constructor so playback, streaming, custom, or future IDs can still be passed through.

Native exit codes will be converted immediately into named `thiserror` variants. Unknown integer values become `Error::UnknownNativeStatus(raw)`, preserving the original status code for diagnostics.

### Parameters

`MindRoveInputParams` will be a public, defaultable Rust struct containing the SDK's sixteen fields and matching its defaults (`ip_port = -1`, `master_board = NO_BOARD`, and empty strings elsewhere). `MindRoveModelParams` will provide the C++ `(metric, classifier)` constructor semantics, default optional strings, and `max_array_size = 8192`.

Both structs will serialize with `serde`/`serde_json` using the exact field names consumed by the native SDK. Interior NULs are rejected before creating FFI strings.

### Arrays and scalars

- `BoardShim` data methods return `ndarray::Array2<f64>`.
- CSP and ICA methods return dimensionally appropriate `Array1`, `Array2`, and `Array3` values.
- Flat DSP results return owned `Vec<f64>` or a named pair/result struct where two arrays have distinct meanings, such as amplitude and frequency.
- In-place filters accept `&mut [f64]`.
- Public methods return `Result<T, Error>` whenever the native SDK can report failure or the Rust adapter can reject invalid input.

All multidimensional arrays are standard-layout/contiguous before being passed to native code. Shape conversion checks element counts and integer bounds.

## Safety and ownership model

Every native invocation is made through a private helper which validates arguments, performs checked length arithmetic, invokes the C function in a small `unsafe` block, converts the returned status immediately, and copies native output into owned Rust storage before returning.

### Board sessions

`BoardShim::new` serializes input parameters, calls the native `prepare_session`, and only returns a `BoardShim` with `prepared = true` after successful preparation. The Rust type therefore owns its native session for the rest of its lifetime. The remaining lifecycle methods correspond to the C++ API:

- `start_stream`;
- `stop_stream`;
- `is_prepared`.

`Drop` is the only release path. It stops an active stream, then calls the native release through a private/internal helper that does not recreate the removed public API. Failure during `Drop` is logged as a `target: "mindrove"` warning and never panics; cleanup is best-effort.

Board data allocation uses checked `rows × requested_samples` arithmetic. `get_current_board_data` uses the native returned-sample count to construct the final `Array2` shape. Draining methods determine the requested count according to the C++ overload semantics and preserve the channels × samples layout.

### Strings and variable-length metadata

String, channel-index, preset, and version results use private length-probe/fill helpers. Returned lengths are validated against allocated capacity and converted from native bytes only after UTF-8 validation. The helpers have a finite maximum allocation and return an explicit buffer/length error instead of looping indefinitely on malformed native output.

### DSP and model buffers

Input slices are never mutated unless the public method is explicitly an in-place operation. Native output buffers are allocated by Rust using dimensions derived from the SDK operation, copied into `Vec`/`ndarray`, and then dropped by Rust. No native allocation is freed through an assumed allocator.

For C ABI operations whose output size is represented by an out-parameter, the returned length is range-checked before constructing the Rust result. For operations with dimensionally fixed or input-bounded output, the adapter allocates the documented bound and trims to the SDK-defined result shape. It never relies on a guessed pointer length or exposes an unbounded native pointer.

`MLModel` stores serialized model parameters and tracks preparation. `predict` allocates according to `max_array_size`, validates the native output length, copies the result, and releases the native classifier explicitly or best-effort during `Drop`.

## Error model

The public `Error` type will distinguish:

- named `thiserror` variants for known MindRove failures;
- an unknown native-status error that preserves its original status code for diagnostics;
- invalid Rust arguments and integer/shape overflow;
- interior NULs and invalid UTF-8;
- JSON serialization/deserialization failures;
- invalid native output lengths or shapes;
- native library/buffer contract failures.

No nonzero native status is silently ignored. Logging helpers accept ordinary Rust strings rather than exposing the C++ varargs API.

## Testing and examples

### Unit tests

Pure Rust tests will cover:

- all exported enum discriminants and board constants;
- parameter defaults and exact JSON serialization;
- status-to-error conversion, including unknown codes;
- checked shape/length calculations;
- contiguous ndarray conversions and invalid-shape handling.

### Native integration tests

Native tests will be separated from pure Rust tests and gated by native SDK availability. The synthetic board will exercise:

- metadata and channel queries;
- prepare/start/stop/release lifecycle;
- current and draining board data;
- markers and streamers where supported;
- cleanup after errors.

DSP tests will use the official example inputs for filters, downsampling, wavelets, FFT/IFFT, PSD and band powers, PPG helpers, serialization, CSP, and ICA. ML tests will cover model prepare/predict/release when the shipped ML library and model assets are available.

A Rust synthetic acquisition example will document the normal lifecycle and match the official C++ example flow.

Required verification commands are:

```text
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

Native-library-dependent tests will be clearly identified so the pure Rust suite remains diagnosable on machines without hardware or optional model assets.

## Scope exclusions

The implementation will not bind C++ classes, expose JNI, expose `get_num_elements_in_file`, reproduce C++ varargs logging, or expose `MindRoveArray` as a second public container abstraction. Unrelated repository cleanup, hardware-specific configuration beyond the official public API, and changes to the native SDK binaries are out of scope.
