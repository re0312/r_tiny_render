use std::default;
use std::ops::Range;

use gltf::image::Format;

use crate::bind_group::{self, BindGroup, BindingType, Texture};
use crate::camera::{Camera, CameraProjection};
use crate::color::Color;
use crate::format::{FormatSize, TextureFormat, VertexFormat};
use crate::math::{Mat4, Vec2, Vec3, Vec4};
use crate::mesh::Vertex;
use crate::shader::{FragmentShader, Location, VertexInput, VertexShader};

pub trait Buffer {}

pub struct VertexBufferLayout {}
pub struct Renderer<'a> {
    pub state: RendererDescriptor<'a>,
    pub frame_buffer: Vec<u8>,
    pub depth_buffer: Vec<f32>,
    // 保留齐次坐标下的w值，其实就是视图空间中的 -z 值
    pub w_buffer: Vec<f32>,
    // 绑定组，感觉好像在软渲染中不太需要
    pub bind_groups: Vec<BindGroup>,
    // 顶点缓冲区
    pub vertex_buffer: &'a [u8],
}

pub struct VertexState<'a> {
    pub(crate) shader: VertexShader,
    // 这里直接简化掉顶点布局，layout数组表示顶点数据自定义的location的数量，每个location的长度由VertexFormta决定
    pub(crate) layout: &'a [VertexFormat],
}

pub struct FragmentState {
    pub(crate) shader: FragmentShader,
}

pub struct RenderSurface {
    pub width: usize,
    pub height: usize,
    pub format: TextureFormat,
}
// 当前暂时先就不区分pipeline 和 renderpass
pub struct RendererDescriptor<'a> {
    pub surface: RenderSurface,
    pub vertex: VertexState<'a>,
    pub fragment: FragmentState,
    pub bind_group_count: usize,
}
impl<'a> Renderer<'a> {
    pub fn new(desc: RendererDescriptor<'a>) -> Self {
        let pixel_count = desc.surface.height * desc.surface.width;
        Renderer {
            frame_buffer: vec![0; pixel_count * desc.surface.format.size()],
            depth_buffer: vec![0.; pixel_count],
            w_buffer: vec![0.; pixel_count],
            bind_groups: vec![vec![]; desc.bind_group_count],
            vertex_buffer: &[],
            state: desc,
        }
    }

    // pub fn with_camera(mut self, camera: Camera) -> Self {
    //     self.frame_buffer = vec![0; camera.viewport.size() as usize * 3];
    //     self.depth_buffer = vec![0.; camera.viewport.size() as usize];
    //     self.w_buffer = vec![1. / 3.; 3];
    //     self.camera = camera;
    //     self
    // }

    pub fn set_vertex_buffer(&mut self, vertex_buffer: &'a [u8]) {
        self.vertex_buffer = vertex_buffer;
    }

    pub fn set_bind_group(&mut self, index: usize, group: BindGroup) {
        self.bind_groups.insert(index, group);
    }

    // 按照WebGpu标准，渲染算法包括下面步骤
    // 索引解析 -- 顶点解析 -- 顶点处理 -- 图元组装 -- 图元裁剪 -- 光栅化 -- 片元解析 -- 深度解析 --绘制像素

