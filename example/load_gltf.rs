use loader::load_gltf;
use math::{Vec2, Vec3, Vec4};
use renderer::{
    texture_sample, BindGroup, BindType, FragmentInput, FragmentOutput, FragmentState,
    RenderSurface, Renderer, RendererDescriptor, Sampler, ShaderType, Texture, TextureFormat,
    VertexInput, VertexOutput, VertexState,
};

fn vertex_main(vertex_input: VertexInput, bind_groups: &mut Vec<BindGroup>) -> VertexOutput {
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

fn fragment_main(input: FragmentInput, bind_groups: &mut Vec<BindGroup>) -> FragmentOutput {
    let in_normal: Vec3 = input.location[0].into();
    let in_texture_uv: Vec2 = input.location[1].into();
    // println!("tex_coord:{:?}", in_texture_uv);

    let texture: Texture = std::mem::take(&mut bind_groups[1][1]).into();
    let sampler: Sampler = std::mem::take(&mut bind_groups[1][2]).into();

    let in_color = texture_sample(&texture, &sampler, in_texture_uv);

    // 还原bindgroup
    bind_groups[1][1] = BindType::Texture(texture);
    bind_groups[1][2] = BindType::Sampler(sampler);
    FragmentOutput {
        frag_depth: 0.5,
        sample_mask: 0,
        location: vec![ShaderType::Vec4(in_color)],
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    view: Vec4,
    flag: u32,
}
fn main() {
    let test = Camera {
        view: Vec4 {
            x: 1.,
            y: 2.,
            z: 3.,
            w: 4.,
        },
        flag: 5,
    };
    let c = bytemuck::cast_slice::<_, u8>(&[test]).to_vec();
    let d: Camera = unsafe {
        let d = c.as_ptr() as *const Camera;
        *d
    };
    println!("{:?}", d);
    // let (meshs, materials) = load_gltf(
    //     "C:\\Users\\27135\\Desktop\\project\\rust\\r_tinny_render\\assets\\box-textured\\BoxTextured.gltf",
    // );
    // // let meshs = load_gltf(
    // //     "/home/10337136@zte.intra/Desktop/rust/r_tinny_render/assets/assistrobot/scene.gltf",
    // // );
    // let mesh = &meshs[0];
    // let material = &materials[0];
    // let desc = RendererDescriptor {
    //     surface: RenderSurface {
    //         format: TextureFormat::Rgba8Unorm,
    //         height: 1000,
    //         width: 1000,
    //     },
    //     vertex: VertexState {
    //         shader: vertex_main,
    //         layout: &mesh.get_vertex_buffer_layout(),
    //     },
    //     fragment: FragmentState {
    //         shader: fragment_main,
    //     },
    // };
    // let mut renderer = Renderer::new(desc);
    // let vertex_buffer = mesh.get_vertex_buffer_data();
    // let index_buffer = mesh.get_index_buffer_data();
    // let bind_group_material = material.get_materail_bind_group();

    // println!("layout: {:?}", mesh.get_vertex_buffer_layout());

    // renderer.set_vertex_buffer(&vertex_buffer);
    // renderer.set_index_buffer(&index_buffer);
    // renderer.set_bind_group(1, bind_group_material);
    // renderer.draw_indexed(0..mesh.count_indices() as u32);
    // image::save_buffer(
    //     "image_mesh.png",
    //     &renderer.frame_buffer,
    //     1000,
    //     1000,
    //     image::ColorType::Rgba8,
    // )
    // .unwrap();
}
