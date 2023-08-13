use bytemuck::{cast, cast_slice, cast_vec};

use crate::bind_group::{self, BindGroup, BindingType, Texture};
use crate::camera::{Camera, CameraProjection};
use crate::color::Color;
use crate::format::{TextureFormat, VertexFormat};
use crate::math::{Mat3, Mat4, Vec2, Vec3, Vec4};
use crate::mesh::Vertex;
use crate::shader::{FragmentInput, FragmentShader, ShaderType, VertexInput, VertexShader};
use std::ops::Range;

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
impl RenderSurface {
    #[rustfmt::skip]
    fn ndc2viewport_matrix(&self) -> Mat4 {
        Mat4::from_rows_slice(&[
            self.width as f32/2. , 0. , 0. , self.width as f32/2.,
            0. , self.height as f32/2. , 0. , self.height as f32/2.,
            0. , 0. , 1. , 0.,
            0. , 0. , 0. , 1.
        ])
    }
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
        let index_count = vertices.len();
        // 通过顶点布局计算顶点缓冲区中每个顶点的数据长度（字节）
        let vertex_len = self
            .state
            .vertex
            .layout
            .iter()
            .map(|format| format.size())
            .sum::<usize>();
        assert!(
            vertex_len == 0 || index_count >= self.vertex_buffer.len() / vertex_len,
            "out of bounds"
        );
        let vertex_location_count = self.state.vertex.layout.len();
        let mut vertex_shader_outputs = Vec::new();

        for i in vertices.clone() {
            // 该顶点在顶点缓冲区中的索引
            let mut vertex_buffer_offset = i as usize * vertex_len;
            let mut vertex_locations: Vec<ShaderType> = Vec::new();

            // 按照顶点的布局解析顶点用户自定义输入数据
            for format in self.state.vertex.layout {
                let format_value: ShaderType = match format {
                    VertexFormat::Float32 => ShaderType::F32(
                        cast_slice::<_, f32>(
                            &self.vertex_buffer
                                [vertex_buffer_offset..vertex_buffer_offset + format.size()],
                        )[0],
                    ),
                    VertexFormat::Float32x2 => ShaderType::Vec2(
                        std::convert::TryInto::<[f32; 2]>::try_into(cast_slice::<_, f32>(
                            &self.vertex_buffer
                                [vertex_buffer_offset..vertex_buffer_offset + format.size()],
                        ))
                        .unwrap()
                        .into(),
                    ),
                    VertexFormat::Float32x3 => ShaderType::Vec3(
                        std::convert::TryInto::<[f32; 3]>::try_into(cast_slice::<_, f32>(
                            &self.vertex_buffer
                                [vertex_buffer_offset..vertex_buffer_offset + format.size()],
                        ))
                        .unwrap()
                        .into(),
                    ),
                    VertexFormat::Float32x4 => ShaderType::Vec4(
                        std::convert::TryInto::<[f32; 4]>::try_into(cast_slice::<_, f32>(
                            &self.vertex_buffer
                                [vertex_buffer_offset..vertex_buffer_offset + format.size()],
                        ))
                        .unwrap()
                        .into(),
                    ),
                    _ => panic!(""),
                };
                vertex_locations.push(format_value);
                vertex_buffer_offset += format.size();
            }

            // 创建顶点着色器输入
            let vertex_shader_input = VertexInput {
                vertex_index: i as u32,
                // 暂时不支持实例化渲染，永远为0
                instance_index: 0,
                location: vertex_locations,
            };

            //执行顶点着色器
            let vertex_shader_ouput =
                (self.state.vertex.shader)(vertex_shader_input, &self.bind_groups);

            vertex_shader_outputs.push(vertex_shader_ouput)
        }

