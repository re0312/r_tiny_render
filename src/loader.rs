use std::collections::HashMap;

use crate::{
    color::Color,
    math::{Vec3, Vec4},
    mesh::{Mesh, Vertex},
    texture::{Sampler, Texture},
};
use gltf::{buffer::Data, Document};

// pub fn load_obj(path: &str) {
//     let mut buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 3) as usize];
//     let (models, materials) = tobj::load_obj(path, &tobj::LoadOptions::default()).unwrap();
//     for (i, m) in models.iter().enumerate() {
//         let mesh = &m.mesh;
//         println!("");
//         println!("model[{}].name             = \'{}\'", i, m.name);
//         println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);
//         println!(
//             "model[{}].face_count       = {}",
//             i,
//             mesh.face_arities.len()
//         );
//         let scale = 300.;
//         for indices in 0..m.mesh.indices.len() / 3 {
//             let i1 = mesh.indices[indices * 3];
//             let i2 = mesh.indices[indices * 3 + 1];
//             let i3 = mesh.indices[indices * 3 + 2];
//             let mut x1 = mesh.positions[i1 as usize * 3] / scale;
//             let mut y1 = mesh.positions[i1 as usize * 3 + 1] / scale;
//             let mut x2 = mesh.positions[i2 as usize * 3] / scale;
//             let mut y2 = mesh.positions[i2 as usize * 3 + 1] / scale;
//             let mut x3 = mesh.positions[i3 as usize * 3] / scale;
//             let mut y3 = mesh.positions[i3 as usize * 3 + 1] / scale;
//             let x1 = ((x1 + 1.) * WIDTH as f32 / 2 as f32) as u32;
//             let y1 = ((y1 + 1.) * HEIGHT as f32 / 2 as f32) as u32;
//             let x2 = ((x2 + 1.) * WIDTH as f32 / 2 as f32) as u32;
//             let y2 = ((y2 + 1.) * HEIGHT as f32 / 2 as f32) as u32;
//             let x3 = ((x3 + 1.) * WIDTH as f32 / 2 as f32) as u32;
//             let y3 = ((y3 + 1.) * HEIGHT as f32 / 2 as f32) as u32;
//             // println!("({},{}),({},{}),({},{})", x1, y1, x2, y2, x3, y3);
//             draw_line(
//                 x1 as i32,
//                 y1 as i32,
//                 x2 as i32,
//                 y2 as i32,
//                 &mut buffer,
//                 Color::WHITE,
//             );
//             draw_line(
//                 x2 as i32,
//                 y2 as i32,
//                 x3 as i32,
//                 y3 as i32,
//                 &mut buffer,
//                 Color::WHITE,
//             );
//             draw_line(
//                 x3 as i32,
//                 y3 as i32,
//                 x1 as i32,
//                 y1 as i32,
//                 &mut buffer,
//                 Color::WHITE,
//             );
//         }
//         println!(
//             "model[{}].positions        = {}",
//             i,
//             mesh.positions.len() / 3
//         );
//         assert!(mesh.positions.len() % 3 == 0);
//     }
//     image::save_buffer("image.png", &buffer, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
// }

#[derive(Debug, Default)]
pub struct TextureStorage {
    pub texture_id_map: HashMap<usize, Texture>,
}
pub fn load_gltf(path: &str) -> (Vec<Mesh>, TextureStorage) {
    let (document, buffers, images) = gltf::import(path).unwrap();

    // 加载mesh
    let mut meshs = Vec::new();
    for mesh in document.meshes() {
        for gltf_primitive in mesh.primitives() {
            let mut mesh = Mesh::default();
            if gltf_primitive.mode() != gltf::mesh::Mode::Triangles {
                panic!("gltf format not support!");
            }
            let reader = gltf_primitive.reader(|b| Some(&buffers[b.index()]));
            let mut positions: Vec<[f32; 3]> = Vec::new();
            let mut normals: Vec<[f32; 3]> = Vec::new();
            let mut colors: Vec<[f32; 3]> = Vec::new();
            let mut texcoords: Vec<[f32; 2]> = Vec::new();

            for (semantic, _) in gltf_primitive.attributes() {
                match semantic {
                    gltf::Semantic::Positions => {
                        positions = reader.read_positions().unwrap().collect();
                    }
                    gltf::Semantic::Normals => {
                        normals = reader.read_normals().unwrap().collect();
                    }
                    gltf::Semantic::Colors(set) => {
                        colors = reader.read_colors(set).unwrap().into_rgb_f32().collect();
                    }
                    gltf::Semantic::TexCoords(set) => {
                        texcoords = reader.read_tex_coords(set).unwrap().into_f32().collect();
                    }
                    _ => {}
                }
            }
            let Some(indices) =  reader.read_indices() else {
                continue;
            };
            let indices = indices.into_u32();
            for index in indices {
                let vertex = Vertex {
                    position: Vec3::from(positions.get(index as usize).unwrap().clone()).extend(1.),
                    normal: normals.get(index as usize).map(|v| v.clone().into()),
                    texcoord: texcoords.get(index as usize).map(|v| v.clone().into()),
                    color: colors
                        .get(index as usize)
                        .map(|v| v.clone().into())
                        .or(Some(Color::WHITE)),
                };
                mesh.vertices.push(vertex);
            }
            meshs.push(mesh);
        }
        #[cfg(feature = "info")]
        println!("positions:{:?}", res);
    }

    // 纹理加载
    let mut textures = Vec::new();
    for texture in document.textures() {
        let source = texture.source();
        let sampler = texture.sampler();
        let image = images.get(source.index()).unwrap();

        let texture = Texture {
            id: texture.index(),
            width: image.width,
            height: image.height,
            format: image.format,
            data: image.pixels.clone(),
            sampler: Sampler {
                mag_filter: sampler.mag_filter(),
                min_filter: sampler.min_filter(),
                wrap_s: sampler.wrap_s(),
                wrap_t: sampler.wrap_t(),
            },
        };
        textures.push(texture);
    }
    (
        meshs,
        TextureStorage {
            texture_id_map: textures
                .into_iter()
                .map(|textrue| (textrue.id, textrue))
                .collect(),
        },
    )
}
