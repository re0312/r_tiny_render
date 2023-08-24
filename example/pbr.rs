use loader::load_gltf;
use math::Vec3;
use pipeline::{
    FragmentState, RenderSurface, Renderer, RendererDescriptor, TextureFormat, VertexState,
};
use render::{
    pbr_shder::{pbr_fragment_main, pbr_vertex_main},
    shader_uniform::MeshUniform,
    Camera, PointLight, Transform,
};

fn main() {
    // let (meshs, materials) = load_gltf(
    //     "C:\\Users\\27135\\Desktop\\project\\rust\\r_tinny_render\\assets\\assistrobot\\scene.gltf",
    // );
    // let (meshs, materials) = load_gltf(
    //     "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/box-textured/BoxTextured.gltf",
    // );
    let (meshs, materials) = load_gltf(
        "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/assistrobot/scene.gltf",
    );

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
            height: 2000,
            width: 2000,
        },
        vertex: VertexState {
            shader: pbr_vertex_main,
            layout: &mesh.get_vertex_buffer_layout(),
        },
        fragment: FragmentState {
            shader: pbr_fragment_main,
        },
    };

    let camera = Camera::default()
        .with_transform(Transform::from_xyz(0., 5., 16.).looking_at(Vec3::ZERO, Vec3::NEG_Y));

    let light = PointLight {
        intensity: 1000.,
        range: 30.,
        transform: Transform::from_xyz(0., 0., 16.),
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
        "image_pbr.png",
        &renderer.frame_buffer,
        2000,
        2000,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
