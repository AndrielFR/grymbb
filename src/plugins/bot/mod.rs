// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the bot plugins setup.

use ferogram::Dispatcher;

mod info;
mod purge;
mod screenshot;
mod start;
mod tic_tac_toe;

pub fn setup(dp: Dispatcher) -> Dispatcher {
    dp.router(|_| info::setup())
        .router(|_| purge::setup())
        .router(|_| screenshot::setup())
        .router(|_| start::setup())
        .router(|_| tic_tac_toe::setup())
}
