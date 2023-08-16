use loader::load_gltf;
use math::{Vec2, Vec3, Vec4};
use render::Mesh;
use renderer::{
    BindGroup, FragmentInput, FragmentOutput, FragmentState, RenderSurface, Renderer,
    RendererDescriptor, ShaderType, TextureFormat, VertexInput, VertexOutput, VertexState,
};

fn vertex_main(vertex_input: VertexInput, bind_groups: &Vec<BindGroup>) -> VertexOutput {
    let mut out = VertexOutput {
        location: vec![ShaderType::Vec4(Vec4::ZERO), ShaderType::Vec2(Vec2::ONE)],
        position: Vec4::ONE,
    };
    let in_postion: Vec3 = vertex_input.location[0].into();
    // println!("vertex postion: {:?}", in_postion);
    let in_normal: Vec3 = vertex_input.location[1].into();
    let in_texture_uv: Vec2 = vertex_input.location[2].into();
    // let in_color: Vec4 = vertex_input.location[4].into();
    let in_color = Vec4::new(1., 0., 0., 1.);
    println!("vertex color: {:?}", in_texture_uv);
    out.position = Vec4::new(in_postion.x / 1., in_postion.y / 1., 0., 1.);
    out.location[0] = in_color.into();
    // out.location[0] = Vec4::new(1., 1., 1., 1.).into();
    out
}

fn fragment_main(input: FragmentInput, bind_groups: &Vec<BindGroup>) -> FragmentOutput {
    let in_color: Vec4 = input.location[0].into();
    FragmentOutput {
        frag_depth: 0.5,
        sample_mask: 0,
        location: vec![ShaderType::Vec4(in_color)],
    }
}

fn main() {
    let meshs = load_gltf(
        "C:\\Users\\27135\\Desktop\\project\\rust\\r_tinny_render\\assets\\cube\\cube.gltf",
    );
    let mesh = &meshs[0];
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
        bind_group_count: 0,
    };
    let mut renderer = Renderer::new(desc);
    let vertex_buffer = mesh.get_vertex_buffer_data();
    let index_buffer = mesh.get_index_buffer_data();

    println!("layout: {:?}", mesh.get_vertex_buffer_layout());
    renderer.set_vertex_buffer(&vertex_buffer);
    renderer.set_index_buffer(&index_buffer);
    renderer.draw_indexed(0..mesh.count_indices() as u32);
    image::save_buffer(
        "image_mesh.png",
        &renderer.frame_buffer,
        1000,
        1000,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
