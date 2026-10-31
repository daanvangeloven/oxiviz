use crate::canvas::Canvas;
use crate::color::Color;
use crate::geometry::Point;

/// Draws a straight line with a square brush `width` pixels wide, clipped to the canvas.
pub(crate) fn draw_line(canvas: &mut Canvas, from: Point, to: Point, color: Color, width: u32) {
    for p in bresenham(from, to) {
        stamp(canvas, p, color, width);
    }
}

fn stamp(canvas: &mut Canvas, center: Point, color: Color, width: u32) {
    let width = width.max(1) as i32;
    let start = -(width - 1) / 2;
    for dy in start..start + width {
        for dx in start..start + width {
            canvas.set(Point::new(center.x + dx, center.y + dy), color);
        }
    }
}

/// The pixels of the line from `from` to `to`, both included.
fn bresenham(from: Point, to: Point) -> impl Iterator<Item = Point> {
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let step_x = if from.x < to.x { 1 } else { -1 };
    let step_y = if from.y < to.y { 1 } else { -1 };
    let mut error = dx + dy;
    let mut next = Some(from);

    std::iter::from_fn(move || {
        let current = next?;
        next = (current != to).then(|| {
            let mut p = current;
            let doubled = 2 * error;
            if doubled >= dy {
                error += dy;
                p.x += step_x;
            }
            if doubled <= dx {
                error += dx;
                p.y += step_y;
            }
            p
        });
        Some(current)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_includes_both_ends_and_is_connected() {
        let points: Vec<Point> = bresenham(Point::new(0, 0), Point::new(5, -3)).collect();
        assert_eq!(points.first(), Some(&Point::new(0, 0)));
        assert_eq!(points.last(), Some(&Point::new(5, -3)));
        assert!(
            points
                .windows(2)
                .all(|pair| { (pair[1].x - pair[0].x).abs() <= 1 && (pair[1].y - pair[0].y).abs() <= 1 })
        );
    }

    #[test]
    fn single_point_line() {
        assert_eq!(bresenham(Point::new(2, 2), Point::new(2, 2)).count(), 1);
    }
}
