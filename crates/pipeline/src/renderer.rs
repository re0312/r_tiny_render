use crate::bind_group::BindGroup;
use crate::format::{TextureFormat, VertexFormat};
use crate::shader::{FragmentInput, FragmentShader, ShaderType, VertexInput, VertexShader};
use crate::VertexOutput;
use bytemuck::cast_slice;
use math::{Vec2, Vec4};
use std::ops::Range;

pub struct Renderer<'a> {
    pub state: RendererDescriptor<'a>,
    pub frame_buffer: Vec<u8>,
    pub depth_buffer: Vec<f32>,
    // 绑定组，感觉好像在软渲染中不太需要
    pub bind_groups: Vec<BindGroup>,
    // 顶点缓冲区
    pub vertex_buffer: &'a [u8],
    // 顶点索引
    pub index_buffer: &'a [u32],
}

pub struct VertexState<'a> {
    pub shader: VertexShader,
    // 这里直接简化掉顶点布局，layout数组表示顶点数据自定义的location的数量，每个location的长度由VertexFormta决定
    pub layout: &'a [VertexFormat],
}

pub struct FragmentState {
    pub shader: FragmentShader,
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
}
impl<'a> Renderer<'a> {
    pub fn new(desc: RendererDescriptor<'a>) -> Self {
        let pixel_count = desc.surface.height * desc.surface.width;
        Renderer {
            frame_buffer: vec![0; pixel_count * desc.surface.format.size()],
            depth_buffer: vec![0.; pixel_count],
            bind_groups: vec![vec![]; 10],
            vertex_buffer: &[],
            index_buffer: &[],
            state: desc,
        }
    }

    pub fn set_vertex_buffer(&mut self, vertex_buffer: &'a [u8]) {
        self.vertex_buffer = vertex_buffer;
    }

    pub fn set_bind_group(&mut self, index: usize, group: BindGroup) {
        self.bind_groups.insert(index, group);
    }

    pub fn set_index_buffer(&mut self, index_buffer: &'a [u32]) {
        self.index_buffer = index_buffer;
    }

    // 按照WebGpu标准，渲染算法包括下面步骤
    // 索引解析 -- 顶点解析 -- 顶点处理 -- 图元组装 -- 图元裁剪 -- 光栅化 -- 片元解析 -- 深度解析 --绘制像素
    // https://gpuweb.github.io/gpuweb/#rendering-operations
    pub fn draw(&mut self, vertices: Range<u32>) {
        let vertices: Vec<u32> = vertices.collect();
        // 顶点处理，这里直接对所有顶点进行计算
        let vertex_shader_outputs = self.vertex_processing(&vertices);
        // 图元组装和裁剪
        let primitive_index_list =
            self.primitive_assembly_clipping(&vertices, &vertex_shader_outputs);
        // 光栅化
        self.rasterization(vertex_shader_outputs, primitive_index_list);
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>) {
        let vertices = self.index_resolution(indices);
        let vertex_shader_outputs = self.vertex_processing(&vertices);
        let primitive_index_list =
            self.primitive_assembly_clipping(&vertices, &vertex_shader_outputs);
        self.rasterization(vertex_shader_outputs, primitive_index_list);
    }

    // 索引解析，返回代处理的顶点
    pub fn index_resolution(&self, indices: Range<u32>) -> Vec<u32> {
        indices.fold(Vec::new(), |mut acc, v| {
            acc.push(self.index_buffer[v as usize]);
            acc
        })
    }

    //顶点处理
    pub fn vertex_processing(&mut self, _vertices: &[u32]) -> Vec<VertexOutput> {
        let vertex_len = self
            .state
            .vertex
            .layout
            .iter()
            .map(|format| format.size())
            .sum::<usize>();

        let vertex_count = self.vertex_buffer.len() / vertex_len;
        // 顶点着色器输出
        let mut vertex_shader_outputs = Vec::new();
        for i in 0..vertex_count {
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
                (self.state.vertex.shader)(vertex_shader_input, &mut self.bind_groups);

            vertex_shader_outputs.push(vertex_shader_ouput)
        }
        vertex_shader_outputs
    }

