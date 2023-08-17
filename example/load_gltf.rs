use loader::load_gltf;
use math::{Vec2, Vec3, Vec4};
use renderer::{
    texture_sample, BindGroup, FragmentInput, FragmentOutput, FragmentState, RenderSurface,
    Renderer, RendererDescriptor, Sampler, ShaderType, Texture, TextureFormat, VertexInput,
    VertexOutput, VertexState,
};

fn vertex_main(vertex_input: VertexInput, bind_groups: &Vec<BindGroup>) -> VertexOutput {
    let mut out = VertexOutput {
        location: vec![ShaderType::Vec4(Vec4::ZERO), ShaderType::Vec2(Vec2::ONE)],
        position: Vec4::ONE,
    };
    let in_postion: Vec3 = vertex_input.location[0].into();
    let in_normal: Vec3 = vertex_input.location[1].into();
    let in_texture_uv: Vec2 = vertex_input.location[2].into();

    // println!("vertex postion: {:?}", in_postion);
    println!("vertex texture uv: {:?}", in_texture_uv);
    out.position = Vec4::new(in_postion.x / 1., in_postion.y / 1., 0.5, 1.);
    out.location[0] = in_normal.into();
    out.location[1] = in_texture_uv.into();
    // out.location[0] = Vec4::new(1., 1., 1., 1.).into();
    out
}

fn fragment_main(input: FragmentInput, bind_groups: &Vec<BindGroup>) -> FragmentOutput {
    let in_normal: Vec3 = input.location[0].into();
    let in_texture_uv: Vec2 = input.location[1].into();
    // println!("tex_coord:{:?}", in_texture_uv);

    let texture: Texture = bind_groups[1][1].clone().into();
    let sampler: Sampler = bind_groups[1][2].clone().into();
    let in_color = texture_sample(&texture, sampler, in_texture_uv);

    FragmentOutput {
        frag_depth: 0.5,
        sample_mask: 0,
        location: vec![ShaderType::Vec4(in_color)],
    }
}

fn main() {
    let (meshs, materials) = load_gltf(
        "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/box-textured/BoxTextured.gltf",
    );
    // let meshs = load_gltf(
    //     "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/assistrobot/scene.gltf",
    // );
    let mesh = &meshs[0];
    let material = &materials[0];
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
        bind_group_count: 2,
    };
    let mut renderer = Renderer::new(desc);
    let vertex_buffer = mesh.get_vertex_buffer_data();
    let index_buffer = mesh.get_index_buffer_data();
    let bind_group_material = material.get_materail_bind_group();

    println!("layout: {:?}", mesh.get_vertex_buffer_layout());

    renderer.set_vertex_buffer(&vertex_buffer);
    renderer.set_index_buffer(&index_buffer);
    renderer.set_bind_group(1, bind_group_material);
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
