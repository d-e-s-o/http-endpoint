// Copyright (C) 2020 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

#[macro_use]
mod endpoint;
mod error;

use std::borrow::Cow;

pub use endpoint::Endpoint;
pub use error::Error;

pub type Str = Cow<'static, str>;
pub type Bytes = Cow<'static, [u8]>;
