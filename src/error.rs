//! IO functions
use base64::DecodeError as Base64DecodeError;
use displaydoc::Display;
use nom::error::{ErrorKind, VerboseError, VerboseErrorKind};
use nom::InputLength;
use std::borrow::Cow;
use std::io::Error as IoError;
use thiserror::Error;

/// Error wrapper when loading a sysdiagram
#[derive(Debug, Error, Display)]
pub enum Error {
    /// Not implemented
    NotImplemented,
    /// Could not decode base64 value
    Base64(#[from] Base64DecodeError),
    /// CFB Error
    Cfb(#[from] IoError),
    /// Stream is too long
    StreamTooLong(std::num::TryFromIntError),
    /// Stream is too short
    SiteTooLong(std::num::TryFromIntError),
    /// Buffer is too long
    BufTooLong(std::num::TryFromIntError),
    /// Missing a stream with the filename
    MissingStream(&'static str),
    /// Parsing incomplete
    Incomplete,
    /// Nom parsing error: {0:?} at -{1}
    ParseError(ErrorKind, usize),
    /// Nom parsing failure: {0:?} at -{1}
    ParseFailure(ErrorKind, usize),
    /// Nom parsing error: {0:#?}
    ParseErrorVerbose(Vec<(VerboseErrorKind, usize)>),
    /// Nom parsing failure: {0:#?}
    ParseFailureVerbose(Vec<(VerboseErrorKind, usize)>),
    /// String encoding error: {0:?}
    StringEncoding(String),
}

/// Result when loading a sysdiagram
pub type LoadResult<T> = Result<T, Error>;

impl<I: InputLength> From<nom::Err<nom::error::Error<I>>> for Error {
    fn from(e: nom::Err<nom::error::Error<I>>) -> Error {
        match e {
            // Need to translate the error here, as this lives longer than the input
            nom::Err::Incomplete(_) => Error::Incomplete,
            nom::Err::Error(e) => Error::ParseError(e.code, e.input.input_len()),
            nom::Err::Failure(e) => Error::ParseFailure(e.code, e.input.input_len()),
        }
    }
}

impl<I: InputLength> From<nom::Err<VerboseError<I>>> for Error {
    fn from(e: nom::Err<VerboseError<I>>) -> Error {
        match e {
            // Need to translate the error here, as this lives longer than the input
            nom::Err::Incomplete(_) => Error::Incomplete,
            nom::Err::Error(e) => Error::ParseErrorVerbose(
                e.errors
                    .into_iter()
                    .map(|e| (e.1, e.0.input_len()))
                    .collect(),
            ),
            nom::Err::Failure(e) => Error::ParseFailureVerbose(
                e.errors
                    .into_iter()
                    .map(|e| (e.1, e.0.input_len()))
                    .collect(),
            ),
        }
    }
}

impl From<Cow<'_, str>> for Error {
    fn from(e: Cow<'_, str>) -> Self {
        Error::StringEncoding(String::from(e))
    }
}
