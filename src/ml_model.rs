use std::ffi::CString;

use crate::internal;
use crate::{Error, MindRoveModelParams, Result};

pub struct MLModel {
    params_json: CString,
    max_array_size: usize,
    prepared: bool,
}

impl MLModel {
    pub fn new(params: MindRoveModelParams) -> Result<Self> {
        internal::ensure_logging_bridge();
        if params.max_array_size <= 0 {
            return Err(Error::InvalidArgument(
                "max_array_size must be greater than zero".into(),
            ));
        }

        let max_array_size =
            usize::try_from(params.max_array_size).map_err(|_| Error::IntegerOverflow)?;
        let params_json = internal::to_json_cstring(&params)?;
        Ok(Self {
            params_json,
            max_array_size,
            prepared: false,
        })
    }

    pub fn prepare(&mut self) -> Result<()> {
        internal::check_status(internal::ml_prepare(self.params_json.as_ptr()))?;
        self.prepared = true;
        Ok(())
    }

    pub fn predict(&self, data: &[f64]) -> Result<Vec<f64>> {
        if data.is_empty() {
            return Err(Error::InvalidArgument(
                "input data must not be empty".into(),
            ));
        }

        let data_len = internal::checked_native_capacity(data.len())?;
        let output_len = internal::checked_native_capacity(self.max_array_size)?;
        let mut output = vec![0.0; self.max_array_size];
        let mut output_len = output_len;

        internal::check_status(internal::ml_predict(
            data.as_ptr().cast_mut(),
            data_len,
            output.as_mut_ptr(),
            &mut output_len,
            self.params_json.as_ptr(),
        ))?;

        let output_len = validate_output_len(output_len, self.max_array_size)?;
        output.truncate(output_len);
        Ok(output)
    }

    pub fn release(&mut self) -> Result<()> {
        internal::check_status(internal::ml_release(self.params_json.as_ptr()))?;
        self.prepared = false;
        Ok(())
    }

    pub fn release_all() -> Result<()> {
        internal::check_status(internal::ml_release_all())
    }

    pub fn get_version() -> Result<String> {
        // Explicit bridge call in addition to the one performed by
        // `read_string`; either path alone would suffice, both together
        // make the dependency on the bridge clear at this standalone
        // entrypoint.
        internal::ensure_logging_bridge();
        internal::read_string(|buffer, len| internal::ml_version(buffer, len, 1 << 20))
    }
}

impl Drop for MLModel {
    fn drop(&mut self) {
        if self.prepared {
            let status = internal::ml_release(self.params_json.as_ptr());
            self.prepared = false;
            if let Err(err) = internal::check_status(status) {
                tracing::warn!(
                    target: "mindrove",
                    error = %err,
                    "MLModel drop release failed"
                );
            }
        }
    }
}

fn validate_output_len(output_len: i32, max_array_size: usize) -> Result<usize> {
    if output_len < 0 {
        return Err(Error::InvalidLength);
    }

    let output_len = usize::try_from(output_len).map_err(|_| Error::InvalidLength)?;
    if output_len > max_array_size {
        return Err(Error::InvalidLength);
    }
    Ok(output_len)
}

#[cfg(test)]
mod tests {
    use super::validate_output_len;
    use crate::Error;

    #[test]
    fn native_output_length_must_fit_owned_buffer() {
        assert_eq!(validate_output_len(2, 2).unwrap(), 2);
        assert!(matches!(
            validate_output_len(-1, 2),
            Err(Error::InvalidLength)
        ));
        assert!(matches!(
            validate_output_len(3, 2),
            Err(Error::InvalidLength)
        ));
    }

    #[test]
    fn invalid_model_inputs_are_rejected_before_native_calls() {
        let params = crate::MindRoveModelParams {
            max_array_size: 0,
            ..Default::default()
        };
        assert!(matches!(
            super::MLModel::new(params),
            Err(Error::InvalidArgument(_))
        ));

        let model = super::MLModel::new(crate::MindRoveModelParams::default()).unwrap();
        assert!(matches!(model.predict(&[]), Err(Error::InvalidArgument(_))));
    }
}
