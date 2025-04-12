#![allow(dead_code)]
mod adb;
mod answerer;
mod context;
mod logging;
mod page;
mod utils;

use std::path::Path;

use adb::Adb;
use answerer::Multimodal;
use clap::Parser;
use context::Context;
use image::{GenericImageView, GrayImage, RgbaImage, buffer::ConvertBuffer};
use page::PageQuestion;

fn main() {
    let ctx = global_init();

    let mut adb = Adb::new(&ctx.adb);
    wait_question_page(&adb);

    let mut answerer = Multimodal::from_args(&ctx);
    let mut question_count = 0;
    while let Some((quesion, page)) = identify_screen(&adb) {
        question_count += 1;
        let ans = answerer.answer(&quesion);
        let choice = page.choice(ans);
        log::info!("{:03}: answer: {:?}, tap screen", question_count, ans);
        adb.tap_random(choice);
        std::thread::sleep(std::time::Duration::from_millis(600));
    }
    log::info!(
        "cost: question: {}: tokens: input: {}, output: {}, total: {}, {:.3}RMB",
        question_count,
        answerer.input_tokens(),
        answerer.output_tokens(),
        answerer.tokens(),
        answerer.input_tokens() as f64 / 1_000_000f64 * ctx.api_cost_input
            + answerer.output_tokens() as f64 / 1_000_000f64 * ctx.api_cost_output
    );
}

fn wait_question_page(adb: &Adb) -> (RgbaImage, PageQuestion) {
    loop {
        log::info!("Waiting for question page...");
        if let Some(val) = identify_screen(adb) {
            log::info!("Question page detected");
            return val;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn identify_screen(adb: &Adb) -> Option<(RgbaImage, PageQuestion)> {
    let screen = adb.screencap();
    let edges = edge_detection(&screen);
    if let Some(question) = PageQuestion::match_page(&edges) {
        let core = &question.core;
        let core = screen
            .view(
                core.left() as u32,
                core.top() as u32,
                core.width(),
                core.height(),
            )
            .to_image();
        return Some((core, question));
    }
    None
}

fn edge_detection(img: &RgbaImage) -> GrayImage {
    let gray: GrayImage = img.convert();
    let edges = imageproc::edges::canny(&gray, 50.0, 150.0);
    edges
}

fn global_init() -> Context {
    let start_time = std::time::Instant::now();
    dotenv();
    let ctx = Context::parse();
    logging::init(&ctx, start_time);
    ctx
}

fn dotenv() {
    let env_file = std::env::var("BILI_LV6_HARDCORE_DOTENV").unwrap_or(".env".to_owned());
    let env_file = Path::new(&env_file);
    if env_file.is_file() {
        dotenvy::from_path(env_file).unwrap();
    }
}
