# MindRove Native Tracing Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) to implement this plan task-by-task.

**Goal:** Transparently forward Linux MindRove SDK log-file output into Rust `tracing` without exposing a bridge type or requiring callers to configure vendor logging.

**Architecture:** The safe crate owns a process-wide `OnceLock<Result<LogBridge, String>>`. On Linux, `LogBridge` creates a pipe, configures the vendor board logger with `/proc/self/fd/<write-fd>`, sets the native level from `tracing::enabled!(target: "mindrove_sdk", ...)`, and starts one named 256 KiB blocking reader thread. The reader parses vendor lines and emits structured events; malformed lines become debug events. Unsupported platforms use a no-op initializer.

**Tech Stack:** Rust standard-library `OnceLock`, `OwnedFd`, `BufReader`, `std::thread`; `libc` for Linux `pipe2`; `tracing` for library events; `tracing-subscriber` only in examples/tests.

**Spec:** User request in the conversation dated 2026-10-12.

## Global Constraints

- The bridge is internal; do not expose `VendorTracingBridge` or a bridge handle.
- Do not call `tracing_subscriber::set_global_default` from library code.
- Do not redirect process stdout/stderr and do not create persistent or temporary files.
- Linux implementation uses a pipe and `/proc/self/fd/<write-fd>`; unsupported platforms compile without blocking the crate.
- Bridge initialization is once per process, shared by all boards, and gracefully degrades with one `target: "mindrove"` warning.
- No Tokio or async runtime.
- Keep vendor logger controls private to the tracing bridge; do not expose public SDK logger controls or redesign unrelated APIs.

---

### Task 1: Add tracing dependencies and define bridge module boundaries

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/logging_bridge.rs`
- Modify: `src/internal/mod.rs`

**Interfaces:**
- Produces `pub(crate) fn ensure_logging_bridge()`.
- Produces `pub(crate) fn refresh_vendor_log_level()` only if needed by tests; do not re-export it publicly.
- `src/logging_bridge.rs` owns all bridge state, parser types, vendor-level mapping, and platform cfgs.

- [ ] Add `tracing = "0.1"` as a normal dependency, `libc = "0.2"` as a Linux implementation dependency, and `tracing-subscriber = "0.3"` as a dev-dependency for examples/tests.
- [ ] Add `mod logging_bridge;` to `src/lib.rs`; keep all bridge symbols `pub(crate)` or private.
- [ ] Add `pub(crate) use logging_bridge::ensure_logging_bridge;` to `src/internal/mod.rs` without exposing it from the crate root.
- [ ] Run `cargo check` to establish the dependency/module baseline before adding bridge behavior.

### Task 2: Implement and test pure vendor-line parsing

**Files:**
- Modify: `src/logging_bridge.rs`

**Interfaces:**
- Private `enum VendorLogLevel { Trace, Debug, Info, Warn, Error }`.
- Private `struct ParsedVendorLine { level: VendorLogLevel, logger: String, message: String }`.
- Private `fn parse_vendor_line(line: &str) -> ParsedVendorLine`.

- [ ] Write failing unit tests for exact input lines containing `trace`, `debug`, `info`, `warn`, `warning`, `error`, and `critical`; assert logger, message, and mapped level. Add malformed-input coverage asserting debug level and original line preservation.
- [ ] Run `cargo test --lib logging_bridge` and verify the tests fail because the parser is absent.
- [ ] Implement bracketed parsing for `[timestamp] [logger] [level] message`; recognize `critical` as `Error`, preserve the message after the final `] `, and return `logger = "unknown"` plus the original line for malformed input.
- [ ] Run the focused parser tests and verify they pass.

### Task 3: Implement Linux pipe bridge and graceful once-only initialization

**Files:**
- Modify: `src/logging_bridge.rs`
- Modify: `src/internal/metadata.rs` only if a raw status-returning file-sink helper is needed

**Interfaces:**
- Private `struct LogBridge { write_fd: OwnedFd, thread: JoinHandle<()> }`.
- Private `fn tracing_log_level() -> VendorLogLevel` using `tracing::enabled!(target: "mindrove_sdk", ...)` in TRACE→DEBUG→INFO→WARN→ERROR order, otherwise Off.
- `ensure_logging_bridge()` returns `()` and never blocks or returns a bridge error to callers.

- [ ] Write a Linux-only pipe lifecycle test that creates a pipe, feeds one formatted line through the write side, reads/parses it, and verifies EOF/cleanup without loading the MindRove shared library.
- [ ] Implement Linux `pipe2(O_CLOEXEC)` and convert both descriptors to `OwnedFd`; build `/proc/self/fd/<write_fd.as_raw_fd()>` without filesystem files.
- [ ] Configure `set_log_file_board_controller` through a private raw helper that does not recursively call `ensure_logging_bridge`; set the inferred native integer level using the existing SDK constants (`TRACE=0`, `DEBUG=1`, `INFO=2`, `WARN=3`, `ERROR=4`, `OFF=6`).
- [ ] Spawn `std::thread::Builder::new().name("mindrove-logger").stack_size(256 * 1024)` with a `BufReader<File>` loop. Emit `tracing::{trace,debug,info,warn,error}` at target `mindrove_sdk`, adding `logger = ...` for parsed lines and preserving malformed lines at debug.
- [ ] Store `Result<LogBridge, String>` in a process-wide `OnceLock`; use a separate `Once` to emit exactly one `target: "mindrove"` warning if construction fails. Do not retry.
- [ ] Keep `write_fd` owned in `LogBridge` for process lifetime; do not join or close it during ordinary operation.
- [ ] Add `#[cfg(not(target_os = "linux"))]` no-op `ensure_logging_bridge()` and parser tests that compile on every target.
- [ ] Run parser, pipe, and library tests; verify bridge failure is not returned through the existing public API.

