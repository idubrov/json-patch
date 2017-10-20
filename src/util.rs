use std::fmt;
use std::error::Error;

/// This type represents all possible errors that can occur when applying JSON patch
#[derive(Debug)]
pub enum PatchError {
    /// One of the paths in the patch is invalid
    InvalidPointer,

    /// 'test' operation failed
    TestFailed,
}

impl Error for PatchError {
    fn description(&self) -> &str {
        use PatchError::*;
        match *self {
            InvalidPointer => "invalid pointer",
            TestFailed => "test failed",
        }
    }
}

impl fmt::Display for PatchError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(fmt)
    }
}


pub fn parse_index(str: &str, len: usize) -> Result<usize, PatchError> {
    // RFC 6901 prohibits leading zeroes in index
    if str.starts_with('0') && str.len() != 1 {
        return Err(PatchError::InvalidPointer);
    }
    match str.parse::<usize>() {
        Ok(idx) if idx < len => Ok(idx),
        Err(_) | Ok(_) => Err(PatchError::InvalidPointer),
    }
}

pub fn split_pointer(pointer: &str) -> Result<(&str, String), PatchError> {
    pointer.rfind('/').ok_or(PatchError::InvalidPointer).map(
        |idx| {
            (
                &pointer[0..idx],
                pointer[idx + 1..].replace("~1", "/").replace("~0", "~"),
            )
        },
    )
}
