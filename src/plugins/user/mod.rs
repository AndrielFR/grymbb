// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the user plugins setup.

use ferogram::Dispatcher;

mod dump;
mod eval;
mod info;
mod purge;
mod reverse_search;
mod screenshot;
mod sed;
mod tic_tac_toe;
mod upload;

pub fn setup(dp: Dispatcher) -> Dispatcher {
    dp.router(|_| dump::setup())
        .router(|_| eval::setup())
        .router(|_| info::setup())
        .router(|_| purge::setup())
        .router(|_| reverse_search::setup())
        .router(|_| screenshot::setup())
        .router(|_| sed::setup())
        .router(|_| tic_tac_toe::setup())
        .router(|_| upload::setup())
}
