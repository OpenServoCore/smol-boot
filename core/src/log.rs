#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        #[cfg(rtt_log)]
        defmt::trace!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(rtt_log)]
        defmt::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        #[cfg(rtt_log)]
        defmt::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        #[cfg(rtt_log)]
        defmt::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        #[cfg(rtt_log)]
        defmt::error!($($arg)*)
    };
}
