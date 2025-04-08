use image::GrayImage;
use imageproc::contours::{Contour, find_contours};
use num_traits::{AsPrimitive, Num, Zero};

pub(crate) struct Location {
    // all: Rect<i32>,
    pub(crate) check_boxes: Vec<Rect<i32>>,
}

impl Location {
    pub(crate) fn new(gray: &GrayImage) -> Self {
        let edges = imageproc::edges::canny(&gray, 50.0, 150.0);
        let contours = find_contours::<i32>(&edges);
        println!("contours: {}", contours.len());

        let contours = contours.iter().filter(|x| {
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
        println!("rects: {}", rects.len());
        let rects = nms(&rects);
        println!("nms: {}", rects.len());

        Location { check_boxes: rects }
    }
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
