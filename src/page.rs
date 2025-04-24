use std::{fmt::Display, ops::Sub};

use image::GrayImage;
use imageproc::{
    contours::{Contour, find_contours},
    point::Point,
    rect::Rect,
};
use num_traits::Bounded;

use crate::answerer::Answer;

pub(crate) struct PageQuestion {
    pub(crate) core: Rect,
    pub(crate) check_boxes: Vec<Rect>,
}

impl PageQuestion {
    pub(crate) fn choice(&self, ans: Answer) -> &Rect {
        &self.check_boxes[ans as usize]
    }

    pub(crate) fn match_page(edges: &GrayImage) -> Result<Self, Vec<Rect>> {
        let width = edges.width();
        let height = edges.height();

        let contours_vec = find_contours::<i32>(&edges);
        log::debug!("contours: {}", contours_vec.len());

        let contours = contours_vec.iter().filter(|x| {
            if x.points.len() < width as usize {
                return false;
            }
            let rect = bounding_rect(x);
            if rect.width() < width / 2 {
                return false;
            }
            true
        });

        let rects: Vec<_> = contours.map(|x| bounding_rect(x)).collect();
        log::debug!("rects: {}", rects.len());
        log::trace!("rects: {:?}", rects);
        let choices = nms(&rects);
        log::debug!("nms: {}", choices.len());
        log::trace!("choices: {:?}", choices);
        let choices = choices.into_iter().filter(|x| {
            // 排除宽高比过大或者过小的框
            let ratio = x.width() as f32 / x.height() as f32;
            5.0 <= ratio && ratio <= 8.0
        });
        let choices: Vec<_> = choices.collect();
        log::debug!("choices: {}", choices.len());

        const CHOICES_NUMBER: usize = 4;
        if choices.len() != CHOICES_NUMBER {
            return Err(choices);
        }

        // 所有的框需要差不多宽并且左右间距是一致的，左右间距一致防止截到过渡动画
        let same_w = is_difference_small(choices.iter().map(|x| x.width()), 3);
        let same_l = is_difference_small(choices.iter().map(|x| x.left()), 3);
        let same_lr_pad = (choices[1].left() - choices[0].right()).abs() < 3;
        if !same_w || !same_l || !same_lr_pad {
            log::debug!(
                "not same: w: {}, l: {}, lr: {}",
                same_w,
                same_l,
                same_lr_pad
            );
            return Err(choices);
        }

        let mut choices = choices;
        choices.sort_by_key(|x| x.top());
        for i in 1..choices.len() {
            if choices[i - 1].bottom() >= choices[i].top() {
                log::warn!("overlap: {} {}", choices[i - 1].bottom(), choices[i].top());
                return Err(choices);
            }
        }

        let core = location_core(
            &contours_vec,
            (choices[0].top() as u32 + choices[0].height()) as usize,
            choices[0].height() as usize,
            width as usize,
            height as usize,
        );
        log::trace!("core: {:?}", core);

        Ok(PageQuestion {
            core,
            check_boxes: choices,
        })
    }
}

fn is_difference_small<T>(data: impl Iterator<Item = T>, threshold: T) -> bool
where
    T: Bounded + PartialOrd + Copy + Sub<Output = T> + Display,
{
    let mut min = T::max_value();
    let mut max = T::min_value();
    let mut n = 0usize;
    for val in data {
        n += 1;
        if val < min {
            min = val;
        }
        if val > max {
            max = val;
        }
    }
    let small = n == 0 || max - min <= threshold;
    if !small {
        log::debug!("difference_small: {min}, {max}");
    }
    small
}

fn location_core(
    contours: &[Contour<i32>],
    center: usize,
    box_h: usize,
    img_w: usize,
    img_h: usize,
) -> Rect {
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

    Rect::at(0, top_y as i32).of_size(img_w as u32, h as u32)
}

fn nms(rects: &[Rect]) -> Vec<Rect> {
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

fn bounding_rect(contour: &Contour<i32>) -> Rect {
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

    Rect::at(min_x, min_y).of_size((max_x + 1 - min_x) as u32, (max_y + 1 - min_y) as u32)
}

pub(crate) trait RectExtra {
    fn area(&self) -> u64;
    fn iou(&self, other: &Self) -> f32;
    fn center(&self) -> Point<i32>;
    fn contains(&self, point: Point<i32>) -> bool;
}

impl RectExtra for Rect {
    fn area(&self) -> u64 {
        self.width() as u64 * self.height() as u64
    }

    fn iou(&self, other: &Self) -> f32 {
        let x1 = self.left().max(other.left());
        let y1 = self.top().max(other.top());
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());
        if x1 > x2 || y1 > y2 {
            0.0
        } else {
            let inter_area = (x2 - x1 + 1) * (y2 - y1 + 1);
            let union_area = self.area() + other.area() - inter_area as u64;
            (inter_area as f64 / union_area as f64) as f32
        }
    }

    fn center(&self) -> Point<i32> {
        let x = self.left() + self.width() as i32 / 2;
        let y = self.top() + self.height() as i32 / 2;
        Point::new(x, y)
    }

    fn contains(&self, point: Point<i32>) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }
}
