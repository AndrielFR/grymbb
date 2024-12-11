// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the dump functions.

/// The trait for dumping.
pub trait Dump {
    /// Dump the object.
    fn dump(&self) -> String;
}

impl<T: std::fmt::Debug> Dump for T {
    fn dump(&self) -> String {
        format!("{:#?}", self)
    }
}
