mod bind_group;
mod camera;
mod color;
mod loader;
mod material;
mod math;
mod mesh;
mod primitives;
mod render;
mod shader;
mod texture;
mod transform;

#[cfg(test)]
mod tests {
    use crate::bind_group::BindGroup;
    use crate::camera::{Camera, Viewport};
    use crate::color::Color;
    use crate::loader::load_gltf;
    use crate::math::{Mat4, Vec3, Vec4};
    use crate::mesh::Vertex;
    use crate::render::Renderer;
    use crate::shader::{VertexInput, VertexOutPut, VertexShader};
    use crate::transform::Transform;

    fn create_render() -> Renderer {
        let camera = Camera {
            viewport: Viewport::new((0., 0.).into(), (1000., 1000.).into()),
            ..Default::default()
        };
        Renderer::new().with_camera(camera)
    }
    #[test]
    fn test_line() {
        let camera = Camera {
            viewport: Viewport::new((0., 0.).into(), (400., 400.).into()),
            ..Default::default()
        };
        let mut renderer = Renderer::new().with_camera(camera);
        let lines = [
            ((50., 160.).into(), (70., 80.).into()),
            ((0., 0.).into(), (10., 300.).into()),
            ((300., 200.).into(), (100., 300.).into()),
            ((300., 300.).into(), (100., 300.).into()),
            ((246., 383.).into(), (229., 388.).into()),
            ((399., 400.).into(), (200., 200.).into()),
            ((500., 0.).into(), (0., 500.).into()),
            ((500., 0.).into(), (100., 300.).into()),
            ((800., 0.).into(), (0., 800.).into()),
            ((400., 0.).into(), (0., 400.).into()),
        ];
        for line in lines {
            renderer.draw_line(line);
        }
        image::save_buffer(
            "image_line.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }

    #[test]
    fn test_mesh() {
        let mut renderer = create_render();
        let (meshs, textures) = load_gltf("./assets/sphere/sphere.gltf");
        // let meshs = load_gltf("./assets/cube/cube.gltf");
        // let meshs = load_gltf("./assets/monkey/monkey.gltf");

        renderer.camera.transform =
            Transform::from_xyz(0., 0., 4.).looking_at([0., 0., 0.].into(), Vec3::Y);

        for mut mesh in meshs {
            let model_matrix = mesh.transform.compute_matrix();
            for i in 0..mesh.vertices.len() / 3 {
                let triangle = &mut mesh.vertices[i * 3..(i * 3) + 3];
                renderer.draw_triangle(triangle, model_matrix);
            }
        }
        image::save_buffer(
            "image_mesh.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }

    #[test]
    fn test_rasterization() {
        let mut renderer = create_render();
        let t1 = [
            Vertex {
                position: Vec4::new(10., 70., 0., 1.),
                color: Some(Color::RED),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(50., 160., 0., 1.),
                color: Some(Color::GREEN),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(70., 80., 0., 1.),
                color: Some(Color::BLUE),
                ..Default::default()
            },
        ];
        let t2 = [
            Vertex {
                position: Vec4::new(180., 50., 0., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(150., 1., 0., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(70., 180., 0., 1.),
                ..Default::default()
            },
        ];
        let t3 = [
            Vertex {
                position: Vec4::new(180., 150., 0., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(120., 160., 0., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(130., 180., 0., 1.),
                ..Default::default()
            },
        ];
        renderer.rasterization(&t1);
        renderer.rasterization(&t2);
        renderer.rasterization(&t3);
        image::save_buffer(
            "image_rasterization.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }

    #[test]
    fn test_mvp() {
        let mut renderer = create_render();
        let mut triangle = [
            Vertex {
                position: Vec4::new(50., 0., -100., 1.),
                color: Some(Color::RED),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(0., 100., -100., 1.),
                color: Some(Color::GREEN),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(-50., 0., -100., 1.),
                color: Some(Color::BLUE),
                ..Default::default()
            },
        ];

        // 相机在 (0,0,300) 看向（0，0，0）
        renderer.camera.transform =
            Transform::from_xyz(0., 0., 200.).looking_at([0., 0., 0.].into(), Vec3::Y);

        let model_matrix = Mat4::IDENTITY;
        renderer.draw_triangle(&mut triangle, model_matrix);
        image::save_buffer(
            "image_mvp.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }

    #[test]
    fn test_zbuffer() {
        let mut renderer = create_render();
        // 相机在 (0,0,300) 看向（0，0，0）
        renderer.camera.transform =
            Transform::from_xyz(0., 0., 600.).looking_at([0., 0., 0.].into(), Vec3::Y);

        let mut triangle1 = [
            Vertex {
                position: Vec4::new(200., 0., 0., 1.),
                color: Some(Color::RED),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(0., 100., 0., 1.),
                color: Some(Color::RED),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(-50., 0., 0., 1.),
                color: Some(Color::RED),
                ..Default::default()
            },
        ];
        let mut triangle2 = [
            Vertex {
                position: Vec4::new(100., 0., 100., 1.),
                color: Some(Color::BLUE),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(0., 100., 100., 1.),
                color: Some(Color::BLUE),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(-50., 0., 100., 1.),
                color: Some(Color::BLUE),
                ..Default::default()
            },
        ];
        let model_matrix = Mat4::IDENTITY;
        renderer.draw_triangle(&mut triangle2, model_matrix);
        renderer.draw_triangle(&mut triangle1, model_matrix);
        image::save_buffer(
            "image_zbuffer.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }
    #[test]
    fn test_texture() {
        let mut renderer = create_render();
        renderer.camera.transform =
            Transform::from_xyz(3., 3., 3.).looking_at([0., 0., 0.].into(), Vec3::Y);

        let (meshs, textures) = load_gltf("./assets/box-textured/BoxTextured.gltf");
        renderer.set_binding(0, textures);
        for mut mesh in meshs {
            let model_matrix = mesh.transform.compute_matrix();
            for i in 0..mesh.vertices.len() / 3 {
                let triangle = &mut mesh.vertices[i * 3..(i * 3) + 3];
                renderer.draw_triangle(triangle, model_matrix);
            }
        }
        image::save_buffer(
            "image_texture.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }

    #[test]
    fn test_texture_robot() {
        let mut renderer = create_render();
        renderer.camera.transform =
            Transform::from_xyz(0., 0., 20.).looking_at([0., 0., 0.].into(), Vec3::Y);

        let (meshs, textures) = load_gltf("./assets/assistrobot/scene.gltf");
        renderer.set_binding(0, textures);
        for mut mesh in meshs {
            let model_matrix = mesh.transform.compute_matrix();
            for i in 0..mesh.vertices.len() / 3 {
                let triangle = &mut mesh.vertices[i * 3..(i * 3) + 3];
                renderer.draw_triangle(triangle, model_matrix);
            }
        }
        image::save_buffer(
            "image_texture.png",
            &renderer.frame_buffer,
            renderer.camera.viewport.physical_size.x as u32,
            renderer.camera.viewport.physical_size.y as u32,
            image::ColorType::Rgb8,
        )
        .unwrap();
    }

    #[test]
    fn test_custum_shader() {
        let mut renderer = create_render();
        let a = VertexInput {
            vertex_index: 0,
            instance_index: 0,
            location: Vec::new(),
        };
        let shader: VertexShader = |v: VertexInput, group: &BindGroup| {
            return VertexOutPut {
                position: Vec4::ZERO,
                location: Vec::new(),
            };
        };
        shader(a, &renderer.bind_group);
    }
}
fn main() {}
