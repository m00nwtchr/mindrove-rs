// `Off` and its `to_native` mapping are only constructed and consumed on
// Linux (see `infer_vendor_level` inside the `linux_bridge` module). On
// other targets the variant is part of the type but never instantiated,
// which would otherwise trip the `dead_code` lint under strict warnings.
// `cfg_attr` keeps the variant available on every target while silencing
// the warning only where it is genuinely unreachable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum VendorLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    Off,
}

impl VendorLogLevel {
    #[cfg(target_os = "linux")]
    const fn to_native(self) -> i32 {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
            Self::Off => 6,
        }
    }
}

struct ParsedVendorLine {
    level: VendorLogLevel,
    logger: String,
    message: String,
}

fn parse_vendor_line(line: &str) -> ParsedVendorLine {
    let malformed = || ParsedVendorLine {
        level: VendorLogLevel::Debug,
        logger: "unknown".to_owned(),
        message: line.to_owned(),
    };

    let Some((_, remainder)) = line
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("] "))
    else {
        return malformed();
    };
    let Some((logger, remainder)) = remainder
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("] "))
    else {
        return malformed();
    };
    let Some((level, message)) = remainder
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("] "))
    else {
        return malformed();
    };

    let level = match level {
        "trace" => VendorLogLevel::Trace,
        "debug" => VendorLogLevel::Debug,
        "info" => VendorLogLevel::Info,
        "warn" | "warning" => VendorLogLevel::Warn,
        "error" | "critical" => VendorLogLevel::Error,
        _ => return malformed(),
    };

    ParsedVendorLine {
        level,
        logger: logger.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(target_os = "linux")]
mod linux_bridge {
    use super::{ParsedVendorLine, VendorLogLevel, parse_vendor_line};
    use crate::internal::{set_log_file_raw, set_log_level_raw};
    use std::ffi::CString;
    use std::fs::File as UnixFile;
    use std::io::{BufRead, BufReader};
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::sync::{Once, OnceLock};
    use std::thread::JoinHandle;

    pub(super) struct LogBridge {
        #[allow(dead_code)]
        write_fd: OwnedFd,
        #[allow(dead_code)]
        thread: JoinHandle<()>,
    }

    static BRIDGE: OnceLock<Result<LogBridge, String>> = OnceLock::new();
    static WARN_ONCE: Once = Once::new();

    pub(super) fn install() {
        let _ = BRIDGE.get_or_init(|| {
            let attempt = create_bridge();
            if let Err(ref message) = attempt {
                WARN_ONCE.call_once(|| {
                    tracing::warn!(
                        target: "mindrove",
                        "failed to install vendor logging bridge: {message}"
                    );
                });
            }
            attempt
        });
    }

