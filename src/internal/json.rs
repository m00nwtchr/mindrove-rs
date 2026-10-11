use std::ffi::CString;

use serde::Serialize;

use crate::{Error, Result};

pub(crate) fn to_cstring(value: &str) -> Result<CString> {
    CString::new(value).map_err(|_| Error::InteriorNul)
}

pub(crate) fn to_json_cstring<T: Serialize>(value: &T) -> Result<CString> {
    to_cstring(&serde_json::to_string(value)?)
}
