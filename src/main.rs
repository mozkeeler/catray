extern crate clap;
extern crate tempfile;

use clap::Parser;
use tempfile::NamedTempFile;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

// aspect ratio is 1:1, so we only need width
struct BoundingBox {
    top_left: Point,
    width: usize,
}

impl BoundingBox {
    fn new(mut top_left: Point, width: usize, height: usize) -> BoundingBox {
        if width > height {
            let to_move_up = (width - height) / 2;
            if top_left.y < to_move_up {
                top_left.y = 0;
            } else {
                top_left.y -= to_move_up;
            }
            BoundingBox { top_left, width }
        } else {
            let to_move_left = (height - width) / 2;
            if top_left.x < to_move_left {
                top_left.x = 0;
            } else {
                top_left.x -= to_move_left;
            }
            BoundingBox {
                top_left,
                width: height,
            }
        }
    }

    fn to_string(&self) -> String {
        format!(
            "{}x{}+{}+{}",
            self.width, self.width, self.top_left.x, self.top_left.y,
        )
    }
}

fn bounding_box(points: Vec<Point>) -> BoundingBox {
    let mut min_x = usize::MAX;
    let mut max_x = 0;
    let mut min_y = usize::MAX;
    let mut max_y = 0;
    for point in points {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.y > max_y {
            max_y = point.y;
        }
    }
    // pad by 10%
    let width_padding = (max_x - min_x) / 10;
    let height_padding = (max_y - min_y) / 10;
    if min_x > width_padding {
        min_x -= width_padding;
    } else {
        min_x = 0;
    }
    max_x += width_padding;
    if min_y > height_padding {
        min_y -= height_padding;
    } else {
        min_y = 0;
    }
    max_y += height_padding;
    BoundingBox::new(Point::new(min_x, min_y), max_x - min_x, max_y - min_y)
}

fn read_points(points_string: String) -> Vec<Point> {
    let mut points = Vec::new();
    let x_coords = points_string.split(' ').skip(1).step_by(2);
    let y_coords = points_string.split(' ').skip(2).step_by(2);
    for (x_str, y_str) in x_coords.zip(y_coords) {
        if let (Ok(x), Ok(y)) = (x_str.parse::<usize>(), y_str.parse::<usize>()) {
            points.push(Point { x, y });
        }
    }
    points
}

fn copy_and_resize(source: &Path, size: usize, destination_dir: &str, index: usize) {
    let new_path = format!("{}/{:08}-{}.jpg", destination_dir, index, size);
    std::fs::copy(source, &new_path).unwrap();
    Command::new("mogrify")
        .arg("-resize")
        .arg(size.to_string())
        .arg(new_path)
        .status()
        .expect("mogrify failed");
}

fn process_one_cat(picture_path: PathBuf, destination_dir: &str, index: usize) -> Result<(), ()> {
    let mut points_path = picture_path.as_os_str().to_owned();
    points_path.push(".cat");
    let mut points_file = File::open(points_path).map_err(|_| ())?;
    let mut points = String::new();
    points_file.read_to_string(&mut points).map_err(|_| ())?;
    let bx = bounding_box(read_points(points));
    let picture_temporary_path = NamedTempFile::new().map_err(|_| ())?;
    std::fs::copy(&picture_path, picture_temporary_path.path()).map_err(|_| ())?;
    Command::new("mogrify")
        .arg("-crop")
        .arg(bx.to_string())
        .arg(picture_temporary_path.path())
        .status()
        .expect("mogrify failed");
    copy_and_resize(picture_temporary_path.path(), 200, destination_dir, index);
    copy_and_resize(picture_temporary_path.path(), 96, destination_dir, index);
    copy_and_resize(picture_temporary_path.path(), 48, destination_dir, index);
    copy_and_resize(picture_temporary_path.path(), 24, destination_dir, index);
    eprintln!("processed {}", picture_path.as_os_str().to_str().ok_or(()).map_err(|_| ())?);
    Ok(())
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    corpus: String,

    #[clap(short, long)]
    destination: String,

    #[clap(short, long, default_value_t = 0)]
    start: usize,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut index = args.start;
    for entry in std::fs::read_dir(args.corpus)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "jpg" {
                if let Ok(()) = process_one_cat(path, &args.destination, index) {
                    index += 1;
                }
            }
        }
    }
    Ok(())
}
