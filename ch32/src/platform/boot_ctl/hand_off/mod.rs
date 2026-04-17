//! App hand-off strategies.
//!
//! Exactly one variant compiles per build, selected by the `system-flash` feature:
//! - [`system`]: software reset; factory ROM re-reads the boot request and dispatches.
//! - [`user`]: reset APB2 peripherals, then jump directly to the app's reset vector.

core::cfg_select! {
    feature = "system-flash" => {
        mod system;
        pub type Active = system::SystemHandOff;
    }
    _ => {
        mod user;
        pub type Active = user::UserHandOff;
    }
}
