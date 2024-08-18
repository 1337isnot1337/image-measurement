extern crate euclid;
extern crate image;
extern crate piston_window;

use euclid::default::Point2D;
use piston_window::{
    clear, image as piston_image, line_from_to, text, Button, G2dTexture, MouseButton,
    MouseCursorEvent, PistonWindow, PressEvent, Texture, TextureSettings, Transformed,
    WindowSettings,
};
use std::fs::{self, OpenOptions};
use std::io::Write;

struct Line {
    start: Point2D<f64>,
    end: Point2D<f64>,
    distance: f64,
}

const POINT_THRESHOLD: f64 = 30.0; // Adjust this threshold as needed

fn points_are_close(p1: Point2D<f64>, p2: Point2D<f64>, threshold: f64) -> bool {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt() <= threshold
}

#[allow(clippy::too_many_lines)]
fn main() {
    let _ = fs::remove_file("connections.txt");
    let _ = fs::remove_file("points.txt");

    // Load the image
    let img = image::open("imsatop.jpg").unwrap().to_rgba8();
    let (width, height) = img.dimensions();

    // Setup the window
    let mut window: PistonWindow = WindowSettings::new("Photo Distance App", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Load font for text rendering
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let font_path = &assets.join("Basic-Regular.ttf");
    let mut glyphs = window.load_font(font_path).unwrap();

    // Create texture context and texture
    let mut texture_context = window.create_texture_context();
    let texture: G2dTexture =
        Texture::from_image(&mut texture_context, &img, &TextureSettings::new()).unwrap();

    // Variables for storing points and lines
    let mut points: Vec<(i32, euclid::Point2D<f64, euclid::UnknownUnit>)> = Vec::new();
    let mut lines = Vec::new();
    let mut current_point: Option<usize> = None;
    let mut current_cursor_pos: Option<Point2D<f64>> = None;
    let mut point_counter = 1; // Start numbering from 1

    while let Some(e) = window.next() {
        // Update current cursor position
        if let Some(pos) = e.mouse_cursor_args() {
            current_cursor_pos = Some(Point2D::new(pos[0], pos[1]));
        }

        // Handle mouse button events
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if let Some(cursor_pos) = current_cursor_pos {
                if let Some(start_index) = current_point {
                    // Create a line between the points
                    let start: euclid::Point2D<f64, euclid::UnknownUnit> = points[start_index].1;
                    let distance = ((start.x - cursor_pos.x).powi(2)
                        + (start.y - cursor_pos.y).powi(2))
                    .sqrt();

                    // Check if the second point is close to any existing point
                    let mut existing_point_index: Option<usize> = None;
                    for (index, &(_, point)) in points.iter().enumerate() {
                        if points_are_close(point, cursor_pos, POINT_THRESHOLD) {
                            existing_point_index = Some(index);
                            break;
                        }
                    }

                    let end_point_number;
                    if let Some(index) = existing_point_index {
                        end_point_number = points[index].0;
                    } else {
                        end_point_number = point_counter;
                        points.push((point_counter, cursor_pos));

                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("points.txt")
                            .unwrap();
                        writeln!(file, "{point_counter}").unwrap();

                        point_counter += 1;
                    }

                    lines.push(Line {
                        start,
                        end: cursor_pos,
                        distance,
                    });

                    // Output to connections.txt
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("connections.txt")
                        .unwrap();
                    writeln!(
                        file,
                        "{} {} {:.2}",
                        points[start_index].0, // Start point number
                        end_point_number,      // End point number
                        distance
                    )
                    .unwrap();

                    current_point = None;
                } else {
                    // Check if the first point is close to any existing point
                    let mut existing_point_index: Option<usize> = None;
                    for (index, &(_, point)) in points.iter().enumerate() {
                        if points_are_close(point, cursor_pos, POINT_THRESHOLD) {
                            existing_point_index = Some(index);
                            break;
                        }
                    }

                    if let Some(index) = existing_point_index {
                        current_point = Some(index);
                    } else {
                        current_point = Some(points.len());
                        points.push((point_counter, cursor_pos));

                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("points.txt")
                            .unwrap();
                        writeln!(file, "{point_counter}").unwrap();

                        point_counter += 1;
                    }
                }
            }
        }

        // Draw the image and lines
        window.draw_2d(&e, |c, g, device| {
            clear([1.0; 4], g);

            // Draw the image
            piston_image(&texture, c.transform, g);

            // Draw all fixed lines and distances
            for line in &lines {
                line_from_to(
                    [0.0, 0.0, 1.0, 1.0],
                    2.0,
                    [line.start.x, line.start.y],
                    [line.end.x, line.end.y],
                    c.transform,
                    g,
                );
                let mid_point = Point2D::new(
                    (line.start.x + line.end.x) / 2.0,
                    (line.start.y + line.end.y) / 2.0,
                );
                text::Text::new_color([0.0, 0.5, 0.5, 1.0], 16)
                    .draw(
                        &format!("{:.2} px", line.distance),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(mid_point.x, mid_point.y - 10.0),
                        g,
                    )
                    .unwrap();
            }

            // Draw current dynamic line and distance
            if let (Some(start_index), Some(cursor_pos)) = (current_point, current_cursor_pos) {
                let start = points[start_index].1;
                line_from_to(
                    [1.0, 0.0, 0.0, 1.0],
                    2.0,
                    [start.x, start.y],
                    [cursor_pos.x, cursor_pos.y],
                    c.transform,
                    g,
                );
                let distance =
                    ((start.x - cursor_pos.x).powi(2) + (start.y - cursor_pos.y).powi(2)).sqrt();
                let mid_point = Point2D::new(
                    (start.x + cursor_pos.x) / 2.0,
                    (start.y + cursor_pos.y) / 2.0,
                );
                text::Text::new_color([0.1, 0.2, 1.0, 1.0], 16)
                    .draw(
                        &format!("{distance:.2} px"),
                        &mut glyphs,
                        &c.draw_state,
                        c.transform.trans(mid_point.x, mid_point.y - 10.0),
                        g,
                    )
                    .unwrap();
            }

            // Update glyphs cache
            glyphs.factory.encoder.flush(device);
        });
    }
}
