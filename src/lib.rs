pub mod board_shim;
pub mod data_filter;
pub mod error;
pub(crate) mod internal;
mod logging_bridge;
pub mod ml_model;
pub mod syncbox;
pub mod types;

pub use board_shim::BoardShim;
pub use data_filter::{DEFAULT_OXYGEN_COEFS, DataFilter};
pub use error::{Error, Result};
pub use ml_model::MLModel;
pub use types::*;

pub(crate) fn ensure_scope_shim_linked() {
    mindrove_sys::ensure_scope_shim_linked();
}
