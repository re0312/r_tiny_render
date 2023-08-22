use loader::load_gltf;
use math::{Vec2, Vec3, Vec4};
use pipeline::{
    texture_sample, BindGroup, BindType, FragmentInput, FragmentOutput, FragmentState,
    RenderSurface, Renderer, RendererDescriptor, Sampler, ShaderType, Texture, TextureFormat,
    VertexInput, VertexOutput, VertexState,
};
use render::{
    pbr_shder::{fragment_main, vertex_main},
    shader_uniform::{MeshUniform, ViewUniform},
    Camera, PointLight, Transform,
};

fn main() {
    let (meshs, materials) = load_gltf(
        "C:\\Users\\27135\\Desktop\\project\\rust\\r_tinny_render\\assets\\assistrobot\\scene.gltf",
    );
    // let (meshs, materials) = load_gltf(
    //     "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/box-textured/BoxTextured.gltf",
    // );
    // let (meshs, materials) = load_gltf(
    //     "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/assistrobot/scene.gltf",
    // );

    let mesh = &meshs[0];
    let material = &materials[0];
    let mesh_tranform = Transform::from_xyz(0., 0., 0.);
    let mesh_uniform = MeshUniform {
        model: mesh_tranform.compute_matrix().inverse(),
        inverse_transpose_model: mesh_tranform.compute_matrix().transpose(),
    };

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

    let camera = Camera::default()
        .with_transform(Transform::from_xyz(2., 2., 2.).looking_at(Vec3::ZERO, Vec3::Y));

    let light = PointLight {
        transform: Transform::from_xyz(0., 100., 0.),
        ..Default::default()
    };
    let mut renderer = Renderer::new(desc);
    let vertex_buffer = mesh.get_vertex_buffer_data();
    let index_buffer = mesh.get_index_buffer_data();

    let bind_group_0 = vec![
        camera.get_camera_uniform().into(),
        light.get_point_light_uniform().into(),
    ];
    let bind_group_material = material.get_material_bind_group();
    let bind_group_mesh = vec![mesh_uniform.into()];

    println!("layout: {:?}", mesh.get_vertex_buffer_layout());
    println!("vertex count:{:?}", mesh.count_vertices());
    println!("indices count:{:?}", mesh.count_indices());

    renderer.set_vertex_buffer(&vertex_buffer);
    renderer.set_index_buffer(&index_buffer);
    renderer.set_bind_group(0, bind_group_0);
    renderer.set_bind_group(1, bind_group_material);
    renderer.set_bind_group(2, bind_group_mesh);
    renderer.draw_indexed(0..mesh.count_indices() as u32);
    image::save_buffer(
        "image_shading.png",
        &renderer.frame_buffer,
        1000,
        1000,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
