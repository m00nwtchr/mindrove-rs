use crate::{Error, Result};

pub(crate) fn check_status(status: i32) -> Result<()> {
    super::ensure_logging_bridge();
    let result = Error::from_status(status);
    if let Err(ref err) = result {
        tracing::error!(
            target: "mindrove",
            error = %err,
            "MindRove SDK operation failed"
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::check_status;
    use crate::Error;
    use std::io;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl SharedBuf {
        fn into_string(self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap_or_default()
        }
    }

    impl io::Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            io::Write::write(&mut *self.0.lock().unwrap(), buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for SharedBuf {
        type Writer = SharedBuf;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[test]
    fn check_status_emits_targeted_error_on_failure() {
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buf.clone())
            .with_max_level(tracing::Level::ERROR)
            .finish();

        let result = tracing::subscriber::with_default(subscriber, || check_status(7));

        assert!(matches!(result, Err(Error::BoardNotReady)));
        let output = buf.into_string();
        assert!(
            output.contains("mindrove"),
            "expected 'mindrove' target in output: {output}"
        );
        assert!(
            output.contains("MindRove SDK operation failed"),
            "expected message in output: {output}"
        );
        assert!(
            output.contains("MindRove board is not ready"),
            "expected error display in output: {output}"
        );
    }

    #[test]
    fn check_status_returns_typed_error_without_emitting_on_success() {
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buf.clone())
            .with_max_level(tracing::Level::ERROR)
            .finish();

        let result = tracing::subscriber::with_default(subscriber, || check_status(0));

        assert!(result.is_ok());
        assert!(
            buf.into_string().is_empty(),
            "check_status must not emit on success"
        );
    }
}
