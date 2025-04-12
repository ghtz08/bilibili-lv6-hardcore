// 257bf64 unauthorized
// 257bf64 device

use std::process::Command;

use image::RgbaImage;
use imageproc::{point::Point, rect::Rect};
use more_asserts::{assert_ge, assert_gt};
use rand::{SeedableRng, rngs::StdRng};
use rand_distr::Distribution;

use crate::page::RectExtra;

pub struct Adb {
    adb: String,
    device: String,
    rng: StdRng,
}

impl Adb {
    pub fn new(adb: &str) -> Self {
        let adb = if adb.find(std::path::MAIN_SEPARATOR).is_none() {
            adb.to_owned()
        } else {
            which(adb)
        };
        let rng = StdRng::from_rng(&mut rand::rng());
        Adb {
            adb,
            device: "".to_owned(),
            rng,
        }
    }

    pub fn tap(&self, point: Point<i32>) {
        self.shell(&[
            "shell",
            "input",
            "tap",
            &point.x.to_string(),
            &point.y.to_string(),
        ]);
    }

    pub fn tap_random(&mut self, area: &Rect) {
        let point = self.random_point(area);
        self.tap(point);
    }

    fn random_point(&mut self, area: &Rect) -> Point<i32> {
        assert_ge!(area.left(), 0);
        assert_ge!(area.top(), 0);
        assert_ge!(area.width(), 3);
        assert_ge!(area.height(), 3);
        if area.width() < 5 || area.height() < 5 {
            return area.center();
        }
        const RATE: u32 = 6;
        let area = Rect::at(
            area.left() + (area.width() / RATE) as i32,
            area.top() + (area.height() / RATE) as i32,
        )
        .of_size(
            area.width() - area.width() / RATE * 2,
            area.height() - area.height() / RATE * 2,
        );

        const STD_DEV: f64 = 0.25;
        let normal = rand_distr::Normal::new(0.5, STD_DEV).unwrap();
        let (x, y) = loop {
            let x = normal.sample(&mut self.rng);
            let y = normal.sample(&mut self.rng);
            if 0.0 <= x && x < 1.0 && 0.0 <= y && y < 1.0 {
                break (x, y);
            }
        };
        let x = area.width() as f64 * x;
        let y = area.height() as f64 * y;
        Point::new(area.left() + x as i32, area.top() + y as i32)
    }

    pub fn screencap(&self) -> RgbaImage {
        let out = self.shell(&["exec-out", "screencap", "-p"]);
        let screen = image::load_from_memory(&out).unwrap();
        assert!(screen.color() == image::ColorType::Rgba8);
        screen.into_rgba8()
    }

    pub fn screen_size(&self) -> (u32, u32) {
        let out = self.shell(&["shell", "wm", "size"]);
        let out = String::from_utf8_lossy(&out);
        let mut size = out.split_ascii_whitespace().last().unwrap().split('x');
        let width = size.next().unwrap().parse::<u32>().unwrap();
        let height = size.next().unwrap().parse::<u32>().unwrap();
        assert_gt!(width, 0);
        assert_gt!(height, 0);
        (width, height)
    }

    pub fn set_device(&mut self, device: String) {
        self.device = device;
    }

    pub fn devices(&self) -> Vec<Device> {
        let stdout = self.shell(&["devices"]);
        let stdout = String::from_utf8_lossy(&stdout);
        let mut devices = vec![];
        for line in stdout.lines().skip(1) {
            let device = line.split_whitespace().next().unwrap_or_default();
            if device.is_empty() {
                continue;
            }
            if line.ends_with("device") {
                devices.push(Device::Device(device.to_string()));
            } else if line.ends_with("unauthorized") {
                devices.push(Device::Unauthorized(device.to_string()));
            } else {
                log::warn!("Unknown device: {}", line);
            }
        }
        log::info!("Devices: {:?}", devices);
        devices
    }

    fn shell(&self, args: &[&str]) -> Vec<u8> {
        let mut cmd = Command::new(&self.adb);
        if !self.device.is_empty() {
            cmd.arg("-s").arg(&self.device);
        }
        command(cmd.args(args))
    }
}

fn which(name: &str) -> String {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("powershell.exe");
        cmd.arg("-Command");
        cmd.arg(format!("(Get-Command '{}').Source", name));
        cmd
    } else if cfg!(unix) {
        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(format!("command -v '{}'", name));
        cmd
    } else {
        panic!("Unsupported platform");
    };
    String::from_utf8_lossy(&command(&mut cmd))
        .trim()
        .to_owned()
}

fn command(cmd: &mut Command) -> Vec<u8> {
    let out = cmd.output().expect(&format!(
        "Failed to execute command: {} {:?}",
        cmd.get_program().to_str().unwrap(),
        cmd.get_args()
    ));
    if out.status.success() {
        out.stdout
    } else {
        log::error!(
            "{} {:?}: {:?}",
            cmd.get_program().to_str().unwrap(),
            cmd.get_args(),
            out.status,
        );
        panic!(
            "{} {:?}\n{}",
            cmd.get_program().to_str().unwrap(),
            cmd.get_args(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[cfg(test)]
mod tests {
    use more_asserts::assert_le;

    use crate::logging;

    use super::*;
    #[test]
    fn test_devices() {
        logging::init_for_test();

        let mut adb = Adb::new("adb");
        let devices = adb.devices();
        assert!(!devices.is_empty(), "No devices found");
        for device in devices {
            if let Device::Device(device) = device {
                adb.set_device(device.clone());
                let out = adb.shell(&["shell", "pwd"]);
                let out = String::from_utf8_lossy(&out);
                println!("Device: {}: {}", device, out);
            }
        }
    }

    #[test]
    fn test_screen_size() {
        logging::init_for_test();
        let adb = Adb::new("adb");
        let (width, height) = adb.screen_size();
        println!("Screen size: {}x{}", width, height);
    }

    #[test]
    fn test_screencap() {
        logging::init_for_test();
        let adb = Adb::new("adb");
        let screen = adb.screencap();
        println!("Screen size: {:?}", screen.dimensions());
    }

    #[test]
    fn test_screencap_save() {
        logging::init_for_test();
        let adb = Adb::new("adb");
        let screen = adb.screencap();
        screen.save("screen.png").unwrap();
    }

    #[test]
    fn test_random_point() {
        logging::init_for_test();
        let mut adb = Adb::new("adb");
        let area = Rect::at(0, 0).of_size(200, 100);
        for i in 0..10000 {
            let point = adb.random_point(&area);
            assert_ge!(point.x, 33);
            assert_le!(point.x, 166);
            assert_ge!(point.y, 16);
            assert_le!(point.y, 83);
            if i < 10 {
                println!("Point: {:3}, {:3}", point.x, point.y);
            }
        }
    }

    #[test]
    fn test_tap() {
        logging::init_for_test();
        let adb = Adb::new("adb");
        let (width, height) = adb.screen_size();
        adb.tap(Point::new((width / 2) as i32, (height / 2) as i32));
    }
}

#[derive(Debug)]
enum Device {
    Unauthorized(String),
    Device(String),
}
