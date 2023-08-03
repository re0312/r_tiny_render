use std::error::Error;

use crate::camera::{Camera, CameraProjection};
use crate::color::Color;

use crate::math::{Mat4, Vec2, Vec3, Vec4};
use crate::mesh::{Mesh, Vertex};

#[derive(Default)]
pub struct Renderer {
    pub camera: Camera,
    pub frame_buffer: Vec<u8>,
    pub depth_buffer: Vec<f32>,
}
impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_camera(mut self, camera: Camera) -> Self {
        self.frame_buffer = vec![0; camera.viewport.size() as usize * 3];
        self.depth_buffer = vec![0.; camera.viewport.size() as usize * 3];
        self.camera = camera;
        self
    }

    pub fn draw_pixel(&mut self, x: usize, mut y: usize, color: Color) {
        let width = self.camera.viewport.physical_size.x as usize;
        y = self.camera.viewport.physical_size.y as usize - y - 1;
        #[cfg(feature = "info")]
        println!("piexl:[({},{})]\n width:{}", x, y, width);

        self.frame_buffer[(width * y + x) * 3] = (color.r * 255.) as u8;
        self.frame_buffer[((width * y + x) * 3) + 1] = (color.g * 255.) as u8;
        self.frame_buffer[((width * y + x) * 3) + 2] = (color.b * 255.) as u8;
    }

    pub fn draw_line(&mut self, line: (Vec2, Vec2)) -> Option<()> {
        let points = clip_line(
            line,
            (0., self.camera.viewport.physical_size.y).into(),
            (self.camera.viewport.physical_size.x, 0.).into(),
        )?;
        let ((x1, y1), (x2, y2)) = (points.0.into(), points.1.into());
        let (mut x1, mut y1, mut x2, mut y2) = (x1 as i32, y1 as i32, x2 as i32, y2 as i32);
        let mut xy_reverse = false;

        // 斜率大于1 xy 倒转，回到写了小于1的情况
        if (x1 - x2).abs() < (y1 - y2).abs() {
            (x1, x2, y1, y2) = (y1, y2, x1, x2);
            xy_reverse = true;
        }

        // 保证 点1 一定要比 点2 小，这里小的含义是 保证点从低 到 高 绘制
        if x1 >= x2 && y1 >= y2 {
            (x1, x2, y1, y2) = (x2, x1, y2, y1);
        }

        // 是从右往左 还是从左往右遍历，因为要保证从 低点 到 高点绘制
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
                self.draw_pixel(y0 as usize, x0 as usize, Color::WHITE);
            } else {
                self.draw_pixel(x0 as usize, y0 as usize, Color::WHITE);
            }
            delta += ky;
            if delta > middle {
                y0 += 1;
                middle += kx;
            }
        }
        None
    }

    pub fn draw_mesh(&mut self, mesh: &mut Mesh) {
        let model_matrix = mesh.transform.compute_matrix();
        for i in 0..mesh.vertices.len() / 3 {
            let triangle = &mut mesh.vertices[i * 3..(i * 3) + 3];

            // mvp
            self.draw_triangle(triangle, model_matrix);
        }
    }

    pub fn draw_triangle(&mut self, triangle: &mut [Vertex], model_matrix: Mat4) {
        // 视图变换
        let view_matrix = self.camera.get_view_matrix();
        // 投影变换矩阵
        let projection_matrix = self.camera.projectiton.get_projection_matrix();
        // 窗口变换矩阵
        let viewport_matrix = self.camera.viewport.get_viewport_matrix();

        // mvp变换
        apply_matrix(triangle, model_matrix);
        apply_matrix(triangle, view_matrix);
        apply_matrix(triangle, projection_matrix);

        // 齐次除法 把经过变换后的向量变为坐标
        homogeneous_division(triangle);

        // 坐标映射到渲染窗口
        apply_matrix(triangle, viewport_matrix);

        // 光栅化处理
        self.rasterization(triangle);
    }

    pub fn draw_wireframe(&mut self, triangle: &[Vertex]) {
        for i in 0..triangle.len() {
            self.draw_line((
                triangle[i].position.xy(),
                triangle[(i + 1) % 3].position.xy(),
            ));
        }
    }

    // 光栅化
    pub fn rasterization(&mut self, triangle: &[Vertex]) {
        let x_min = triangle
            .iter()
            .map(|x| x.position.x as usize)
            .min()
            .unwrap();
        let y_min = triangle
            .iter()
            .map(|x| x.position.y as usize)
            .min()
            .unwrap();

        let x_max = triangle
            .iter()
            .map(|x| x.position.x as usize)
            .max()
            .unwrap();
        let y_max = triangle
            .iter()
            .map(|x| x.position.y as usize)
            .max()
            .unwrap();
        for x in x_min..x_max {
            for y in y_min..y_max {
                if inside_triangle(x as f32, y as f32, triangle) {
                    self.draw_pixel(x, y, Color::WHITE);
                }
            }
        }
        #[cfg(target_featur = "info")]
        println!("rasterization:{},{},{},{}", x_min, y_min, x_max, y_max);
    }
}

const INSIDE: u8 = 0; // 0000
const LEFT: u8 = 1; // 0001
const RIGHT: u8 = 2; // 0010
const BOTTOM: u8 = 4; // 0100
const TOP: u8 = 8; // 1000
pub fn endpoint_code(p: Vec2, rec_left_top: Vec2, rec_right_bot: Vec2) -> u8 {
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

// 线段裁剪，裁剪不在窗口范围里面的线段
pub fn clip_line(
    line: (Vec2, Vec2),
    rec_left_top: Vec2,
    rec_right_bot: Vec2,
) -> Option<(Vec2, Vec2)> {
    let p0_code = endpoint_code(line.0, rec_left_top, rec_right_bot);
    let p1_code = endpoint_code(line.1, rec_left_top, rec_right_bot);

    if p0_code & p1_code != 0 {
        return None;
    } else if p0_code | p1_code == 0 {
        return Some(line);
    }

    return None;
}

fn inside_triangle(x: f32, y: f32, triangle: &[Vertex]) -> bool {
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

pub fn apply_matrix(triangle: &mut [Vertex], matrix: Mat4) {
    for i in 0..triangle.len() {
        triangle[i].position = matrix * triangle[i].position;
    }
}

pub fn homogeneous_division(triangle: &mut [Vertex]) {
    triangle.iter_mut().for_each(|x| {
        x.position.x /= x.position.w;
        x.position.y /= x.position.w;
        x.position.z /= x.position.w;
        x.position.w = 1.;
    })
}
pub fn back_face_culling(triangle: &[Vertex; 3], view_dir: Vec3) {}
