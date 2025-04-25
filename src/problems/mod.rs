//! Module containing problem definitions for various build systems.
//!
//! This module provides problems that can be identified in build logs,
//! organized by the build system or tool they relate to.

/// Problems specific to autopkgtest logs.
pub mod autopkgtest;

/// Common problems that can occur in various build environments.
pub mod common;

/// Problems specific to Debian packaging and build tools.
pub mod debian;
