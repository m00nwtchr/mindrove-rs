# MindRove Rust bindings

This crate exposes safe Rust wrappers for the MindRove SDK. The public API is
organized into the `board_shim`, `data_filter`, `error`, `ml_model`, and `types`
modules, with the main types also re-exported at the crate root.

## Synthetic acquisition

The official-style synthetic board example uses the safe lifecycle API and
relies on the `BoardShim`'s RAII `Drop` for release:

```sh
cargo run --example synthetic_acquisition --features native-tests
```

The board data returned by `get_current_board_data` is an `Array2<f64>` whose
shape is **channels x samples**. The example prints both dimensions.

## Logging

Applications configure `tracing` and install the subscriber that should receive
their logs. The library does not install a subscriber. On Linux, MindRove
forwards vendor SDK logs automatically through that application-configured
tracing pipeline.

If the Linux logging bridge cannot be constructed, the library continues
without vendor-log forwarding and emits one warning with `target: "mindrove"`.
Native SDK internal logs are capturable only when the SDK honors the configured
`/proc/self/fd/<write-fd>` sink. The bridge does not redirect or otherwise
modify stdout or stderr.

Pure unit tests exercise Rust-side behavior and require no hardware, but the
test binary still loads the vendored FFI libraries at startup. Synthetic board
and other native tests additionally require the vendored Linux runtime. Run the
full native test target with:

```sh
cargo test --all-targets --features native-tests
```

<!-- If the SDK cannot find `libstdc++.so.6`, include the directory containing the -->
<!-- system library as well: -->

<!-- ```sh -->
<!-- LIBSTDCPP=$(gcc -print-file-name=libstdc++.so.6) cargo test --all-targets --features native-tests -- --nocapture -->
<!-- ``` -->

The native ML lifecycle test uses `MINDROVE_MODEL_PATH` when it is set. Without
that variable, only that ML test skips because the model asset is unavailable;
other native failures remain visible.

## C++ example parity

| Upstream C++ example | Rust example |
| --- | --- |
| `examples/cpp/get_data/src/get_data.cpp` | `cargo run --example get_data --features native-tests -- --board-id ...` |
| `examples/cpp/get_data/src/get_data_twice.cpp` | `cargo run --example get_data_twice` |
| `examples/cpp/get_data/src/markers.cpp` | `cargo run --example markers -- --board-id ...` |
| `examples/cpp/get_data/src/multiple_streamers.cpp` | `cargo run --example multiple_streamers` |
| `examples/cpp/get_data/src/syncbox_get_data.cpp` | `cargo run --example syncbox_get_data -- --mac-address ...` |
| `examples/cpp/ml/src/eeg_metrics.cpp` | `cargo run --example eeg_metrics -- --board-id ...` |
| `examples/cpp/ml/src/eeg_metrics_ci.cpp` | `cargo run --example eeg_metrics_ci -- --board-id ... --classifier ... --metric ...` |

Board and model arguments mirror the C++ flags. For syncbox, repeat
`--mac-address` and/or `--ip-port` for additional devices. Native examples need
the same Linux loader setup documented above.
