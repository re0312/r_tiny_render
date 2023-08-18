use loader::load_gltf;
use math::{Mat4, Vec2, Vec3, Vec4};
use pipeline::{
    texture_sample, BindGroup, BindType, FragmentInput, FragmentOutput, FragmentState,
    RenderSurface, Renderer, RendererDescriptor, Sampler, ShaderType, Texture, TextureFormat,
    VertexInput, VertexOutput, VertexState,
};
use render::{shader_uniform::ViewUniform, Camera, Transform};

fn vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
    let mut out = VertexOutput {
        location: vec![ShaderType::Vec4(Vec4::ZERO), ShaderType::Vec2(Vec2::ONE)],
        position: Vec4::ONE,
    };
    let in_postion: Vec3 = vertex_input.location[0].into();
    let in_normal: Vec3 = vertex_input.location[1].into();
    let in_texture_uv: Vec2 = vertex_input.location[2].into();
    println!("texture_uv_vertex:{:?}",in_texture_uv);

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    let texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();

    let clip_postion = view_uniform.view_proj * in_postion.extend(1.);
    println!("{:?}",clip_postion);

    // 还原bindgroup
    bind_groups[0][0] = view_uniform.into();
    bind_groups[1][1] = texture.into();
    bind_groups[1][2] = sampler.into();

    out.position = clip_postion;
    out.location[0] = in_normal.into();
    out.location[1] = in_texture_uv.into();
    out
}

fn fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let in_normal: Vec3 = input.location[0].into();
    let in_texture_uv: Vec2 = input.location[1].into();
    // println!("tex_coord:{:?}", in_texture_uv);

    let view_uniform: ViewUniform = std::mem::take(&mut bind_groups[0][0]).into();
    // println!("view_uniform:{:?}", view_uniform);
    let texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();

    let in_color = texture_sample(&texture, &sampler, in_texture_uv);

    // 还原bindgroup
    bind_groups[0][0] = view_uniform.into();
    bind_groups[1][1] = texture.into();
    bind_groups[1][2] = sampler.into();
    FragmentOutput {
        frag_depth: 0.5,
        sample_mask: 0,
        location: vec![ShaderType::Vec4(in_color)],
    }
}

fn main() {
    // let (meshs, materials) = load_gltf(
    //     "C:\\Users\\27135\\Desktop\\project\\rust\\r_tinny_render\\assets\\box-textured\\BoxTextured.gltf",
    // );
    let (meshs, materials) = load_gltf(
        "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/box-textured/BoxTextured.gltf",
    );

    let mesh = &meshs[0];
    let material = &materials[0];

    let desc = RendererDescriptor {
        surface: RenderSurface {
            format: TextureFormat::Rgba8Unorm,
            height: 100,
            width: 100,
        },
        vertex: VertexState {
            shader: vertex_main,
            layout: &mesh.get_vertex_buffer_layout(),
        },
        fragment: FragmentState {
            shader: fragment_main,
        },
    };

    let camera = Camera::default()
        .with_transform(Transform::from_xyz(0., 0., 2.).looking_at(Vec3::ZERO, Vec3::Y));
    let camera_uniform = camera.get_camera_uniform();
    let c = bytemuck::cast_slice::<_, u8>(&[camera_uniform]).to_vec();
    let bind_group_0 = vec![BindType::Uniform(c)];
    let mut renderer = Renderer::new(desc);
    let vertex_buffer = mesh.get_vertex_buffer_data();
    let index_buffer = mesh.get_index_buffer_data();
    let bind_group_material = material.get_materail_bind_group();

    println!("layout: {:?}", mesh.get_vertex_buffer_layout());
    println!("vertex count:{:?}",mesh.count_vertices());
    println!("indices count:{:?}",mesh.count_indices());

    renderer.set_vertex_buffer(&vertex_buffer);
    renderer.set_index_buffer(&index_buffer);
    renderer.set_bind_group(0, bind_group_0);
    renderer.set_bind_group(1, bind_group_material);
    renderer.draw_indexed(0..mesh.count_indices() as u32);
    image::save_buffer(
        "image_mesh.png",
        &renderer.frame_buffer,
        100,
        100,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
