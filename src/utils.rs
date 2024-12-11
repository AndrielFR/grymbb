// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains some utility functions.

use std::path::Path;

use bytes::Bytes;
use ferogram::Result;
use grammers_client::button::{self, Inline};
use reqwest::header::{HeaderMap, CONTENT_DISPOSITION, CONTENT_TYPE, USER_AGENT};
use serde_json::json;
use tokio_uring::fs::File;
use uuid::Uuid;

/// The URL of the API to take screenshots.
const API_URL: &str = "https://htmlcsstoimage.com/demo_run";

/// Convert a size in bytes to a human readable format.
pub fn human_readable_size(size: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

    let size = size as f64;
    let i = (size.ln() / 1024_f64.ln()).floor() as i32;
    let size = size / 1024_f64.powi(i);

    format!("{:.2} {}", size, units[i as usize])
}

/// Convert a board to inline buttons.
pub fn board_to_buttons(board: Vec<Vec<char>>, game_id: i32) -> Vec<Vec<Inline>> {
    board
        .into_iter()
        .enumerate()
        .map(|(column, row)| {
            row.into_iter()
                .enumerate()
                .map(|(row, symbol)| {
                    button::inline(symbol, format!("ttt {0} {1} {2}", game_id, column, row))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

/// Take a screenshot of the given URL.
pub async fn take_a_screenshot(url: String) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36".parse().unwrap());

    let data = json!({
        "url": url,
        "css": format!("random-tag: {}", Uuid::new_v4()),
        "render_when_ready": false,
        "viewport_width": 1280,
        "viewport_height": 720,
        "device_scale": 1,
    });

    let request = reqwest::Client::new()
        .post(API_URL)
        .headers(headers)
        .json(&data);

    match request.send().await {
        Ok(response) => {
            let json = response.json::<serde_json::Value>().await?;
            let photo_url = json["url"].as_str().unwrap();

            Ok(photo_url.to_string())
        }
        _ => Err("Failed to take screenshot".into()),
    }
}

/// Download a file from the given URL to the given path.
pub async fn download_file<U: ToString, P: AsRef<Path>>(url: U, path: P) -> Result<()> {
    let url = url.to_string();
    let path = path.as_ref();

    let response = reqwest::get(&url).await?;

    if !path.exists() {
        tokio_uring::fs::create_dir_all(path).await?;
    }

    let file_name = if let Some(disposition) = response.headers().get(CONTENT_DISPOSITION) {
        let disposition = disposition.to_str().unwrap();
        let file_name = disposition
            .split("filename=")
            .last()
            .unwrap_or("file")
            .replace("\"", "");

        file_name
    } else {
        url.split("/").last().unwrap_or("file").to_string()
    };

    let file_path = if path.is_dir() {
        path.join(file_name)
    } else {
        path.to_path_buf()
    };
    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }

    let bytes = response.bytes().await?;

    let file = File::create(&file_path).await?;
    let (res, _) = file.write_all_at(bytes.to_vec(), 0).await;
    res?;

    file.sync_all().await?;
    file.close().await?;

    Ok(())
}

/// Fetch a stream from the given URL.
pub async fn fetch_stream<U: ToString>(url: U) -> Result<Stream> {
    let url = url.to_string();

    let response = reqwest::get(&url).await?;

    let file_name = if let Some(disposition) = response.headers().get(CONTENT_DISPOSITION) {
        let disposition = disposition.to_str().unwrap();
        let file_name = disposition
            .split("filename=")
            .last()
            .unwrap_or("file")
            .replace("\"", "");

        file_name
    } else {
        url.split("/").last().unwrap_or("file").to_string()
    };

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .unwrap_or(&"application/octet-stream".parse().unwrap())
        .to_str()
        .unwrap()
        .to_string();
    let content_length = response.content_length();

    Ok(Stream {
        bytes: response.bytes().await?,
        file_name,
        content_type,
        content_length,
    })
}

/// A stream of bytes with some metadata.
pub struct Stream {
    /// The bytes of the stream.
    bytes: Bytes,
    /// The file name of the stream.
    file_name: String,
    /// The content type of the stream.
    content_type: String,
    /// The content length of the stream.
    content_length: Option<u64>,
}

impl Stream {
    /// Gets the length of the stream.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Gets the file name of the stream.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Gets the content type of the stream.
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// Gets the content length of the stream.
    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    /// Gets the bytes of the stream as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    /// Checks if the stream is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}