### Task 4: Wire bridge initialization before SDK use and add wrapper diagnostics

**Files:**
- Modify: `src/board_shim.rs`
- Modify: `src/data_filter.rs`
- Modify: `src/ml_model.rs`
- Modify: `src/internal/status.rs`

**Interfaces:**
- Existing public methods remain source-compatible.
- `internal::check_status` emits a structured `target: "mindrove"` warning/error only when a native call fails; it still returns the existing typed `Error`.

- [ ] Call `internal::ensure_logging_bridge()` before the first native operation in `BoardShim::new`, `MLModel::new`, board metadata/version entrypoints, and the first DataFilter/version entrypoint; avoid recursive calls from bridge setup.
- [ ] Add `tracing::debug!`/`info!` lifecycle diagnostics around BoardShim preparation, streaming, stopping, and release without logging sensitive serialized parameters.
- [ ] Add `tracing::warn!` when `Drop` cleanup fails, preserving best-effort non-panicking cleanup.
- [ ] In `internal::check_status`, convert status first, then emit `tracing::error!(target: "mindrove", error = %err, "MindRove SDK operation failed")` on failure and return the same error.
- [ ] Add pure tests for status logging behavior only if a test subscriber can be installed locally without a global subscriber; otherwise test status conversion as already covered.
- [ ] Run strict clippy and all existing pure tests.

### Task 5: Update examples to configure tracing, not vendor logging

**Files:**
- Modify: `examples/support/mod.rs`
- Modify: `examples/get_data.rs`
- Modify: `examples/get_data_twice.rs`
- Modify: `examples/markers.rs`
- Modify: `examples/multiple_streamers.rs`
- Modify: `examples/syncbox_get_data.rs`
- Modify: `examples/eeg_metrics.rs`
- Modify: `examples/eeg_metrics_ci.rs`
- Modify: `examples/synthetic_acquisition.rs`

**Interfaces:**
- Private `support::init_tracing()` configures a formatter subscriber in binaries only.

- [ ] Add `support::init_tracing()` using `tracing_subscriber::fmt::init()`; never call it from library code.
- [ ] Call it at the start of each example and remove explicit `enable_*_logger` calls so bridge level inference observes the application subscriber.
- [ ] Replace SyncBox `BoardShim::log_message` calls with `tracing::error!(target: "mindrove", error = %err, "...")` while preserving error handling and cleanup.
- [ ] Run `cargo test --examples`, `cargo check --examples`, and strict example clippy.

### Task 6: Documentation and final verification

**Files:**
- Modify: `README.md`

- [ ] Document that applications configure tracing/subscribers and the crate forwards vendor logs automatically on Linux.
- [ ] Document that no subscriber is installed by the library and that bridge failures degrade gracefully with one warning.
- [ ] State that native SDK internal logs are only capturable when the SDK honors the configured `/proc/self/fd` sink; stdout/stderr are untouched.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, native-feature tests with the robust Linux loader path, `cargo test --examples`, `cargo check --examples`, and `git diff --check`.
- [ ] Review all changed files for public bridge leakage, persistent log files, async runtime additions, and repeated initialization.
