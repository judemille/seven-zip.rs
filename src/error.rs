#![allow(clippy::module_name_repetitions)]

use std::io;

use snafu::prelude::*;

#[derive(Snafu, Debug)]
pub enum ReadError {
    #[snafu(display("The file is not valid 7z!"))]
    Invalid7z,
    #[snafu(display("CRC validation failed! File corruption suspected."))]
    CrcInvalid,
    #[snafu(display("The compression method is not supported!"))]
    MethodUnsupported,
    #[snafu(display("An I/O error has occurred."))]
    #[snafu(context(false))]
    Io {
        source: io::Error
    }
}