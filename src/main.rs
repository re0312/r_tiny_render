use image::{ColorType, ImageBuffer, Rgb};
use r_tiny_render::{
    color::{self, Color},
    math::Vec2,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn draw_pixel(x: u32, mut y: u32, buffer: &mut Vec<u8>, color: Color) {
    y = y.min(HEIGHT - 1);
    y = HEIGHT - y - 1;
    buffer[((WIDTH * y + x) * 3) as usize] = (color.r * 255.) as u8;
    buffer[((WIDTH * y + x) * 3) as usize + 1] = (color.g * 255.) as u8;
    buffer[((WIDTH * y + x) * 3) as usize + 2] = (color.b * 255.) as u8;
}

fn draw_line(
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    buffer: &mut Vec<u8>,
    color: Color,
) {
    if x1 >= x2 && y1 >= y2 {
        (x1, x2, y1, y2) = (x2, x1, y2, y1);
    }
    let mut x1y = false;
    if x1 > x2 {
        x1y = true
    }
    if y1 > y2 {
        x1y = true;
        (x1, x2, y1, y2) = (x2, x1, y2, y1);
    }

    let mut x2y = false;
    if (x1 - x2).abs() < (y1 - y2).abs() {
        (x1, x2, y1, y2) = (y1, y2, x1, x2);
        x2y = true;
    }
    let dx = if x1y { (x1 - x2) } else { (x2 - x1) };
    let kx = dx << 1;
    let ky = (y2 - y1) << 1;
    let mut y0 = y1;
    let mut delta = 0;
    let mut middle = dx;
    println!("{},{},{},{}", x1, x2, y1, y2);
    for x0 in if !x1y {
        (x1..=x2).collect::<Vec<i32>>()
    } else {
        (x2..=x1).rev().collect::<Vec<i32>>()
    } {
        if x2y {
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
fn endpoint_code(p: &Vec2, rec_left_top: &Vec2, rec_right_bot: &Vec2) -> u8 {
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

fn clip_line(
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

fn load_gltf() {}
fn load_obj(path: &str) {
    let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    let (models, materials) = tobj::load_obj(path, &tobj::LoadOptions::default()).unwrap();
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("");
        println!("model[{}].name             = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!(
            "model[{}].face_count       = {}",
            i,
            mesh.face_arities.len()
        );

        let scale =25.;
        for indices in 0..m.mesh.indices.len() / 3 {
            let i1 = mesh.indices[indices];
            let i2 = mesh.indices[indices + 1];
            let i3 = mesh.indices[indices + 2];
            let mut x1 = mesh.positions[i1 as usize * 3] / scale;
            let mut y1 = mesh.positions[i1 as usize * 3 + 1]/scale;
            let mut x2 = mesh.positions[i2 as usize * 3] /scale;
            let mut y2 = mesh.positions[i2 as usize * 3 + 1] /scale;
            let mut x3 = mesh.positions[i3 as usize * 3]/ scale;
            let mut y3 = mesh.positions[i3 as usize * 3 + 1]/scale;
            let x1 = ((x1 + 1.) * WIDTH as f32 / 2 as f32) as u32;
            let y1 = ((y1 + 1.) * HEIGHT as f32 / 2 as f32) as u32;
            let x2 = ((x2 + 1.) * WIDTH as f32 / 2 as f32) as u32;
            let y2 = ((y2 + 1.) * HEIGHT as f32 / 2 as f32) as u32;
            let x3 = ((x3 + 1.) * WIDTH as f32 / 2 as f32) as u32;
            let y3 = ((y3 + 1.) * HEIGHT as f32 / 2 as f32) as u32;
            // println!("({},{}),({},{}),({},{})", x1, y1, x2, y2, x3, y3);
            draw_line(
                x1 as i32,
                y1 as i32,
                x2 as i32,
                y2 as i32,
                &mut buffer,
                Color::WHITE,
            );

            draw_line(
                x2 as i32,
                y2 as i32,
                x3 as i32,
                y3 as i32,
                &mut buffer,
                Color::WHITE,
            );

            draw_line(
                x3 as i32,
                y3 as i32,
                x1 as i32,
                y1 as i32,
                &mut buffer,
                Color::WHITE,
            );
        }
        // for vex in 0..mesh.positions.len() / 3 {

        // }
        // for face in 0..mesh.face_arities.len() {
        //     let end = next_face + mesh.face_arities[face] as usize;

        //     let face_indices = &mesh.indices[next_face..end];
        //     println!(" face[{}].indices          = {:?}", face, face_indices);
        //     for k in (0..face_indices.len() - 1) {
        //         let mut i = face_indices[k];
        //         let x1 = (mesh.positions[3 * i as usize] + 1.) * WIDTH as f32 / 2 as f32;
        //         let y1 = (mesh.positions[1 + 3 * i as usize] + 1.) * HEIGHT as f32 / 2 as f32;
        //         i = face_indices[k + 1];
        //         let x2 = (mesh.positions[3 * i as usize] + 1.) * WIDTH as f32 / 2 as f32;
        //         let y2 = (mesh.positions[1 + 3 * i as usize] + 1.) * HEIGHT as f32 / 2 as f32;
        //         println!("{},{},{},{}", x1, y1, x2, y2);
        //         draw_line(
        //             x1 as i32,
        //             y1 as i32,
        //             x2 as i32,
        //             y2 as i32,
        //             &mut buffer,
        //             Color::RED,
        //         );
        //     }
        //     // if !mesh.texcoord_indices.is_empty() {     let texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
        //     //     println!(
        //     //         " face[{}].texcoord_indices = {:?}",
        //     //         face, texcoord_face_indices
        //     //     );
        //     // }
        //     // if !mesh.normal_indices.is_empty() {
        //     //     let normal_face_indices = &mesh.normal_indices[next_face..end];
        //     //     println!(
        //     //         " face[{}].normal_indices   = {:?}",
        //     //         face, normal_face_indices
        //     //     );
        //     // }

        //     next_face = end;
        // }

        // Normals and texture coordinates are also loaded, but not printed in
        // this example.
        println!(
            "model[{}].positions        = {}",
            i,
            mesh.positions.len() / 3
        );
        assert!(mesh.positions.len() % 3 == 0);

        // for vtx in 0..mesh.positions.len() / 3 {
        //     println!(
        //         "              position[{}] = ({}, {}, {})",
        //         vtx,
        //         mesh.positions[3 * vtx],
        //         mesh.positions[3 * vtx + 1],
        //         mesh.positions[3 * vtx + 2]
        //     );
        // }
    }

    image::save_buffer("image.png", &buffer, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
}
fn main() {
    let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    draw_line(0, 200, 200, 0, &mut buffer, Color::BLUE);
    draw_line(0, 0, 10, 300, &mut buffer, Color::BLUE);
    draw_line(300, 200, 100, 300, &mut buffer, Color::BLUE);
    draw_line(300, 300, 100, 300, &mut buffer, Color::BLUE);
    draw_line(246, 383, 229, 388, &mut buffer, Color::BLUE);
    draw_line(399, 400, 200, 200, &mut buffer, Color::BLUE);

    image::save_buffer("image.png", &buffer, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
    load_obj("./assets/Lowpoly_tree_sample.obj");
}
