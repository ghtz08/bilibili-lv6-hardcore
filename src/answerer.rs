use std::io::Cursor;

use base64::{Engine, prelude::BASE64_STANDARD};
use clap::ValueEnum;
use image::{ImageFormat, RgbImage, RgbaImage, buffer::ConvertBuffer};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde_json::json;

use crate::{
    context::Context, json_at, json_value_as_i64, json_value_as_str, json_value_as_vec, parse_json,
};

#[derive(Default)]
pub struct Multimodal {
    url: String,
    model: String,
    key: String,
    client: Client,

    prompt_tokens: u64,
    completion_tokens: u64,
}

impl Multimodal {
    pub fn from_args(ctx: &Context) -> Self {
        let url = ctx.api_url.clone();
        let model = ctx.api_model.clone();
        let key = ctx.api_key.clone();
        Self::new(url, model, key)
    }
    pub fn new(url: String, model: String, key: String) -> Self {
        let client = Client::new();
        Self {
            url,
            model,
            key,
            client,
            prompt_tokens: 0,
            completion_tokens: 0,
        }
    }

    pub fn answer(&mut self, question: &RgbaImage) -> Answer {
        let resp = self.post(&question.convert());
        log::trace!("{}", serde_json::to_string(&resp).unwrap());
        let choices = json_value_as_vec!(json_at!(resp, "choices").unwrap()).unwrap();
        let choice = &choices[0];
        let usage = json_at!(resp, "usage").unwrap();
        let message = json_value_as_str!(json_at!(choice, "message", "content").unwrap()).unwrap();
        let prompt_tokens =
            json_value_as_i64!(json_at!(usage, "prompt_tokens").unwrap()).unwrap() as u64;
        let completion_tokens =
            json_value_as_i64!(json_at!(usage, "completion_tokens").unwrap()).unwrap() as u64;
        log::debug!("answer: {message:?}");
        log::debug!(
            "tokens: prompt: {prompt_tokens}, completion: {completion_tokens}, total: {}",
            prompt_tokens + completion_tokens
        );
        self.prompt_tokens += prompt_tokens;
        self.completion_tokens += completion_tokens;
        log::trace!(
            "acc tokens: prompt: {}, completion: {}, total: {}",
            self.prompt_tokens,
            self.completion_tokens,
            self.tokens()
        );

        let answer = message.trim();
        let answer = if answer.len() == 1 {
            answer.chars().next().unwrap()
        } else {
            let pos = answer.find("答案").unwrap();
            let mut ans = ' ';
            for c in answer[pos..].chars() {
                if c.is_ascii_alphabetic() {
                    ans = c;
                    break;
                }
            }
            ans
        };

        let mut buf = [0u8; 4];
        let answer: &str = answer.encode_utf8(&mut buf);
        Answer::from_str(answer, true).expect(answer)
    }

    pub fn input_tokens(&self) -> u64 {
        self.prompt_tokens
    }

    pub fn output_tokens(&self) -> u64 {
        self.completion_tokens
    }

    pub fn tokens(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }

    fn post(&self, question: &RgbImage) -> serde_json::Value {
        let img_base64 = image_to_jpeg_to_base64(&question);

        let headers = new_headers(&[
            ("Content-Type", "application/json"),
            ("Authorization", &format!("Bearer {}", self.key)),
        ]);
        let prompt = "回答图片里的选择题。你的回答会被代码解析，只需要回答选项，不需要多余的解释。需要保证正确性，不能随便回答。如果不确定答案，请回答正确的可能性最大的那个，即使不确定也不需要任何解释和说明";
        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/jpeg;base64,{}", img_base64),
                            },
                        },
                        {
                            "type":"text",
                            "text": prompt,
                        },
                    ],
                },
            ],
        });
        let body = body.to_string();
        log::debug!("request {}", self.url);
        let resp = self.client.post(&self.url).headers(headers).body(body);
        let resp = resp.send().unwrap();

        let resp_status = resp.status();
        let resp = resp.error_for_status();
        let resp = resp.expect(&format!("status: {}", resp_status));

        parse_json!(resp.text().unwrap())
    }
}

fn new_headers(headers: &[(&str, &str)]) -> HeaderMap {
    let mut req_headers = HeaderMap::new();
    for (key, value) in headers {
        let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
        let value = HeaderValue::from_str(value).unwrap();
        req_headers.insert(name, value);
    }
    req_headers
}

pub fn image_to_jpeg_to_base64(img: &RgbImage) -> String {
    let mut buf = Vec::new();
    let mut writer = Cursor::new(&mut buf);
    img.write_to(&mut writer, ImageFormat::Jpeg).unwrap();
    BASE64_STANDARD.encode(&buf)
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Answer {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}