    fn create_bridge() -> Result<LogBridge, String> {
        let mut fds = [0 as libc::c_int; 2];
        let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        if rc != 0 {
            return Err(format!(
                "pipe2(O_CLOEXEC) failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        // SAFETY: pipe2 returned 0 and the kernel handed us two valid FDs.
        let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };

        // Start the reader thread BEFORE configuring the SDK with the
        // /proc/self/fd/<write-fd> sink. If the SDK configuration below
        // fails (non-zero status, or the writer we passed is now in an
        // indeterminate state), the returned `Err` causes the
        // `JoinHandle` to be dropped, which *detaches* the thread rather
        // than joining it. The thread continues to own the read side of
        // the pipe, so any subsequent vendor write into the configured
        // sink remains drainable instead of producing SIGPIPE on a dead
        // pipe. The thread terminates with the process.
        let read_file = UnixFile::from(read_fd);
        let thread = std::thread::Builder::new()
            .name("mindrove-logger".to_owned())
            .stack_size(256 * 1024)
            .spawn(move || reader_loop(read_file))
            .map_err(|e| format!("failed to spawn reader thread: {e}"))?;

        let path = format!("/proc/self/fd/{}", write_fd.as_raw_fd());
        let c_path = CString::new(path)
            .map_err(|e| format!("bridge path contained an interior NUL: {e}"))?;

        let status = set_log_file_raw(c_path.as_ptr());
        if status != 0 {
            return Err(format!(
                "set_log_file_board_controller returned status {status}"
            ));
        }

        let level = infer_vendor_level();
        let level_status = set_log_level_raw(level.to_native());
        if level_status != 0 {
            return Err(format!(
                "set_log_level_board_controller returned status {level_status}"
            ));
        }

        Ok(LogBridge { write_fd, thread })
    }

    fn infer_vendor_level() -> VendorLogLevel {
        if tracing::enabled!(target: "mindrove_sdk", tracing::Level::TRACE) {
            VendorLogLevel::Trace
        } else if tracing::enabled!(target: "mindrove_sdk", tracing::Level::DEBUG) {
            VendorLogLevel::Debug
        } else if tracing::enabled!(target: "mindrove_sdk", tracing::Level::INFO) {
            VendorLogLevel::Info
        } else if tracing::enabled!(target: "mindrove_sdk", tracing::Level::WARN) {
            VendorLogLevel::Warn
        } else if tracing::enabled!(target: "mindrove_sdk", tracing::Level::ERROR) {
            VendorLogLevel::Error
        } else {
            VendorLogLevel::Off
        }
    }

    fn reader_loop(file: UnixFile) {
        let mut reader = BufReader::with_capacity(8 * 1024, file);
        let mut buffer: Vec<u8> = Vec::with_capacity(256);
        loop {
            buffer.clear();
            match reader.read_until(b'\n', &mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    if buffer.last() == Some(&b'\n') {
                        buffer.pop();
                    }
                    let text = String::from_utf8_lossy(&buffer);
                    let parsed = parse_vendor_line(&text);
                    emit_parsed(&parsed);
                }
                Err(_) => break,
            }
        }
    }

    fn emit_parsed(parsed: &ParsedVendorLine) {
        match parsed.level {
            VendorLogLevel::Trace => {
                tracing::trace!(
                    target: "mindrove_sdk",
                    logger = %parsed.logger,
                    "{}",
                    parsed.message
                );
            }
            VendorLogLevel::Debug => {
                tracing::debug!(
                    target: "mindrove_sdk",
                    logger = %parsed.logger,
                    "{}",
                    parsed.message
                );
            }
            VendorLogLevel::Info => {
                tracing::info!(
                    target: "mindrove_sdk",
                    logger = %parsed.logger,
                    "{}",
                    parsed.message
                );
            }
            VendorLogLevel::Warn => {
                tracing::warn!(
                    target: "mindrove_sdk",
                    logger = %parsed.logger,
                    "{}",
                    parsed.message
                );
            }
            VendorLogLevel::Error => {
                tracing::error!(
                    target: "mindrove_sdk",
                    logger = %parsed.logger,
                    "{}",
                    parsed.message
                );
            }
            VendorLogLevel::Off => {
                tracing::debug!(
                    target: "mindrove_sdk",
                    logger = %parsed.logger,
                    "{}",
                    parsed.message
                );
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn ensure_logging_bridge() {
    linux_bridge::install();
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn ensure_logging_bridge() {}

#[cfg(test)]
mod tests {
    use super::{VendorLogLevel, parse_vendor_line};

    #[test]
    fn parses_vendor_levels_and_fields() {
        let cases = [
            ("trace", VendorLogLevel::Trace),
            ("debug", VendorLogLevel::Debug),
            ("info", VendorLogLevel::Info),
            ("warn", VendorLogLevel::Warn),
            ("warning", VendorLogLevel::Warn),
            ("error", VendorLogLevel::Error),
            ("critical", VendorLogLevel::Error),
        ];

        for (level, expected_level) in cases {
            let parsed = parse_vendor_line(&format!(
                "[2026-10-12T12:34:56Z] [vendor.logger] [{level}] message body"
            ));

            assert_eq!(parsed.level, expected_level);
            assert_eq!(parsed.logger, "vendor.logger");
            assert_eq!(parsed.message, "message body");
        }
    }

    #[test]
    fn preserves_malformed_vendor_line() {
        for line in [
            "not a bracketed vendor line",
            "[ts] [logger] [fatal] message",
            "[ts] [logger] message",
            "",
            "[",
        ] {
            let parsed = parse_vendor_line(line);

            assert_eq!(parsed.level, VendorLogLevel::Debug);
            assert_eq!(parsed.logger, "unknown");
            assert_eq!(parsed.message, line);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn pipe_lifecycle_round_trip_without_native_library() {
        use std::fs::File as UnixFile;
        use std::io::{BufRead, BufReader, BufWriter, Write};
        use std::os::fd::FromRawFd;

        let mut fds = [0 as libc::c_int; 2];
        let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        assert_eq!(rc, 0, "pipe2(O_CLOEXEC) should succeed");

        // SAFETY: pipe2 returned 0 with two fresh, valid FDs.
        let read_file = unsafe { UnixFile::from_raw_fd(fds[0]) };
        let write_file = unsafe { UnixFile::from_raw_fd(fds[1]) };

        let mut writer = BufWriter::new(write_file);
        let formatted = b"[2026-10-12T12:34:56Z] [vendor.logger] [info] hello from pipe\n";
        writer.write_all(formatted).expect("write should succeed");
        writer.flush().expect("flush should succeed");

        let mut reader = BufReader::new(read_file);
        let mut buffer: Vec<u8> = Vec::new();
        let read = reader
            .read_until(b'\n', &mut buffer)
            .expect("read should succeed before write end closes");
        assert!(read > 0, "expected at least one byte from the pipe");
        assert_eq!(buffer.last(), Some(&b'\n'));

        let text = std::str::from_utf8(&buffer[..buffer.len() - 1])
            .expect("vendor line should be valid UTF-8");
        let parsed = parse_vendor_line(text);
        assert_eq!(parsed.level, VendorLogLevel::Info);
        assert_eq!(parsed.logger, "vendor.logger");
        assert_eq!(parsed.message, "hello from pipe");

        drop(writer);

        buffer.clear();
        let read_after_close = reader
            .read_until(b'\n', &mut buffer)
            .expect("read after close should report EOF, not error");
        assert_eq!(read_after_close, 0, "expected EOF after closing write end");
    }

    /// Simulates the `create_bridge` failure/cleanup path: the reader
    /// thread is started first, the `JoinHandle` is then dropped (the
    /// way an `Err` return from `create_bridge` would propagate it),
    /// and a write to the still-configured writer end must succeed
    /// without `SIGPIPE` because the detached thread retains the read
    /// side. The thread exits cleanly once the writer is closed.
    #[cfg(target_os = "linux")]
    #[test]
    fn dropping_reader_handle_after_spawn_keeps_pipe_drainable() {
        use std::fs::File as UnixFile;
        use std::io::{BufReader, BufWriter, Write};
        use std::os::fd::FromRawFd;

        let mut fds = [0 as libc::c_int; 2];
        let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        assert_eq!(rc, 0, "pipe2(O_CLOEXEC) should succeed");

        // SAFETY: pipe2 returned 0 with two fresh, valid FDs.
        let read_file = unsafe { UnixFile::from_raw_fd(fds[0]) };
        let write_file = unsafe { UnixFile::from_raw_fd(fds[1]) };

        // Mirror `create_bridge`: start the reader thread first, then
        // simulate a later failure by dropping the JoinHandle. The
        // thread closure owns the read side and simply drains it to
        // EOF; that is enough to prove the read FD stays open after
        // the JoinHandle is dropped.
        let handle = std::thread::Builder::new()
            .name("mindrove-logger-failure-test".to_owned())
            .stack_size(256 * 1024)
            .spawn(move || {
                let mut reader = BufReader::with_capacity(8 * 1024, read_file);
                let mut sink = std::io::sink();
                let _ = std::io::copy(&mut reader, &mut sink);
            })
            .expect("reader thread should spawn");
        drop(handle);

        // The writer end is configured with the SDK (out of band here);
        // a write must not raise SIGPIPE because the detached reader
        // thread still owns the read side of the pipe.
        let mut writer = BufWriter::new(write_file);
        let payload = b"[2026-10-12T12:34:56Z] [vendor.logger] [info] post-failure write\n";
        writer
            .write_all(payload)
            .expect("write must succeed while detached reader holds the read side");
        writer.flush().expect("flush must succeed");

        // Closing the writer lets the detached reader reach EOF and
        // exit. We don't synchronize on the thread because it is
        // detached; a brief sleep is sufficient for the kernel to
        // deliver EOF and the thread to drop its read FD.
        drop(writer);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
