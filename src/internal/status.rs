use crate::{Error, Result};

pub(crate) fn check_status(status: i32) -> Result<()> {
    Error::from_status(status)
}
