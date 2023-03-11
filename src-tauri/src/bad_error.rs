#[allow(unused_imports)]
use crate::{dbg, debug, error};

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

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
            error!("{}", e);
        }
        self
    }
}

pub trait Inspectable<R> {
    fn look(self, f: impl FnOnce(&Self) -> R) -> Self;
}

impl<T, R> Inspectable<R> for T
where
    T: Debug,
{
    fn look(self, f: impl FnOnce(&Self) -> R) -> Self {
        let _r = f(&self);
        self
    }
}

pub trait InspectableErr<T> {
    fn look_err(self, f: impl FnOnce(&Self) -> T) -> Self;
}

impl<R, E, T> InspectableErr<T> for Result<R, E>
where
    E: Debug,
{
    fn look_err(self, f: impl FnOnce(&Self) -> T) -> Self {
        let _r = f(&self);
        self
    }
}

pub trait Loggable {
    fn log(self) -> Self;
}

pub trait LoggableResult {
    fn log_r(self) -> Self;
}

impl<T> Loggable for T
where
    T: Debug,
{
    fn log(self) -> Self {
        dbg!(&self);
        self
    }
}

impl<R, E> LoggableResult for Result<R, E>
where
    E: Display,
    R: Debug,
{
    fn log_r(self) -> Self {
        match &self {
            Ok(r) => {
                dbg!(r);
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        self
    }
}