    // https://gpuweb.github.io/gpuweb/#rendering-operations
    pub fn draw(&mut self, vertices: Range<u32>) {
        // 通过顶点布局计算顶点缓冲区中每个顶点的数据长度（字节）
        let vertex_len = self
            .state
            .vertex
            .layout
            .iter()
            .map(|format| format.size())
            .sum::<usize>();
        assert!(
            vertex_len == 0 || vertices.len() > self.vertex_buffer.len() / vertex_len,
            "out of bounds"
        );
        for i in vertices {
            // 该顶点在顶点缓冲区中的索引
            let vertex_buffer_offset = i * vertex_len as u32;
            let mut vertex_locations: Vec<Box<dyn Location>> = Vec::new();
            for format in self.state.vertex.layout {
                let format_value: Box<dyn Location> = match format {
                    VertexFormat::Float32 => Box::new(0.),
                    VertexFormat::Float32x2 => Box::new(Vec2::ZERO),
                    VertexFormat::Float32x3 => Box::new(Vec3::ZERO),
                    VertexFormat::Float32x4 => Box::new(Vec4::ZERO),
                    _ => Box::new(0.),
                };
                vertex_locations.push(format_value);
            }
            let vertex_shader_input = VertexInput {
                vertex_index: i,
                // 暂时不支持实例化渲染，永远为0
                instance_index: 0,
                location: Vec::new(),
            };
        }
    }

    pub fn draw_pixel(&mut self, x: usize, mut y: usize, color: Color) {
        let width = self.state.surface.width;
        y = self.state.surface.height - y - 1;
        #[cfg(feature = "info")]
        println!("piexl:[({},{})]\n width:{}", x, y, width);

        self.frame_buffer[(width * y + x) * 3] = (color.r * 255.) as u8;
        self.frame_buffer[((width * y + x) * 3) + 1] = (color.g * 255.) as u8;
        self.frame_buffer[((width * y + x) * 3) + 2] = (color.b * 255.) as u8;
    }

