// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ferogram::{Dispatcher, Injector};
use grammers_client::Client;

mod bot;
mod user;

pub fn bot(user: Client, mut resources: Injector) -> Dispatcher {
    resources.insert(user);
    bot::setup(Dispatcher::default().dependencies(|_| resources))
}

pub fn user(bot: Client, mut resources: Injector) -> Dispatcher {
    resources.insert(bot);
    user::setup(
        Dispatcher::default()
            .dependencies(|_| resources)
            .allow_from_self(),
    )
}
