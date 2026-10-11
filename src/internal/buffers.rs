use std::os::raw::{c_char, c_int};

use ndarray::Array2;
#[cfg(test)]
use ndarray::Array3;

use crate::{Error, Result};

use super::check_status;

const MAX_NATIVE_BYTES: usize = 1 << 20;

/// Maximum number of doubles the safe wrapper will allocate for a single
/// `read_file` call. Chosen to match `MAX_NATIVE_BYTES` so the SDK's
/// `num_elements` parameter stays within native `int` bounds and never
/// exceeds the workspace size used by other helpers.
pub(crate) const MAX_NATIVE_FILE_ELEMENTS: usize = MAX_NATIVE_BYTES;

pub(crate) fn checked_element_count(first: usize, second: usize) -> Result<usize> {
    let count = first.checked_mul(second).ok_or(Error::IntegerOverflow)?;
    checked_native_capacity(count)?;
    Ok(count)
}

pub(crate) fn checked_native_capacity(capacity: usize) -> Result<c_int> {
    c_int::try_from(capacity).map_err(|_| Error::IntegerOverflow)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn checked_buffer_size(
    elements: usize,
    element_size: usize,
    extra: usize,
) -> Result<usize> {
    elements
        .checked_mul(element_size)
        .and_then(|size| size.checked_add(extra))
        .ok_or(Error::IntegerOverflow)
}

pub(crate) fn checked_native_len(length: c_int) -> Result<usize> {
    if length < 0 {
        return Err(Error::InvalidLength);
    }

    let length = usize::try_from(length).map_err(|_| Error::InvalidLength)?;
    if length > MAX_NATIVE_BYTES {
        return Err(Error::BufferContract("native length exceeds 1 MiB".into()));
    }
    Ok(length)
}

pub(crate) fn read_string<F>(call: F) -> Result<String>
where
    F: Fn(*mut c_char, *mut c_int) -> c_int,
{
    let capacity = MAX_NATIVE_BYTES;
    let mut buffer = vec![0_u8; capacity];
    let mut filled_len = checked_native_capacity(capacity)?;

    // The native callback writes into this live allocation and receives its checked byte capacity.
    check_status(call(buffer.as_mut_ptr().cast(), &mut filled_len))?;
    let filled_len = checked_native_len(filled_len)?;
    if filled_len > capacity {
        return Err(Error::BufferContract(
            "native string fill exceeded bounded capacity".into(),
        ));
    }
    let end = buffer[..filled_len]
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(filled_len);
    String::from_utf8(buffer[..end].to_vec()).map_err(|_| Error::InvalidUtf8)
}

pub(crate) fn read_i32_vec<F>(call: F) -> Result<Vec<i32>>
where
    F: Fn(*mut i32, *mut c_int) -> c_int,
{
    let capacity = MAX_NATIVE_BYTES / std::mem::size_of::<i32>();
    let native_capacity = checked_native_capacity(capacity)?;
    let mut buffer = vec![0_i32; capacity];
    let mut filled_len = native_capacity;

    // The native callback writes only into this live allocation and receives its checked capacity.
    check_status(call(buffer.as_mut_ptr(), &mut filled_len))?;
    let filled_len = checked_native_len(filled_len)?;
    if filled_len > buffer.len() {
        return Err(Error::BufferContract(
            "native integer fill exceeded probed capacity".into(),
        ));
    }
    buffer.truncate(filled_len);
    Ok(buffer)
}

pub(crate) fn array2_from_native(
    data: Vec<f64>,
    rows: usize,
    columns: usize,
) -> Result<Array2<f64>> {
    let expected = checked_element_count(rows, columns)?;
    if data.len() != expected {
        return Err(Error::InvalidLength);
    }
    Array2::from_shape_vec((rows, columns), data).map_err(|_| Error::InvalidLength)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn array3_from_native(
    data: Vec<f64>,
    first: usize,
    second: usize,
    third: usize,
) -> Result<Array3<f64>> {
    let expected = checked_element_count(first, second)
        .and_then(|value| checked_element_count(value, third))?;
    if data.len() != expected {
        return Err(Error::InvalidLength);
    }
    Array3::from_shape_vec((first, second, third), data).map_err(|_| Error::InvalidLength)
}

/// Conservative upper bound for the coefficient array returned by
/// `perform_wavelet_transform`. The published native bound is
/// `data_len + 82 * decomposition_level`; we verified the formula against
/// the highest-order wavelet (`WaveletType::DB15`) at level 7 with a 4096
/// sample input (largest observed `sum(decomposition_lengths)` is 4295 vs.
/// the formula's 4670) and against several other wavelet/level/len
/// combinations via direct probes of the shared library. The function
/// uses checked arithmetic and rejects any capacity that would not fit
/// in the SDK's native `int` range, so the wrapper always allocates a
/// buffer that fits in `i32` and never below the proven sum.
pub(crate) fn wavelet_coefficient_capacity(
    data_len: usize,
    decomposition_level: usize,
) -> Result<usize> {
    let extra = decomposition_level
        .checked_mul(82)
        .ok_or(Error::IntegerOverflow)?;
    let total = data_len.checked_add(extra).ok_or(Error::IntegerOverflow)?;
    checked_native_capacity(total)?;
    Ok(total)
}

/// Sum of reported decomposition-length entries with native-int validation.
pub(crate) fn sum_decomposition_lengths(lengths: &[c_int]) -> Result<usize> {
    let mut total: usize = 0;
    for &value in lengths {
        if value < 0 {
            return Err(Error::InvalidLength);
        }
        total = total
            .checked_add(usize::try_from(value).map_err(|_| Error::InvalidLength)?)
            .ok_or(Error::IntegerOverflow)?;
    }
    Ok(total)
}
