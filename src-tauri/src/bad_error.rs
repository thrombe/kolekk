use std::{borrow::Cow, fmt::Display};

use serde::Serialize;

#[derive(Serialize, Debug, thiserror::Error)]
#[error("something went wrong: {0}")]
pub struct Error(pub Cow<'static, str>);

impl Error {
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self(msg.into())
    }
}

pub trait BadError<R> {
    fn bad_err(self, msg: impl Into<Cow<'static, str>>) -> Result<R, Error>;
}

pub trait InferBadError<R> {
    fn infer_err(self) -> Result<R, Error>;
    fn dbg(self) -> Self;
}

impl<R> BadError<R> for Option<R> {
    fn bad_err(self, msg: impl Into<Cow<'static, str>>) -> Result<R, Error> {
        match self {
            Some(s) => Ok(s),
            None => Err(Error(msg.into())),
        }
    }
}

impl<R, E> BadError<R> for Result<R, E> {
    fn bad_err(self, msg: impl Into<Cow<'static, str>>) -> Result<R, Error> {
        match self {
            Ok(r) => Ok(r),
            Err(_) => Err(Error(msg.into())),
        }
    }
}

impl<R, E> InferBadError<R> for Result<R, E>
where
    E: Display,
{
    fn infer_err<'a>(self) -> Result<R, Error> {
        match self {
            Ok(r) => Ok(r),
            Err(e) => Err(Error(e.to_string().into())),
        }
    }

    fn dbg(self) -> Self {
        if let Err(e) = &self {
            println!("{}", e);
        }
        self
    }
}