    // 图元组装和裁剪
    pub fn primitive_assembly_clipping(
        &self,
        vertices: &[u32],
        vertex_shader_outputs: &[VertexOutput],
    ) -> Vec<Vec<u32>> {
        let mut primitive_list = Vec::new();
        for i in 0..vertices.len() / 3 {
            // 图元组装 我们这里只支持基础的三角形 "triangle-list"
            let primitive = [
                &vertex_shader_outputs[vertices[i * 3] as usize],
                &vertex_shader_outputs[vertices[i * 3 + 1] as usize],
                &vertex_shader_outputs[vertices[i * 3 + 2] as usize],
            ];
            // 图元裁剪
            // 顶点着色器会有输出 position(x,y,z,w)，我们在这里进行裁剪（其实就是齐次空间的视锥裁剪）
            // −p.w ≤ p.x ≤ p.w
            // −p.w ≤ p.y ≤ p.w
            // 0 ≤ p.z ≤ p.w (depth clipping)
            // tips ：按照标准 这里可以会产生新的顶点，但是 暂时未支持，图元有一个顶点在视锥范围外直接抛弃
            if primitive_clipping(primitive.iter().map(|x| x.position).collect()) {
                continue;
            }
            primitive_list.push(vertices[i * 3..i * 3 + 3].to_vec());
        }
        primitive_list
    }

