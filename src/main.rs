mod camera;
mod color;
mod loader;
mod material;
mod math;
mod mesh;
mod render;
mod transform;

#[cfg(test)]
mod tests {
    use crate::camera::{Camera, Viewport};
    use crate::color::Color;
    use crate::loader::load_gltf;
    use crate::math::{Mat4, Vec3, Vec4};
    use crate::mesh::Vertex;
    use crate::render::Renderer;
    use crate::transform::Transform;

    fn create_render() -> Renderer {
        let camera = Camera {
            viewport: Viewport::new((0., 0.).into(), (400., 400.).into()),
            ..Default::default()
        };
        Renderer::new().with_camera(camera)
    }
    #[test]
    fn test_line() {
        let mut renderer = create_render();
        let lines = [
            ((50., 160.).into(), (70., 80.).into()),
            ((0., 0.).into(), (10., 300.).into()),
            ((300., 200.).into(), (100., 300.).into()),
            ((300., 300.).into(), (100., 300.).into()),
            ((246., 383.).into(), (229., 388.).into()),
            ((399., 400.).into(), (200., 200.).into()),
            ((3909., 4000.).into(), (2000., 2000.).into()),
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
        let meshs = load_gltf("./assets/sphere/sphere.gltf");

        // 相机在 (0,0,200) 看向（0，0，0）
        renderer.camera.transform =
            Transform::from_xyz(0., 0., 2.7).looking_at([0., 0., 0.].into(), Vec3::Y);

        for mut mesh in meshs {
            renderer.draw_mesh(&mut mesh);
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
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(50., 160., 0., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(70., 80., 0., 1.),
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
                position: Vec4::new(50., 0., -400., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(0., 100., -400., 1.),
                ..Default::default()
            },
            Vertex {
                position: Vec4::new(-50., 0., -400., 1.),
                ..Default::default()
            },
        ];

        // 相机在 (0,0,300) 看向（0，0，0）
        renderer.camera.transform =
            Transform::from_xyz(0., 0., 300.).looking_at([0., 0., 0.].into(), Vec3::Y);

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
}
fn main() {}
