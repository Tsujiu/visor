//! Módulo de integração com o sistema operacional Windows.

#![allow(unused_imports)]

pub mod assoc;
pub mod clipboard;
pub mod config;
pub mod explorer;
pub mod i18n;
pub mod paths;
pub mod security;
pub mod shell_sta;
pub mod trash;

pub use assoc::register_and_store;
pub use config::{AppSettings, MouseWheelAction, SettingsWriter};
pub use explorer::{open_in_browser, open_with_dialog, reveal_in_explorer};
pub use i18n::{AppLanguage, I18nStrings, strings};
pub use security::{SecurityStatus, enforce_startup_security, get_security_status};
