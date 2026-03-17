pub(crate) mod afio;
pub(crate) mod flash;
pub mod gpio;
pub(crate) mod pfic;
pub mod rcc;
pub(crate) mod usart;

#[cfg(not(feature = "system-flash"))]
pub(crate) mod boot_request;