    pub fn draw_line(&mut self, line: (Vec2, Vec2)) -> Option<()> {
        let points = clip_line(
            line,
            (0., self.state.surface.height as f32).into(),
            (self.state.surface.width as f32, 0.).into(),
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
                self.draw_pixel(y0 as usize, x0 as usize, Color::RED);
            } else {
                self.draw_pixel(x0 as usize, y0 as usize, Color::RED);
            }
            delta += ky;
            if delta > middle {
                y0 += 1;
                middle += kx;
            }
        }
        None
    }

    // pub fn draw_triangle(&mut self, triangle: &mut [Vertex], model_matrix: Mat4) {
    //     // 视图变换
    //     let view_matrix = self.camera.get_view_matrix();
    //     // 投影变换矩阵
    //     let projection_matrix = self.camera.projectiton.get_projection_matrix();
    //     // 窗口变换矩阵
    //     let viewport_matrix = self.camera.viewport.get_viewport_matrix();

    //     // 模型坐标系 -> 实际坐标系
    //     apply_matrix(triangle, model_matrix);

    //     // 世界坐标系 -> 视图坐标系
    //     apply_matrix(triangle, view_matrix);

    //     // 背面剔除顺时针的图元
    //     if !back_face_culling(triangle, Vec3::NEG_Z) {
    //         return;
    //     }

    //     // 投影矩阵 从视图坐标系 -> 齐次坐标系
    //     apply_matrix(triangle, projection_matrix);

    //     // let frustum = Frustum::from_view_projection(&projection_z_reverse_matrix);

    //     // 齐次裁剪，这里没有视窗裁剪,这里不是太严谨
    //     if !self.homogeneous_clip(triangle) {
    //         return;
    //     }

    //     // 保留齐次坐标系下面的w值，后面需要透视矫正
    //     self.w_buffer = triangle.iter().map(|v| v.position.w).collect::<Vec<f32>>();

    //     // 齐次除法 齐次坐标系-> 设备标准坐标系
    //     homogeneous_division(triangle);

    //     // 设备标准坐标系 -> 视窗（屏幕）坐标系
    //     apply_matrix(triangle, viewport_matrix);

    //     // 光栅化处理
    //     self.rasterization(triangle);
    // }

    // pub fn draw_wireframe(&mut self, triangle: &[Vertex]) {
    //     for i in 0..triangle.len() {
    //         self.draw_line((
    //             triangle[i].position.xy(),
    //             triangle[(i + 1) % 3].position.xy(),
    //         ));
    //     }
    // }

    // 光栅化
    // pub fn rasterization(&mut self, triangle: &[Vertex]) {
    //     let x_min = triangle
    //         .iter()
    //         .map(|x| x.position.x.ceil() as usize)
    //         .min()
    //         .unwrap()
    //         .max(0);
    //     let y_min = triangle
    //         .iter()
    //         .map(|x| x.position.y.ceil() as usize)
    //         .min()
    //         .unwrap()
    //         .max(0);

    //     let x_max = triangle
    //         .iter()
    //         .map(|x| x.position.x.ceil() as usize)
    //         .max()
    //         .unwrap()
    //         .min(self.state.surface.width as usize);
    //     let y_max = triangle
    //         .iter()
    //         .map(|x| x.position.y.ceil() as usize)
    //         .max()
    //         .unwrap()
    //         .min(self.state.surface.height);
    //     for x in x_min..x_max {
    //         for y in y_min..y_max {
    //             // 重心坐标
    //             let barycenter = barycentric_2d(
    //                 (x as f32, y as f32).into(),
    //                 triangle[0].position.xy(),
    //                 triangle[1].position.xy(),
    //                 triangle[2].position.xy(),
    //             );

    //             if inside_triangle_barcentric(barycenter) {
    //                 //经过透视矫正后的重心坐标
    //                 let p_barycenter = perspective_correct(barycenter.into(), &self.w_buffer);
    //                 // zbuffer
    //                 let z_interpolation = triangle[0].position.z * p_barycenter.x
    //                     + triangle[1].position.z * p_barycenter.y
    //                     + triangle[2].position.z * p_barycenter.z;

    //                 // reversed z z值越大越近，z为 0 是远平面
    //                 if z_interpolation > self.depth_buffer[y * self.state.surface.height + x] {
    //                     // 颜色插值
    //                     let fragment_color = triangle[0].color.unwrap_or(Color::WHITE)
    //                         * p_barycenter.x
    //                         + triangle[1].color.unwrap_or(Color::WHITE) * p_barycenter.y
    //                         + triangle[2].color.unwrap_or(Color::WHITE) * p_barycenter.z;
    //                     self.depth_buffer[y * self.surface.height + x] = z_interpolation;

    //                     let pixel_color =
    //                         self.fragment_shader(triangle, fragment_color, p_barycenter);
    //                     self.draw_pixel(x, y, pixel_color);
    //                 }
    //             }
    //         }
    //     }
    //     // self.draw_wireframe(triangle);
    //     #[cfg(target_featur = "info")]
    //     println!("rasterization:{},{},{},{}", x_min, y_min, x_max, y_max);
    // }

    // 齐次坐标下视锥裁剪
    // pub fn homogeneous_clip(&self, triangle: &[Vertex]) -> bool {
    //     // 三角形 任意一个点在近平面后面或者超过远平面范围 直接抛弃整个三角形,因为如果在平面上可能会导致除0错误
    //     if triangle.iter().any(|v| {
    //         v.position.w > self.camera.projectiton.far
    //             || v.position.w < self.camera.projectiton.near
    //     }) {
    //         return false;
    //     }
    //     // 三角形每个点都不在视锥范围里面就裁剪掉，由于是已经是裁剪空间，所以直接比较x,y值和w的大小就行
    //     // 要求 -w<x<w -w<y<w
    //     else if triangle
    //         .iter()
    //         .all(|v| v.position.x.abs() > v.position.w || v.position.y.abs() > v.position.w)
    //     {
    //         return false;
    //     }
    //     return true;
    // }

    // pub fn fragment_shader(&self, triangle: &[Vertex], color: Color, barycenter: Vec3) -> Color {
    //     let texcoord = if triangle[0].texcoord.is_some()
    //         && triangle[1].texcoord.is_some()
    //         && triangle[2].texcoord.is_some()
    //     {
    //         Some(
    //             triangle[0].texcoord.unwrap() * barycenter.x
    //                 + triangle[1].texcoord.unwrap() * barycenter.y
    //                 + triangle[2].texcoord.unwrap() * barycenter.z,
    //         )
    //     } else {
    //         None
    //     };
    //     let texcolor = if texcoord.is_some() {
    //         self.bindings
    //             .texture_id_map
    //             .get(&3)
    //             .map(|texture| texture.sample(texcoord.unwrap()))
    //     } else {
    //         None
    //     };
    //     return if texcolor.is_some() {
    //         texcolor.unwrap()
    //     } else {
    //         color
    //     };
    // }
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

