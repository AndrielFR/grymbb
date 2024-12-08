// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains the reverse search command handler.

use ferogram::{handler, Context, Filter, Result, Router};
use grammers_client::{
    types::{Downloadable, Media},
    InputMessage,
};
use maplit::hashmap;
use regex::Regex;
use reqwest::{
    header::{
        HeaderMap, ACCEPT, ACCEPT_LANGUAGE, CONNECTION, HOST, UPGRADE_INSECURE_REQUESTS, USER_AGENT,
    },
    multipart::{Form, Part},
};

use crate::{filters, modules::i18n::I18n};

/// Setup the reverse search command.
pub fn setup() -> Router {
    Router::default().handler(
        handler::new_message(filters::commands(&["rs", "reverse"]).and(filters::sudoers()))
            .then(reverse_search),
    )
}

/// The URL of the Google Images search by image.
const GOOGLE_IMAGE_URL: &str = "http://www.google.hr/searchbyimage/upload";

/// Get the headers for the Google Images search by image.
pub fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(HOST, "www.google.hr".parse().unwrap());
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(UPGRADE_INSECURE_REQUESTS, "1".parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36".parse().unwrap());
    headers.insert(
        ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"
            .parse()
            .unwrap(),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        "pt-BR,pt;q=0.9,en-US;q=0.8,en;q=0.7,zh-TW;q=0.6,zh;q=0.5"
            .parse()
            .unwrap(),
    );

    headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());

    headers.insert(
        "sec-ch-ua",
        "\"Chromium\";v=\"103\", \"Not A(Brand\";v=\"24\", \"Google Chrome\";v=\"103\""
            .parse()
            .unwrap(),
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());

    headers
}

/// Handles the reverse search command.
async fn reverse_search(ctx: Context, i18n: I18n) -> Result<()> {
    let t = |key: &str| i18n.translate(key);
    let t_a = |key: &str, args| i18n.translate_with_args(key, args);

    let client = ctx.client();
    let req_client = reqwest::Client::new();

    if let Some(reply) = ctx.get_reply().await? {
        if let Some(media) = reply.media() {
            match media {
                Media::Photo(ref photo) => {
                    ctx.edit(t("downloading_photo")).await?;

                    let mut bytes = Vec::with_capacity(photo.size() as usize);

                    let mut iter = client.iter_download(&Downloadable::Media(media));
                    while let Some(chunk) = iter.next().await? {
                        bytes.extend(chunk);
                    }

                    ctx.edit(t("searching_photo")).await?;

                    let request = req_client
                        .post(GOOGLE_IMAGE_URL)
                        .headers(get_headers())
                        .multipart(
                            Form::new()
                                .part("encoded_image", Part::bytes(bytes))
                                .part("image_content", Part::text("image/jpeg")),
                        );
                    if let Ok(response) = request.send().await {
                        let text = response.text().await?;

                        let re = Regex::new(r#"value="(.*?)" aria-label="Pesquisar""#).unwrap();
                        let captures = re.captures(&text).unwrap();

                        let url = captures.get(0).unwrap().as_str();
                        let title = captures.get(1).unwrap().as_str();

                        ctx.edit(InputMessage::html(t_a(
                            "search_result",
                            hashmap! {"url" => url, "title" => title},
                        )))
                        .await?;
                    } else {
                        ctx.edit(t("search_error")).await?;
                    }
                }
                _ => {
                    ctx.reply(t("reply_not_photo")).await?;
                }
            }
        } else {
            ctx.reply(t("reply_not_photo")).await?;
        }
    } else {
        ctx.reply(t("reply_needed")).await?;
    }

    Ok(())
}