        for i in 0..vertices.len() / 3 {
            // 图元组装 我们这里只支持基础的三角形 "triangle-list"
            let primitive_list = &mut vertex_shader_outputs[i * 3..i * 3 + 2];
            // 图元裁剪
            // 顶点着色器会有输出 position(x,y,z,w)，我们在这里进行裁剪（其实就是齐次空间的视锥裁剪）
            // −p.w ≤ p.x ≤ p.w
            // −p.w ≤ p.y ≤ p.w
            // 0 ≤ p.z ≤ p.w (depth clipping)
            // tips ：按照标准 这里可以会产生新的顶点，但是 暂时未支持，图元有一个顶点在视锥范围外直接抛弃
            if primitive_clipping(primitive_list.iter().map(|x| x.position).collect()) {
                break;
            }

            // 光栅化阶段
            // 齐次坐标系->设备标准坐标系（NDC）[透视除法]
            let mut divisors = Vec::new();
            primitive_list.iter_mut().for_each(|v| {
                v.position.x /= v.position.w;
                v.position.y /= v.position.w;
                v.position.z /= v.position.w;
                divisors.push(1. / v.position.w);
            });

            // NDC -> 帧缓冲坐标(或者说视窗坐标)
            let frame_buffer_coordinates: Vec<Vec2> = primitive_list
                .iter()
                .map(|v| {
                    (
                        self.state.surface.width as f32 * 0.5 * (v.position.x + 1.),
                        self.state.surface.height as f32 * 0.5 * (v.position.y + 1.),
                    )
                        .into()
                })
                .collect();
            // 多边形光栅
            // 顺时针标准 ，area > 0 说明是正面
            let area: f32 = calculate_polygon_area(&frame_buffer_coordinates);

            // aabb 包围盒，左上 右下
            let aabb = calculate_polygon_aabb(&frame_buffer_coordinates);
            // 这里暂时不支持超采样,所以一个像素对应一个fragment
            for x in aabb[0]..=aabb[2] {
                for y in aabb[1]..=aabb[3] {
                    // 以坐标中心为像素坐标
                    let fragment_position = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    // for linear interpolation
                    let barycenter =
                        calculate_polygon_barycenter(fragment_position, &frame_buffer_coordinates);

                    // for perspective interpolation
                    let correct_barycenter = perspective_correct(&barycenter, &divisors);

                    let fragment_w_divisor_perspective_interpolated =
                        interpolate(&divisors, &correct_barycenter);
                    let fragment_depth_perspective_interpolated = interpolate(
                        &(primitive_list
                            .iter()
                            .map(|v| v.position.z)
                            .collect::<Vec<f32>>()),
                        &correct_barycenter,
                    );

                    let mut vertex_location_bundle: Vec<Vec<ShaderType>> = (0
                        ..vertex_location_count)
                        .map(|_index| Vec::new())
                        .collect();
                    for index in 0..vertex_location_count {}
                    // 对顶点着色器的用户自定义输入进行插值 这里默认使用透视插值，暂时不支持其他插值
                    let fragment_input_locations: Vec<ShaderType> = {
                        primitive_list
                            .iter()
                            .fold(&mut vertex_location_bundle, |mut acc, v| {
                                for index in 0..vertex_location_count {
                                    acc[index].push(v.location[index]);
                                }
                                acc
                            });

                        for index in 0..vertex_location_bundle.len() {
                            // let a = self.state.vertex.layout[index].size();
                            let a: Vec<u8> = cast_vec(vertex_location_bundle[index]);
                        }
                        todo!()
                    };
                    let fragment_input = FragmentInput {
                        front_facing: area > 0.,
                        position: Vec4::new(
                            fragment_position.x,
                            fragment_position.y,
                            fragment_depth_perspective_interpolated,
                            fragment_w_divisor_perspective_interpolated,
                        ),
                        sample_index: 0, //暂时没有超采样
                        sample_mask: 0,
                        location: Vec::new(),
                    };
                    // 计算深度
                }
            }
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

// pub fn perspective_correct((alpha, beta, gamma): (f32, f32, f32), w: &Vec<f32>) -> Vec3 {
//     let w0 = w[0].recip() * alpha;
//     let w1 = w[1].recip() * beta;
//     let w2 = w[2].recip() * gamma;
//     let normalizer = 1.0 / (w0 + w1 + w2);
//     [w0 * normalizer, w1 * normalizer, w2 * normalizer].into()
// }
// 最简单的方法，图元有一个点不在视锥内就直接抛弃了，这样不会产生新的节点了
fn primitive_clipping(vertex_positions: Vec<Vec4>) -> bool {
    vertex_positions
        .iter()
        .any(|v| v.x > v.w || v.x < -v.w || v.y > v.w || v.y < -v.w || v.z > v.w || v.z < 0.)
}

// 多边形面积计算
fn calculate_polygon_area(coordinates: &[Vec2]) -> f32 {
    let mut area = 0.;
    for i in 0..coordinates.len() {
        area += coordinates[i].cross(coordinates[(i + 1) % coordinates.len()]);
    }
    0.5 * area
}
// 计算多边形aabb包围盒
// 返回包围盒 左上 和 右下 坐标  [x1,y1,x2,y2]
//屏幕左上角（0，0），右下角（width，height）
fn calculate_polygon_aabb(coordinates: &[Vec2]) -> [u32; 4] {
    let mut aabb = [u32::MAX, u32::MAX, 0, 0];
    for coordinate in coordinates {
        aabb[0] = aabb[0].min(coordinate.x.floor() as u32);
        aabb[1] = aabb[1].min(coordinate.y.floor() as u32);
        aabb[2] = aabb[2].max(coordinate.x.ceil() as u32);
        aabb[3] = aabb[3].max(coordinate.y.ceil() as u32);
    }
    aabb
}

fn calculate_triangle_barycenter(triangel: &[Vec2]) -> Vec2 {
    todo!()
}
// 计算点p对于平面多边形的重心坐标
// https://gpuweb.github.io/gpuweb/#barycentric-coordinates
fn calculate_polygon_barycenter(p: Vec2, polygon: &[Vec2]) -> Vec<f32> {
    let polygon_len = polygon.len();
    let mut res = Vec::new();
    for i in 0..polygon_len {
        let lamda = (p - polygon[i]).cross(polygon[(i + 1) % polygon_len] - polygon[i]);
        res.push(lamda);
    }
    let sum: f32 = res.iter().sum();
    res.iter().map(|v| v / sum).collect()
}

// 计算重心参数的矫正
pub fn perspective_correct(barycenter: &[f32], divisors: &[f32]) -> Vec<f32> {
    let iterator = (0..barycenter.len()).map(|i| barycenter[i] * divisors[i]);
    let sum: f32 = iterator.clone().sum();
    iterator.map(|x| x / sum).collect()
}
// 计算插值
pub fn interpolate(val: &[f32], weights: &[f32]) -> f32 {
    assert!(val.len() != weights.len());
    (0..val.len()).fold(0., |acc, index| acc + val[index] * weights[index])
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_calculate_aabb() {}
    #[test]
    fn test_calculate_area() {}
    #[test]
    fn test_calculate_barycenter() {}
}
