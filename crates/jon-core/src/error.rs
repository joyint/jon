// Copyright (c) 2026 Joydev GmbH (joydev.com)
// SPDX-License-Identifier: MIT

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JonError {
    #[error(transparent)]
    Joy(#[from] joy_core::error::JoyError),

    #[error("template: {0}")]
    Template(String),

    #[error("{0}")]
    Other(String),
}
