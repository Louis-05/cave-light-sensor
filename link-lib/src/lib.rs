#![no_std]
pub mod error;

#[cfg(feature = "embedded")]
pub mod embedded_link;
#[cfg(feature = "embedded")]
pub use embedded_link::Link;

#[cfg(feature = "async")]
pub mod async_link;
#[cfg(feature = "async")]
pub use async_link::Link;