// Cohen-Sutherland线段裁剪算法，裁剪不在窗口范围里面的线段
pub fn clip_line(
    line: (Vec2, Vec2),
    rec_left_top: Vec2,
    rec_right_bot: Vec2,
) -> Option<(Vec2, Vec2)> {
    let (mut p0, mut p1) = line;
    let mut p0_code = endpoint_code(p0, rec_left_top, rec_right_bot);
    let mut p1_code = endpoint_code(p1, rec_left_top, rec_right_bot);

    loop {
        if p0_code & p1_code != 0 {
            return None;
        } else if p0_code | p1_code == 0 {
            return Some((p0, p1));
        }
        let out_code = if p0_code > p1_code { p0_code } else { p1_code };
        let mut p = Vec2::ZERO;
        if out_code & TOP != 0 {
            p.x = (p1.x - p0.x) / (p1.y - p0.y) * (rec_left_top.y - p0.y) + p0.x;
            p.y = rec_left_top.y;
        } else if out_code & BOTTOM != 0 {
            p.x = p0.x + (p0.x - p0.x) * (rec_right_bot.y - p0.y) / (p1.y - p0.y);
            p.y = rec_right_bot.y;
        } else if out_code & RIGHT != 0 {
            p.x = rec_right_bot.x;
            p.y = p0.y + (p1.y - p0.y) * (rec_right_bot.x - p0.x) / (p1.x - p0.x);
        } else if out_code & LEFT != 0 {
            p.x = rec_left_top.x;
            p.y = p0.y + (p1.y - p0.y) * (rec_left_top.x - p0.x) / (p1.x - p0.x);
        }
        if out_code == p0_code {
            p0 = p;
            p0_code = endpoint_code(p0, rec_left_top, rec_right_bot);
        } else {
            p1 = p;
            p1_code = endpoint_code(p1, rec_left_top, rec_right_bot);
        }
    }
}

//  通过向量叉乘判断点是否在三角形里
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

// 通过重心坐标判断点是否在三角形中
fn inside_triangle_barcentric(barcentric: Vec3) -> bool {
    barcentric.x > 0. && barcentric.y > 0. && barcentric.z > 0.
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
// 背面剔除，剔除所有顺时针的三角形
pub fn back_face_culling(triangle: &[Vertex], view_dir: Vec3) -> bool {
    let face_normal = (triangle[1].position.xyz() - triangle[0].position.xyz())
        .cross(triangle[2].position.xyz() - triangle[1].position.xyz());
    return face_normal.dot(view_dir) < 0.0;
}

pub fn barycentric_2d(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> Vec3 {
    let double_triangle_area = (b - a).cross(c - a);
    let alpha = (b - p).cross(c - p) / double_triangle_area;
    let beta = (c - p).cross(a - p) / double_triangle_area;
    let gamma = (a - p).cross(b - p) / double_triangle_area;
    [alpha, beta, gamma].into()
}

pub fn perspective_correct((alpha, beta, gamma): (f32, f32, f32), w: &Vec<f32>) -> Vec3 {
    let w0 = w[0].recip() * alpha;
    let w1 = w[1].recip() * beta;
    let w2 = w[2].recip() * gamma;
    let normalizer = 1.0 / (w0 + w1 + w2);
    [w0 * normalizer, w1 * normalizer, w2 * normalizer].into()
}
