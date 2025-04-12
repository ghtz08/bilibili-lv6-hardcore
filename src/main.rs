#![allow(dead_code)]
mod adb;
mod answerer;
mod context;
mod logging;
mod page;
mod utils;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use adb::Adb;
use answerer::Multimodal;
use clap::Parser;
use context::Context;
use image::{GenericImageView, GrayImage, RgbaImage, buffer::ConvertBuffer};
use imageproc::drawing::draw_hollow_rect_mut;
use page::PageQuestion;

fn main() {
    let ctx = global_init();

    let mut adb = Adb::new(&ctx.adb);
    if let Some(device) = &ctx.device {
        adb.set_device(device.to_owned());
    }
    wait_question_page(&adb);

    let mut answerer = Multimodal::from_args(&ctx);
    let mut question_count = 0;
    const INTERVAL: Duration = Duration::from_millis(600);
    loop {
        let mut res = None;
        const IDENTIFY_TRY_LIMIT: usize = 2;
        for i in 0..IDENTIFY_TRY_LIMIT {
            if i != 0 {
                std::thread::sleep(INTERVAL);
            }
            if let Some(val) = identify_screen(&adb, &ctx.debug_save_path) {
                res = Some(val);
                break;
            }
        }
        if res.is_none() {
            log::info!("No question page detected");
            break;
        }
        let (question, page) = res.unwrap();

        question_count += 1;
        let ans = answerer.answer(&question);
        let choice = page.choice(ans);
        log::info!("{:03}: answer: {:?}, tap screen", question_count, ans);
        adb.tap_random(choice);
        std::thread::sleep(INTERVAL);
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
        if let Some(val) = identify_screen(adb, &None) {
            log::info!("Question page detected");
            return val;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn identify_screen(adb: &Adb, save_error: &Option<PathBuf>) -> Option<(RgbaImage, PageQuestion)> {
    let screen = adb.screencap();
    let mut edges = edge_detection(&screen);
    match PageQuestion::match_page(&edges) {
        Ok(question) => {
            let core = &question.core;
            let core = screen
                .view(
                    core.left() as u32,
                    core.top() as u32,
                    core.width(),
                    core.height(),
                )
                .to_image();
            Some((core, question))
        }
        Err(rects) => {
            if let Some(save_path) = save_error {
                log::debug!("save error image: {:?}", save_path);
                if !save_path.exists() {
                    std::fs::create_dir_all(save_path).unwrap();
                }
                for rect in &rects {
                    draw_hollow_rect_mut(&mut edges, *rect, image::Luma([255]));
                }
                let name = format!(
                    "screen-{}.jpg",
                    chrono::Local::now().format("%y%m%d-%H%M%S")
                );
                edges.save(save_path.join(name)).unwrap();
            }
            None
        }
    }
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
