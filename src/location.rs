use std::i32;

use image::GrayImage;
use imageproc::contours::{Contour, find_contours};
use num_traits::{AsPrimitive, Num, Zero};

pub(crate) struct Location {
    pub(crate) core: Rect<i32>,
    pub(crate) check_boxes: Vec<Rect<i32>>,
}

impl Location {
    pub(crate) fn new(gray: &GrayImage) -> Self {
        let edges = imageproc::edges::canny(&gray, 50.0, 150.0);
        let contours_vec = find_contours::<i32>(&edges);
        log::debug!("contours: {}", contours_vec.len());

        let contours = contours_vec.iter().filter(|x| {
            if x.points.len() < gray.width() as usize {
                return false;
            }
            let rect = bounding_rect(x);
            if rect.width < (gray.width() / 2) as i32 {
                return false;
            }
            true
        });

        let rects: Vec<_> = contours.map(|x| bounding_rect(x)).collect();
        log::debug!("rects: {}", rects.len());
        let rects = nms(&rects);
        log::debug!("nms: {}", rects.len());

        let core = location_core(
            &contours_vec,
            (rects[1].y + rects[1].height) as usize,
            rects[1].height as usize,
            gray.width() as usize,
            gray.height() as usize,
        );

        Location {
            core,
            check_boxes: rects,
        }
    }
}

fn location_core(
    contours: &[Contour<i32>],
    center: usize,
    box_h: usize,
    img_w: usize,
    img_h: usize,
) -> Rect<i32> {
    let mut point_count = vec![0; img_h];
    for contour in contours {
        for point in &contour.points {
            point_count[point.y as usize] += 1;
        }
    }

    let mut top_y = 0;
    let mut begin = center;
    const THRESHOLD: i32 = 42;
    for i in (0..center).rev() {
        if point_count[i] < THRESHOLD {
            continue;
        }
        if begin - i > box_h + 1 {
            let off = box_h / 4 * 3;
            top_y = begin.max(off) - off;
            break;
        }
        begin = i;
    }
    let mut bottom_y = img_h;
    let mut begin = center;
    for i in center..img_h {
        if point_count[i] < THRESHOLD {
            continue;
        }
        if i - begin > box_h + 1 {
            bottom_y = (begin + box_h / 2).min(img_h);
            break;
        }
        begin = i;
    }
    const ALIGIN: usize = 28;
    let h = bottom_y - top_y;
    assert!(h > ALIGIN, "height: {}", h);
    let h = h / ALIGIN * ALIGIN;
    let mut top_y = top_y + (ALIGIN - 1) / ALIGIN * ALIGIN;
    let bottom_y = bottom_y + (ALIGIN - 1) / ALIGIN * ALIGIN;

    if bottom_y - top_y > h {
        top_y += ALIGIN / 2;
    }

    Rect::new(0, top_y as i32, img_w as i32, h as i32)
}

fn nms<T: RectItem>(rects: &[Rect<T>]) -> Vec<Rect<T>> {
    let mut res = vec![];
    for src in rects {
        let mut suppression = false;
        for target in &res {
            if src.iou(target) > 0.6 {
                suppression = true;
                if src.area() > target.area() {
                    res.pop();
                    res.push(src.clone());
                }
                break;
            }
        }
        if !suppression {
            res.push(src.clone());
        }
    }
    res
}

fn bounding_rect(contour: &Contour<i32>) -> Rect<i32> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for point in &contour.points {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y > max_y {
            max_y = point.y;
        }
    }

    Rect::new(min_x, min_y, max_x + 1 - min_x, max_y + 1 - min_y)
}

pub(crate) trait RectItem: Num + Zero + Copy + AsPrimitive<f32> + Ord {}
impl<T: Num + Zero + Copy + AsPrimitive<f32> + Ord> RectItem for T {}
#[derive(Clone, Debug)]
pub(crate) struct Rect<T: RectItem> {
    x: T,
    y: T,
    width: T,
    height: T,
}

impl<T: RectItem> Rect<T> {
    fn new(x: T, y: T, width: T, height: T) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    fn area(&self) -> T {
        self.width * self.height
    }

    fn iou(&self, other: &Self) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        if x1 < x2 && y1 < y2 {
            let inter_area = (x2 - x1) * (y2 - y1);
            let union_area = self.area() + other.area() - inter_area;
            return inter_area.as_() / union_area.as_();
        } else {
            0.0
        }
    }
}

impl Into<imageproc::rect::Rect> for Rect<i32> {
    fn into(self) -> imageproc::rect::Rect {
        imageproc::rect::Rect::at(self.x, self.y).of_size(self.width as u32, self.height as u32)
    }
}
