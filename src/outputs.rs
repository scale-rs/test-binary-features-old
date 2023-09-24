use crate::BUFFER_SIZE;
#[cfg(feature = "nightly")]
use std::io::{IoSlice, IoSliceMut};
use std::io::{Read, Result as IoResult, Write};
#[cfg(feature = "nightly")]
use std::ops::Deref;

fn copy_all_bytes_classic(
    out: &mut impl Write,
    inp: &mut impl Read,
    buffer: &mut [u8],
) -> IoResult<usize> {
    let mut total_len = 0usize;

    loop {
        let len_read = inp.read(buffer)?;
        if len_read == 0 {
            break Ok(total_len);
        }

        out.write(&buffer[0..len_read])?;
        total_len += len_read;
    }
}

#[cfg(feature = "nightly")]
fn copy_all_bytes_vectored(
    out: &mut impl Write,
    inp: &mut impl Read,
    buffer: &mut [u8],
) -> IoResult<usize> {
    let slice_in = IoSliceMut::new(buffer);
    let mut slice_in_wrapped = [slice_in];
    let mut total_len = 0usize;

    loop {
        let len_read = inp.read_vectored(&mut slice_in_wrapped)?;
        if len_read == 0 {
            break Ok(total_len);
        }

        let slice_from = IoSlice::new(&slice_in_wrapped[0].deref()[0..len_read]);
        out.write_all_vectored(&mut [slice_from])?;
        total_len += len_read;
    }
}

/// Copy, through a buffer of [BUFFER_SIZE] bytes. Return the total length copied (on success).
pub fn copy_all_bytes(out: &mut impl Write, inp: &mut impl Read) -> IoResult<usize> {
    let mut buffer = [0u8; BUFFER_SIZE];

    #[cfg(feature = "nightly")]
    if inp.is_read_vectored() && out.is_write_vectored() {
        copy_all_bytes_vectored(out, inp, &mut buffer)
    } else {
        copy_all_bytes_classic(out, inp, &mut buffer)
    }
    #[cfg(not(feature = "nightly"))]
    copy_all_bytes_classic(out, inp, &mut buffer)
}
