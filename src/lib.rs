#![deny(unused_must_use, unconditional_recursion)]
#![deny(clippy::clone_on_copy)]
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic)]
#![warn(clippy::allow_attributes)]
#![allow(dead_code, async_fn_in_trait)]
#![allow(
    // We won't release
    clippy::cargo_common_metadata,
    clippy::missing_docs_in_private_items,
    // Detection is not smart
    clippy::cognitive_complexity,
    // Sometimes useful
    clippy::enum_glob_use,
    clippy::wildcard_imports,
    clippy::iter_on_single_items,
    clippy::multiple_crate_versions,
    clippy::single_call_fn,
    clippy::unreadable_literal,
    // Sometimes annoying
    clippy::use_self,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]
#![feature(
    bool_to_result,
    error_generic_member_access,
    min_specialization,
    new_range_api,
    return_type_notation,
    trait_alias,
    try_blocks,
    variant_count
)]

pub mod application;
pub mod constant;
pub mod domain;
pub mod infra;
pub mod presentation;
pub mod utils;
