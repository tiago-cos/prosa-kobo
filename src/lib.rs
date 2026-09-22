#![allow(clippy::unreadable_literal)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::module_inception)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use config::Configuration;
use std::sync::LazyLock;

pub mod app;
pub mod client;
pub mod config;
pub mod database;

pub static CONFIG: LazyLock<Configuration> =
    LazyLock::new(|| Configuration::new().expect("Failed to load configuration"));
