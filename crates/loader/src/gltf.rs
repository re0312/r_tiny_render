use gltf::image;
use pipeline::{Texture, TextureFormat};
use render::{Color, Mesh, StandardMaterial};

pub fn load_gltf(path: &str) -> (Vec<Mesh>, Vec<StandardMaterial>) {
    let (document, buffers, images) = gltf::import(path).unwrap();

    // 加载mesh
    let mut meshs = Vec::new();
    for mesh in document.meshes() {
        for gltf_primitive in mesh.primitives() {
            let mut mesh = Mesh::new();
            if gltf_primitive.mode() != gltf::mesh::Mode::Triangles {
                panic!("gltf format not support!");
            }
            let reader = gltf_primitive.reader(|b| Some(&buffers[b.index()]));

            for (semantic, _) in gltf_primitive.attributes() {
                match semantic {
                    gltf::Semantic::Positions => {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            reader.read_positions().unwrap().collect::<Vec<[f32; 3]>>(),
                        );
                        // positions = reader.read_positions().unwrap().collect();
                    }
                    gltf::Semantic::Normals => {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_NORMAL,
                            reader.read_normals().unwrap().collect::<Vec<[f32; 3]>>(),
                        );
                    }
                    gltf::Semantic::Colors(set) => {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_COLOR,
                            reader
                                .read_colors(set)
                                .unwrap()
                                .into_rgba_f32()
                                .collect::<Vec<[f32; 4]>>(),
                        );
                    }
                    gltf::Semantic::TexCoords(0) => {
                        println!("coordinate:{}", 0);
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_UV_0,
                            reader
                                .read_tex_coords(0)
                                .unwrap()
                                .into_f32()
                                .collect::<Vec<[f32; 2]>>(),
                        );
                    }
                    gltf::Semantic::Tangents => {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_TANGENT,
                            reader.read_tangents().unwrap().collect::<Vec<[f32; 4]>>(),
                        );
                    }
                    _ => {
                        println!("other semantic:{:?}", semantic)
                    }
                }
            }
            if let Some(indices) = reader.read_indices() {
                let indices: Vec<u32> = indices.into_u32().collect();
                mesh.set_indices(indices);
            }
            meshs.push(mesh);
        }
    }

    // 加载纹理
    let mut materials = Vec::new();
    for material in document.materials() {
        let mut standard_material = StandardMaterial::default();
        let pbr = material.pbr_metallic_roughness();

        standard_material.base_color = Color::from_vec4(pbr.base_color_factor().into());
        standard_material.perceptual_roughness = pbr.roughness_factor();
        standard_material.metallic = pbr.metallic_factor();
        standard_material.emissive = Color::from_vec3(material.emissive_factor().into());

        standard_material.base_color_texture = pbr.base_color_texture().map(|info| {
            let source = info.texture().source();
            let image = images.get(source.index()).unwrap();
            Texture {
                width: image.width,
                height: image.height,
                format: match image.format {
                    image::Format::R8 => TextureFormat::R8Unorm,
                    image::Format::R8G8B8 => TextureFormat::Rgb8Unorm,
                    image::Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
                    _ => TextureFormat::Rgba8Unorm,
                },
                data: image.pixels.clone(),
            }
        });

        standard_material.metallic_roughness_texture = pbr.metallic_roughness_texture().map(|v| {
            let source = v.texture().source();
            let image = images.get(source.index()).unwrap();
            Texture {
                width: image.width,
                height: image.height,
                format: match image.format {
                    image::Format::R8 => TextureFormat::R8Unorm,
                    image::Format::R8G8B8 => TextureFormat::Rgb8Unorm,
                    image::Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
                    _ => TextureFormat::Rgba8Unorm,
                },
                data: image.pixels.clone(),
            }
        });

        standard_material.normal_map_texture = material.normal_texture().map(|v| {
            let source = v.texture().source();
            let image = images.get(source.index()).unwrap();
            Texture {
                width: image.width,
                height: image.height,
                format: match image.format {
                    image::Format::R8 => TextureFormat::R8Unorm,
                    image::Format::R8G8B8 => TextureFormat::Rgb8Unorm,
                    image::Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
                    _ => TextureFormat::Rgba8Unorm,
                },
                data: image.pixels.clone(),
            }
        });

        standard_material.emissive_texture = material.emissive_texture().map(|v| {
            let source = v.texture().source();
            let image = images.get(source.index()).unwrap();
            Texture {
                width: image.width,
                height: image.height,
                format: match image.format {
                    image::Format::R8 => TextureFormat::R8Unorm,
                    image::Format::R8G8B8 => TextureFormat::Rgb8Unorm,
                    image::Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
                    _ => TextureFormat::Rgba8Unorm,
                },
                data: image.pixels.clone(),
            }
        });
        materials.push(standard_material);
    }
    (meshs, materials)
}
