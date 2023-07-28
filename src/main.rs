use r_tiny_render::{
    color::Color,
    constant::{HEIGHT, WIDTH},
    loader::{load_gltf, load_obj},
    math::Vec4,
    render::{draw_line, draw_triangle, Vertex},
};

fn test_line() {
    let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    draw_line(50, 160, 70, 80, &mut buffer, Color::BLUE);
    draw_line(0, 0, 10, 300, &mut buffer, Color::BLUE);
    draw_line(300, 200, 100, 300, &mut buffer, Color::BLUE);
    draw_line(300, 300, 100, 300, &mut buffer, Color::BLUE);
    draw_line(246, 383, 229, 388, &mut buffer, Color::BLUE);
    draw_line(399, 400, 200, 200, &mut buffer, Color::BLUE);
    image::save_buffer("image.png", &buffer, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
}

fn test_mesh() {
    let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    let meshs = load_gltf("./assets/sphere/sphere.gltf");
    println!("meshs length:{}", meshs.len());
    let scale = 2.;
    for i in 0..meshs.len() / 3 {
        let x1 = meshs[i * 3].x;
        let y1 = meshs[i * 3].y;

        let x2 = meshs[i * 3 + 1].x;
        let y2 = meshs[i * 3 + 1].y;

        let x3 = meshs[i * 3 + 2].x;
        let y3 = meshs[i * 3 + 2].y;

        let x1 = ((x1 + 1.) / scale * WIDTH as f32 / 2 as f32) as u32;
        let y1 = ((y1 + 1.) / scale * HEIGHT as f32 / 2 as f32) as u32;
        let x2 = ((x2 + 1.) / scale * WIDTH as f32 / 2 as f32) as u32;
        let y2 = ((y2 + 1.) / scale * HEIGHT as f32 / 2 as f32) as u32;
        let x3 = ((x3 + 1.) / scale * WIDTH as f32 / 2 as f32) as u32;
        let y3 = ((y3 + 1.) / scale * HEIGHT as f32 / 2 as f32) as u32;
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
            Color::RED,
        );

        draw_line(
            x3 as i32,
            y3 as i32,
            x1 as i32,
            y1 as i32,
            &mut buffer,
            Color::BLUE,
        );
    }
    image::save_buffer("image.png", &buffer, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
}

fn test_rasterization() {
    let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
    let t1 = [
        Vertex {
            position: Vec4::new(10., 70., 0., 1.),
        },
        Vertex {
            position: Vec4::new(50., 160., 0., 1.),
        },
        Vertex {
            position: Vec4::new(70., 80., 0., 1.),
        },
    ];
    let t2 = [
        Vertex {
            position: Vec4::new(180., 50., 0., 1.),
        },
        Vertex {
            position: Vec4::new(150., 1., 0., 1.),
        },
        Vertex {
            position: Vec4::new(70., 180., 0., 1.),
        },
    ];
    let t3 = [
        Vertex {
            position: Vec4::new(180., 150., 0., 1.),
        },
        Vertex {
            position: Vec4::new(120., 160., 0., 1.),
        },
        Vertex {
            position: Vec4::new(130., 180., 0., 1.),
        },
    ];
    draw_triangle(&t1, &mut buffer);
    draw_triangle(&t2, &mut buffer);
    draw_triangle(&t3, &mut buffer);
    image::save_buffer("image.png", &buffer, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
}


fn main() {
    test_rasterization();
}
