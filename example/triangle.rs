use math::{Vec2, Vec3, Vec4};
use pipeline::{
    BindGroup, FragmentInput, FragmentOutput, FragmentState, RenderSurface, Renderer,
    RendererDescriptor, ShaderType, TextureFormat, VertexFormat, VertexInput, VertexOutput,
    VertexState,
};

fn vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
    let mut out = VertexOutput {
        location: vec![ShaderType::Vec4(Vec4::ZERO), ShaderType::Vec2(Vec2::ONE)],
        position: Vec4::ONE,
    };
    let in_postion: Vec3 = vertex_input.location[0].into();
    let in_color: Vec4 = vertex_input.location[1].into();
    println!("vertex postion:{:?}", in_postion);
    println!("vertex color:{:?}", in_color);
    out.position = Vec4::new(in_postion.x, in_postion.y, in_postion.z, 1.);
    out.location[0] = in_color.into();
    out
}

fn fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let in_color: Vec4 = input.location[0].into();
    FragmentOutput {
        frag_depth: None,
        sample_mask: 0,
        location: vec![ShaderType::Vec4(in_color)],
    }
}
fn main() {
    // 数组长度0..3是坐标，3..7是顶点颜色
    let vertex_buffer: Vec<[f32; 7]> = vec![
        [0., -0.5, 1., 1., 0., 0., 1.],
        [-0.5, 0.5, 1., 0., 1., 0., 1.],
        [0.5, 0.5, 1., 0., 0., 1., 1.],
    ];
    let desc = RendererDescriptor {
        surface: RenderSurface {
            format: TextureFormat::Rgba8Unorm,
            height: 1000,
            width: 1000,
        },
        vertex: VertexState {
            shader: vertex_main,
            layout: &[VertexFormat::Float32x3, VertexFormat::Float32x4],
        },
        fragment: FragmentState {
            shader: fragment_main,
        },
    };
    let mut renderer = Renderer::new(desc);
    renderer.set_vertex_buffer(bytemuck::cast_slice(&vertex_buffer));
    renderer.draw(0..vertex_buffer.len() as u32);
    image::save_buffer(
        "image_triangle.png",
        &renderer.frame_buffer,
        1000,
        1000,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
