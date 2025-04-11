mod adb;
mod context;
mod location;
mod logging;

use clap::Parser;
use context::Context;
use image::Rgba;
use imageproc::drawing::draw_hollow_rect_mut;
use location::Location;

fn main() {
    let start_time = std::time::Instant::now();
    dotenvy::dotenv().ok();
    let ctx = Context::parse();
    logging::init(&ctx, start_time);

    let mut img_rgb = image::open("target/lv6-hardcore.png").unwrap();
    let gray = img_rgb.to_luma8();

    let location = time("location", || Location::new(&gray));
    println!("core: {:?}", location.core);
    println!("check_boxes: {:?}", location.check_boxes);
    for rect in &location.check_boxes {
        let rect = rect.clone().into();
        draw_hollow_rect_mut(&mut img_rgb, rect, Rgba([255, 0, 0, 255]));
    }
    draw_hollow_rect_mut(
        &mut img_rgb,
        location.core.clone().into(),
        Rgba([255, 0, 0, 255]),
    );

    img_rgb.save("target/contours.png").unwrap();
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
