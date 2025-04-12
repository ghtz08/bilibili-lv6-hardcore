#![allow(dead_code)]
mod adb;
mod answerer;
mod context;
mod logging;
mod page;
mod utils;

use answerer::Multimodal;
use clap::Parser;
use context::Context;
use image::{GenericImageView, Rgba};
use imageproc::drawing::draw_hollow_rect_mut;
use page::PageQuestion;

fn main() {
    let start_time = std::time::Instant::now();
    dotenvy::dotenv().ok();
    let ctx = Context::parse();
    logging::init(&ctx, start_time);

    let mut img_rgb = image::open("target/lv6-hardcore.png").unwrap();
    let gray = img_rgb.to_luma8();
    let edges = imageproc::edges::canny(&gray, 50.0, 150.0);

    let question = time("location", || PageQuestion::match_page(&edges)).unwrap();
    for rect in &question.check_boxes {
        let rect = rect.clone().into();
        draw_hollow_rect_mut(&mut img_rgb, rect, Rgba([255, 0, 0, 255]));
    }
    draw_hollow_rect_mut(
        &mut img_rgb,
        question.core.clone().into(),
        Rgba([255, 0, 0, 255]),
    );

    let img = img_rgb.crop(
        question.core.left() as u32,
        question.core.top() as u32,
        question.core.width(),
        question.core.height(),
    );

    let mut answer = Multimodal::new(
        ctx.api_url.clone(),
        ctx.api_model.clone(),
        ctx.api_key.clone(),
    );
    answer.answer(&img);
}

fn time<T>(name: &str, func: impl FnOnce() -> T) -> T {
    let begin_time = std::time::Instant::now();
    let res = func();
    let end_time = std::time::Instant::now();
    println!(
        "{}: {:.3}ms",
        name,
        end_time.duration_since(begin_time).as_micros() as f64 / 1000.0
    );
    res
}
