use std::{io::Cursor, time::Duration};

use base64::{Engine, prelude::BASE64_STANDARD};
use image::{ImageFormat, RgbImage, RgbaImage, buffer::ConvertBuffer};
use rand::Rng;
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
    thinking: bool,

    prompt_tokens: u64,
    completion_tokens: u64,

    answer_total_count: u32,
    answer_fallback_count: u32,
    answer_fallback_ratio: f32,
}

impl Multimodal {
    pub fn from_args(ctx: &Context) -> Self {
        let url = ctx.api_url.clone();
        let model = ctx.api_model.clone();
        let key = ctx.api_key.clone();
        Self::new(
            url,
            model,
            key,
            ctx.answer_fallback_ratio,
            ctx.answer_thinking,
        )
    }
    pub fn new(
        url: String,
        model: String,
        key: String,
        fallback_ratio: f32,
        thinking: bool,
    ) -> Self {
        let client = Client::new();
        Self {
            url,
            model,
            key,
            client,
            thinking,
            prompt_tokens: 0,
            completion_tokens: 0,
            answer_total_count: 0,
            answer_fallback_count: 0,
            answer_fallback_ratio: fallback_ratio,
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

        self.answer_total_count += 1;
        parse_answer(message).unwrap_or_else(|| {
            self.answer_fallback_count += 1;
            let limit = self.answer_fallback_ratio;
            let ratio = self.answer_fallback_count as f32 / self.answer_total_count as f32;
            assert!(
                ratio <= limit,
                "fallback ratio exceeded: {ratio} > {}: {message}",
                limit
            );
            log::warn!("failed to parse answer: {message}, use random answer");
            Answer::random()
        })
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
        let prompt = "回答图片里的选择题，你的回答会被代码解析，直接输出你认为最合适的选项字母，仅输出选项字母，不需要多余的解释，即使不确定也必须选择一个选项。";
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
            "thinking": {
                "type": if self.thinking { "enabled" } else { "disabled" },
            },
        });
        let body = body.to_string();
        log::debug!("request {}", self.url);
        let resp = self
            .client
            .post(&self.url)
            .headers(headers)
            .body(body)
            .timeout(Duration::from_secs(600));
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

impl Answer {
    pub fn random() -> Self {
        match rand::rng().random_range(0..4) {
            0 => Answer::A,
            1 => Answer::B,
            2 => Answer::C,
            _ => Answer::D,
        }
    }
}

fn parse_answer(arg_answer: &str) -> Option<Answer> {
    let answer = arg_answer.trim();
    let answer = if answer.len() == 1 {
        answer.chars().next().unwrap()
    } else {
        let pos = answer.find("答案")?;
        let mut ans = ' ';
        for c in answer[pos..].chars() {
            if c.is_ascii_alphabetic() {
                ans = c;
                break;
            }
        }
        ans
    };

    match answer.to_ascii_uppercase() {
        'A' => Some(Answer::A),
        'B' => Some(Answer::B),
        'C' => Some(Answer::C),
        'D' => Some(Answer::D),
        _ => {
            log::warn!("Unknown answer: {arg_answer}");
            None
        }
    }
}
