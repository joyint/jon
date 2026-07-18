// Copyright (c) 2026 Joydev GmbH (joydev.com)
// SPDX-License-Identifier: MIT

//! Core library for Jon, the natural language layer of the Joyint
//! ecosystem. Extends joy-core with the product development assistant
//! (PDA): a guided, stateful session that turns an idea into a decided
//! product definition inside a Joy project (JON-0005-19). The
//! command-routing tiers (rule engine, embedded LLM) stay on the
//! subprocess `--json` contract and live in their own epics
//! (JON-0001-C1, JON-000D-58).
//!
//! Everything here is non-interactive shared logic, following the same
//! rule as joy-core's `ai_setup`: jon-cli wraps it with prompts and
//! terminal output, the platform calls it directly.

#![deny(clippy::all)]

pub mod bootstrap;
pub mod error;
pub mod pda;

pub use error::JonError;

// jon-cli and the platform reach joy-core through this re-export so
// they depend on one crate and cannot drift to a different joy-core.
pub use joy_core;