    // 光栅化
    pub fn rasterization(
        &mut self,
        vertex_shader_outputs: Vec<VertexOutput>,
        primitive_index_list: Vec<Vec<u32>>,
    ) {
        // 首先要得到vertex shader的输出布局，要映射到fragment shader的输入
        let vertex_shader_ouput_layouts: Vec<VertexFormat> = vertex_shader_outputs[0]
            .location
            .iter()
            .map(|&ty| match ty {
                ShaderType::F32(_v) => VertexFormat::Float32,
                ShaderType::Vec2(_v) => VertexFormat::Float32x2,
                ShaderType::Vec3(_v) => VertexFormat::Float32x3,
                ShaderType::Vec4(_v) => VertexFormat::Float32x4,
            })
            .collect();
        // 得到当前图元的 index 索引
        for primitive_index in primitive_index_list {
            // 拿到光栅图元
            let mut primitive: Vec<VertexOutput> = primitive_index
                .iter()
                .map(|&v| vertex_shader_outputs[v as usize].clone())
                .collect();

            // 齐次坐标系->设备标准坐标系（NDC）[透视除法]
            let mut divisors = Vec::new();
            primitive.iter_mut().for_each(|v| {
                v.position.x /= v.position.w;
                v.position.y /= v.position.w;
                v.position.z /= v.position.w;
                divisors.push(1. / v.position.w);
            });

            // NDC -> 帧缓冲坐标(或者说视窗坐标)
            let frame_buffer_coordinates: Vec<Vec2> = primitive
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
            // cw 顺时针标准 ，area > 0 说明是正面，可以用来作为背面剔除的判断条件
            let area: f32 = calculate_polygon_area(&frame_buffer_coordinates);

            // 背面剔除
            // if area < 0. {
            //     continue;
            // }

            // aabb 包围盒，左上 右下
            let aabb = calculate_polygon_aabb(&frame_buffer_coordinates);

            // 这里暂时不支持超采样,所以一个像素对应一个fragment
            for x in aabb[0]..aabb[2].clamp(0, self.state.surface.width) {
                for y in aabb[1]..aabb[3].clamp(0, self.state.surface.height) {
                    // 以坐标中心为像素坐标
                    let fragment_position = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);

                    // 线性插值使用的重心参数
                    let barycenter = calculate_polygon_barycenter(
                        fragment_position,
                        &frame_buffer_coordinates,
                        area,
                    );
                    // 验证当前像素点是否在多边形里面
                    if barycenter.iter().any(|&v| v < 0.) {
                        continue;
                    }

                    // 透视插值使用的重心参数
                    let correct_barycenter = perspective_correct(&barycenter, &divisors);

                    // 计算透视插值下的w因子和深度
                    let fragment_w_divisor_perspective_interpolated =
                        interpolate(&divisors, &correct_barycenter);
                    let fragment_depth_perspective_interpolated = interpolate(
                        &(primitive.iter().map(|v| v.position.z).collect::<Vec<f32>>()),
                        &correct_barycenter,
                    );

                    // 这里可以直接执行early z

                    // 对顶点着色器的用户自定义输入location进行插值给fragment shader 这里默认使用透视插值，暂时不支持其他插值
                    let fragment_input_locations: Vec<ShaderType> = {
                        let fragment_location_vec4 = primitive.iter().enumerate().fold(
                            vec![Vec4::ZERO; vertex_shader_ouput_layouts.len()],
                            |mut acc, (vindex, v)| {
                                self.shader_to_vec4(&v.location)
                                    .into_iter()
                                    .enumerate()
                                    .for_each(|(index, v)| {
                                        acc[index] += v * correct_barycenter[vindex]
                                    });
                                acc
                            },
                        );

                        self.vec4_to_shader(fragment_location_vec4, &vertex_shader_ouput_layouts)
                    };
                    // 创建fragment shader输入
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
                        location: fragment_input_locations,
                    };
                    // 顶点着色器执行
                    let fragment_output =
                        (self.state.fragment.shader)(fragment_input, &mut self.bind_groups);

                    let fragment_depth = fragment_output
                        .frag_depth
                        .unwrap_or(fragment_depth_perspective_interpolated)
                        .clamp(0.0, 1.0);
                    // 深度测试
                    // z 值从 0-1 ,这里使用bevy（因为bevy使用reverse z）的标准，默认越大越近
                    // 当然这里理论上应该是可以配置的，对应wgpu配置项  depth_compare: CompareFunction::Greater,
                    if fragment_depth <= self.depth_buffer[y * self.state.surface.width + x] {
                        continue;
                    }
                    // 深度写入
                    self.depth_buffer[y * self.state.surface.width + x] = fragment_depth;
                    // 着色器输出loaction(0)是对应的color
                    let fragment_color = fragment_output.location[0];
                    let color = match fragment_color {
                        ShaderType::Vec4(v) => v,
                        _ => panic!("error fragment output location format"),
                    };
                    // 还有模版测试 颜色混合等未实施
                    self.draw_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Vec4) {
        let width = self.state.surface.width;

        self.frame_buffer[(width * y + x) * 4] = (color.x * 255.) as u8;
        self.frame_buffer[((width * y + x) * 4) + 1] = (color.y * 255.) as u8;
        self.frame_buffer[((width * y + x) * 4) + 2] = (color.z * 255.) as u8;
        self.frame_buffer[((width * y + x) * 4) + 3] = 255;
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

        for x0 in if !x_reverse {
            (x1..x2).collect::<Vec<i32>>()
        } else {
            (x2..x1).rev().collect::<Vec<i32>>()
        } {
            if xy_reverse {
                self.draw_pixel(y0 as usize, x0 as usize, Vec4::new(1.0, 0., 0., 1.));
            } else {
                self.draw_pixel(x0 as usize, y0 as usize, Vec4::new(0., 1., 0., 1.));
            }
            delta += ky;
            if delta > middle {
                y0 += 1;
                middle += kx;
            }
        }
        None
    }

    fn vec4_to_shader(&self, vecs: Vec<Vec4>, vertex_layouts: &[VertexFormat]) -> Vec<ShaderType> {
        vertex_layouts
            .iter()
            .enumerate()
            .map(|(index, format)| match format {
                VertexFormat::Float32 => ShaderType::F32(vecs[index].x()),
                VertexFormat::Float32x2 => ShaderType::Vec2(vecs[index].xy()),
                VertexFormat::Float32x3 => ShaderType::Vec3(vecs[index].xyz()),
                VertexFormat::Float32x4 => ShaderType::Vec4(vecs[index]),
                _ => panic!(""),
            })
            .collect()
    }

    fn shader_to_vec4(&self, shaders: &Vec<ShaderType>) -> Vec<Vec4> {
        shaders
            .iter()
            .map(|&shader| match shader {
                ShaderType::F32(val) => Vec4::new(val, 0., 0., 0.),
                ShaderType::Vec2(val) => Vec4::new(val.x, val.y, 0., 0.),
                ShaderType::Vec3(val) => Vec4::new(val.x, val.y, val.z, 0.),
                ShaderType::Vec4(val) => val,
            })
            .collect()
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
fn calculate_polygon_aabb(coordinates: &[Vec2]) -> [usize; 4] {
    let mut aabb = [usize::MAX, usize::MAX, 0, 0];
    for coordinate in coordinates {
        aabb[0] = aabb[0].min(coordinate.x.floor() as usize);
        aabb[1] = aabb[1].min(coordinate.y.floor() as usize);
        aabb[2] = aabb[2].max(coordinate.x.ceil() as usize);
        aabb[3] = aabb[3].max(coordinate.y.ceil() as usize);
    }
    aabb
}

// 计算点p对于平面多边形的重心坐标
// https://gpuweb.github.io/gpuweb/#barycentric-coordinates
fn calculate_polygon_barycenter(p: Vec2, polygon: &[Vec2], area: f32) -> Vec<f32> {
    let polygon_len = polygon.len();
    let mut res = vec![0.; polygon_len];
    for i in 0..polygon_len {
        let lamda = (p - polygon[i]).cross(p - polygon[(i + 1) % polygon_len]) / area;
        res[(polygon_len - 1 + i) % polygon_len] = lamda;
    }
    res
}

// 计算重心参数的矫正
pub fn perspective_correct(barycenter: &[f32], divisors: &[f32]) -> Vec<f32> {
    let iterator = (0..barycenter.len()).map(|i| barycenter[i] * divisors[i]);
    let sum: f32 = iterator.clone().sum();
    iterator.map(|x| x / sum).collect()
}

// 计算插值
pub fn interpolate(val: &[f32], weights: &[f32]) -> f32 {
    assert!(val.len() == weights.len());
    (0..val.len()).fold(0., |acc, index| acc + val[index] * weights[index])
}
