use std::fmt;
use std::io;
use std::num;

use quick_xml;

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    Custom(String),
}

impl Error {
    pub fn custom(msg: &str) -> Self {
        Error::Custom(String::from(msg))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => msg.fmt(f),
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum FileError {
    UnsuppExt(String),
    Io(io::Error),
    InvalidUnicode(String),
    XmlIo(quick_xml::Error),
}

impl FileError {
    pub fn unsupported_extension(ext: &str, supported: &[&str]) -> Self {
        let mut msg = {
            if ext.is_empty() {
                String::from("The file has no extension.")
            } else {
                format!("Unsupported extension '{}' was given.", ext)
            }
        };

        msg = format!("{} Please use a valid path to the osm-file.", msg);
        msg = format!("{}\nSupported extensions are: {:?}", msg, supported);

        FileError::UnsuppExt(msg)
    }

    pub fn invalid_unicode() -> Self {
        FileError::InvalidUnicode(String::from("File name is invalid Unicode."))
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileError::UnsuppExt(msg) => msg.fmt(f),
            FileError::Io(e) => e.fmt(f),
            FileError::XmlIo(e) => e.fmt(f),
            FileError::InvalidUnicode(msg) => msg.fmt(f),
        }
    }
}

impl From<io::Error> for FileError {
    fn from(e: io::Error) -> Self {
        FileError::Io(e)
    }
}

impl From<quick_xml::Error> for FileError {
    fn from(e: quick_xml::Error) -> Self {
        FileError::XmlIo(e)
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Int(num::ParseIntError),
    Float(num::ParseFloatError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Io(e) => e.fmt(f),
            ParseError::Int(e) => e.fmt(f),
            ParseError::Float(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::Io(e)
    }
}

impl From<num::ParseIntError> for ParseError {
    fn from(e: num::ParseIntError) -> Self {
        ParseError::Int(e)
    }
}

impl From<num::ParseFloatError> for ParseError {
    fn from(e: num::ParseFloatError) -> Self {
        ParseError::Float(e)
    }
}
