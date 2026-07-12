#![no_std]

//! Platform-independent music-domain types.
//!
//! This crate deliberately depends only on `core`: its public API must not
//! require an allocator, operating system, or I/O implementation. Owned
//! collections belong in higher-level crates behind an explicit `alloc`
//! boundary.

pub mod chord;
pub mod event;
pub mod frequency;
pub mod note;
pub mod pitch;
pub mod scale;
pub mod time;
