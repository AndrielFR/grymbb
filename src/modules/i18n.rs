// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the internationalization module.

use std::{collections::HashMap, fs, sync::Arc};

use serde_json::Value;
use tokio::sync::Mutex;

const PATH: &str = "./assets/locales/";

/// Internationalization module.
#[derive(Clone)]
pub struct I18n {
    current_locale: Arc<Mutex<String>>,
    default_locale: String,

    locales: HashMap<String, Value>,
}

impl I18n {
    /// Creates a new `I18n` instance.
    pub fn with(default_locale: impl Into<String>) -> Self {
        let default_locale = default_locale.into();

        Self {
            current_locale: Arc::new(Mutex::new(default_locale.clone())),
            default_locale,

            locales: HashMap::new(),
        }
    }

    /// Loads the locales.
    pub fn load(&mut self) {
        let locales = fs::read_dir(PATH)
            .expect("Failed to read locales directory.")
            .map(|f| {
                f.expect("Failed to read file.")
                    .file_name()
                    .to_str()
                    .expect("Failed to convert file name.")
                    .split_once(".")
                    .expect("Failed to split file name.")
                    .0
                    .to_owned()
            })
            .collect::<Vec<String>>();

        for locale in locales.into_iter() {
            let path = format!("{0}/{1}.json", PATH, locale);
            let content = fs::read_to_string(&path).expect("Failed to read file.");
            let object = serde_json::from_str::<Value>(&content).expect("Failed to parse JSON.");
            self.locales.insert(locale, object);
        }
    }

    #[allow(dead_code)]
    /// Reloads the locales.
    pub fn reload(&mut self) {
        self.locales.clear();
        self.load();
    }

    #[allow(dead_code)]
    /// Gets the current locale.
    pub fn locale(&self) -> String {
        self.current_locale.try_lock().unwrap().clone()
    }

    #[allow(dead_code)]
    /// Gets the avaiable locales.
    pub fn locales(&self) -> Vec<String> {
        self.locales.keys().cloned().collect()
    }

    /// Sets the current locale.
    pub fn set_locale(&self, locale: impl Into<String>) {
        let mut current_locale = self.current_locale.try_lock().unwrap();

        *current_locale = locale.into();
    }

    #[allow(dead_code)]
    /// Uses a locale in a context.
    pub fn with_locale(&self, locale: impl Into<String>) -> LocaleGuard<'_> {
        LocaleGuard::new(&self, locale)
    }

    /// Translates a key.
    pub fn translate(&self, key: impl Into<String>) -> String {
        let current_locale = self.current_locale.try_lock().unwrap();

        self.translate_from_locale(key, current_locale.to_string())
    }

    /// Translates a key with arguments.
    pub fn translate_with_args(
        &self,
        key: impl Into<String>,
        args: HashMap<&str, impl Into<String>>,
    ) -> String {
        let current_locale = self.current_locale.try_lock().unwrap();

        self.translate_from_locale_with_args(key, current_locale.to_string(), args)
    }

    /// Translates a key from a specific locale.
    pub fn translate_from_locale(
        &self,
        key: impl Into<String>,
        locale: impl Into<String>,
    ) -> String {
        let key = key.into();
        let locale = locale.into();

        let object = self.locales.get(&locale).map_or_else(
            || {
                self.locales
                    .get(&self.default_locale)
                    .expect("Default locale not found.")
            },
            |v| v,
        );
        let value = object.get(&key).map_or("KEY_NOT_FOUND", |v| {
            v.as_str().expect("Failed to convert value.")
        });

        value.to_string()
    }

    /// Translates a key from a specific locale with arguments.
    pub fn translate_from_locale_with_args(
        &self,
        key: impl Into<String>,
        locale: impl Into<String>,
        args: HashMap<&str, impl Into<String>>,
    ) -> String {
        let mut result = self.translate_from_locale(key, locale);

        for (key, value) in args.into_iter() {
            result = result.replace(&format!("${{{}}}", key), &value.into());
        }

        result
    }
}

/// Contextual use of i18n.
pub struct LocaleGuard<'a> {
    i18n: &'a I18n,
    previous_locale: String,
}

impl<'a> LocaleGuard<'a> {
    #[allow(dead_code)]
    /// Creates a new `LocaleGuard` instance.
    pub fn new(i18n: &'a I18n, locale: impl Into<String>) -> Self {
        let previous_locale = i18n.locale();
        i18n.set_locale(locale.into());

        Self {
            i18n,
            previous_locale,
        }
    }
}

impl<'a> Drop for LocaleGuard<'a> {
    fn drop(&mut self) {
        self.i18n.set_locale(&self.previous_locale);
    }
}
