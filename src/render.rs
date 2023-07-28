use std::vec;

use crate::color::Color;

use crate::constant::{HEIGHT, WIDTH};
use crate::math::{Mat4, Vec2, Vec3, Vec4};

pub struct Vertex {
    pub position: Vec4,
}
pub fn draw_pixel(x: u32, mut y: u32, buffer: &mut Vec<u8>, color: Color) {
    y = HEIGHT - y - 1;
    buffer[((WIDTH * y + x) * 3) as usize] = (color.r * 255.) as u8;
    buffer[((WIDTH * y + x) * 3) as usize + 1] = (color.g * 255.) as u8;
    buffer[((WIDTH * y + x) * 3) as usize + 2] = (color.b * 255.) as u8;
}

pub fn draw_line(
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    buffer: &mut Vec<u8>,
    color: Color,
) {
    let mut xy_reverse = false;
    if (x1 - x2).abs() < (y1 - y2).abs() {
        (x1, x2, y1, y2) = (y1, y2, x1, x2);
        xy_reverse = true;
    }
    if x1 >= x2 && y1 >= y2 {
        (x1, x2, y1, y2) = (x2, x1, y2, y1);
    }
    let mut x_reverse = false;
    if x1 > x2 {
        x_reverse = true
    }
    if y1 > y2 {
        x_reverse = true;
        (x1, x2, y1, y2) = (x2, x1, y2, y1);
    }

    let dx = if x_reverse { x1 - x2 } else { x2 - x1 };
    let kx = dx << 1;
    let ky = (y2 - y1) << 1;
    let mut y0 = y1;
    let mut delta = 0;
    let mut middle = dx;
    #[cfg(feature = "info")]
    println!("line:[({},{}),({},{})]", x1, y1, x2, y2);
    for x0 in if !x_reverse {
        (x1..x2).collect::<Vec<i32>>()
    } else {
        (x2..x1).rev().collect::<Vec<i32>>()
    } {
        if xy_reverse {
            draw_pixel(y0 as u32, x0 as u32, buffer, color);
        } else {
            draw_pixel(x0 as u32, y0 as u32, buffer, color);
        }
        delta += ky;
        if delta > middle {
            y0 += 1;
            middle += kx;
        }
    }
}

const INSIDE: u8 = 0; // 0000
const LEFT: u8 = 1; // 0001
const RIGHT: u8 = 2; // 0010
const BOTTOM: u8 = 4; // 0100
const TOP: u8 = 8; // 1000
pub fn endpoint_code(p: &Vec2, rec_left_top: &Vec2, rec_right_bot: &Vec2) -> u8 {
    let hc = if p.x < rec_left_top.x {
        LEFT
    } else if p.x > rec_right_bot.x {
        RIGHT
    } else {
        INSIDE
    };
    let vc = if p.y < rec_right_bot.y {
        BOTTOM
    } else if p.y > rec_left_top.y {
        TOP
    } else {
        INSIDE
    };
    hc | vc
}

pub fn clip_line(
    line: (Vec2, Vec2),
    rec_left_top: &Vec2,
    rec_right_bot: &Vec2,
) -> Option<(Vec2, Vec2)> {
    let p0_code = endpoint_code(&line.0, rec_left_top, rec_right_bot);
    let p1_code = endpoint_code(&line.1, rec_left_top, rec_right_bot);
    if p0_code & p1_code != 0 {
        return None;
    } else if p0_code | p1_code == 0 {
        return Some(line);
    }

    return None;
}

pub fn draw_triangle(triangle: &[Vertex; 3], buffer: &mut Vec<u8>) {
    rasterization(triangle, buffer);
    for i in 0..triangle.len() {
        draw_line(
            triangle[i].position.x as i32,
            triangle[i].position.y as i32,
            triangle[(i + 1) % 3].position.x as i32,
            triangle[(i + 1) % 3].position.y as i32,
            buffer,
            Color::RED,
        )
    }
}

fn inside_triangle(x: f32, y: f32, triangle: &[Vertex; 3]) -> bool {
    let mut flag = 0;
    for i in 0..triangle.len() {
        let x1 = triangle[i].position.x;
        let y1 = triangle[i].position.y;
        let x2 = triangle[(i + 1) % 3].position.x;
        let y2 = triangle[(i + 1) % 3].position.y;
        let v1 = Vec2::new(x2 - x1, y2 - y1);
        let v2 = Vec2::new(x - x1, y - y1);
        if v1.cross(v2).is_sign_negative() {
            if flag == 1 {
                return false;
            } else {
                flag = -1
            }
        } else {
            if flag == -1 {
                return false;
            } else {
                flag = 1
            }
        };
    }
    return true;
}

pub fn rasterization(triangle: &[Vertex; 3], buffer: &mut Vec<u8>) {
    let x_min = triangle.iter().map(|x| x.position.x as u32).min().unwrap();
    let y_min = triangle.iter().map(|x| x.position.y as u32).min().unwrap();

    let x_max = triangle.iter().map(|x| x.position.x as u32).max().unwrap();
    let y_max = triangle.iter().map(|x| x.position.y as u32).max().unwrap();
    for x in x_min..x_max {
        for y in y_min..y_max {
            if inside_triangle(x as f32, y as f32, triangle) {
                draw_pixel(x, y, buffer, Color::WHITE);
            }
        }
    }
    println!("rasterization:{},{},{},{}", x_min, y_min, x_max, y_max);
}




pub fn model_transform(triangle: &mut [Vertex; 3], model_matrix: Mat4) {
    for i in 0..triangle.len() {
        triangle[i].position = model_matrix * triangle[i].position;
    }
}

pub fn view_transform(triangle: &mut [Vertex; 3], veiw_matrix: Mat4) {
    for i in 0..triangle.len() {
        triangle[i].position = veiw_matrix * triangle[i].position;
    }
}

pub fn projection(triangle: &mut [Vertex; 3], veiw_matrix: Mat4) {
    for i in 0..triangle.len() {
        triangle[i].position = veiw_matrix * triangle[i].position;
    }
}

pub fn mvp() {}
pub fn back_face_culling(triangle: &[Vertex; 3], view_dir: Vec3) {}
