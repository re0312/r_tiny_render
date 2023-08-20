use math::{Vec2, Vec3, Vec4};
use render::Mesh;
use pipeline::{
    BindGroup, FragmentInput, FragmentOutput, FragmentState, RenderSurface, Renderer,
    RendererDescriptor, ShaderType, TextureFormat, VertexInput, VertexOutput, VertexState,
};

fn vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
    let in_postion: Vec3 = vertex_input.location[0].into();
    let in_color: Vec4 = vertex_input.location[1].into();

    println!("vertex postion:{:?}", in_postion);
    println!("vertex color:{:?}", in_color);

    let mut out = VertexOutput {
        location: vec![ShaderType::Vec4(Vec4::ZERO), ShaderType::Vec2(Vec2::ONE)],
        position: Vec4::ONE,
    };

    out.position = Vec4::new(in_postion.x, in_postion.y, in_postion.z, 1.);
    out.location[0] = in_color.into();
    out
}

fn fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let in_color: Vec4 = input.location[0].into();
    FragmentOutput {
        frag_depth: None,
        sample_mask: 2,
        location: vec![ShaderType::Vec4(in_color)],
    }
}

fn main() {
    let mut mesh = Mesh::new();
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0.5, 0., 0.], [0., -1., 0.], [0.5, 0., 0.]],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]],
    );
    println!("{:?}", mesh.get_vertex_buffer_layout());
    let desc = RendererDescriptor {
        surface: RenderSurface {
            format: TextureFormat::Rgba8Unorm,
            height: 1000,
            width: 1000,
        },
        vertex: VertexState {
            shader: vertex_main,
            layout: &mesh.get_vertex_buffer_layout(),
        },
        fragment: FragmentState {
            shader: fragment_main,
        },
    };
    let mut renderer = Renderer::new(desc);
    let binding = mesh.get_vertex_buffer_data();
    renderer.set_vertex_buffer(&binding);
    renderer.draw(0..mesh.count_vertices() as u32);
    image::save_buffer(
        "image_mesh.png",
        &renderer.frame_buffer,
        1000,
        1000,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
