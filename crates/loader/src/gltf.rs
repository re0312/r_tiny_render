use render::Mesh;

pub fn load_gltf(path: &str) -> Vec<Mesh> {
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
                    gltf::Semantic::TexCoords(set) => {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_UV_0,
                            reader
                                .read_tex_coords(set)
                                .unwrap()
                                .into_f32()
                                .collect::<Vec<[f32; 2]>>(),
                        );
                    }
                    _ => {}
                }
            }
            if let Some(indices) = reader.read_indices() {
                let indices: Vec<u32> = indices.into_u32().collect();
                mesh.set_indices(indices);
            }
            meshs.push(mesh);
        }
    }
    meshs

    // 纹理加载
    // let mut textures = Vec::new();
    // for texture in document.textures() {
    //     let source = texture.source();
    //     let sampler = texture.sampler();
    //     let image = images.get(source.index()).unwrap();

    //     let texture = Texture {
    //         id: texture.index(),
    //         width: image.width,
    //         height: image.height,
    //         format: image.format,
    //         data: image.pixels.clone(),
    //         sampler: Sampler {
    //             mag_filter: sampler.mag_filter(),
    //             min_filter: sampler.min_filter(),
    //             wrap_s: sampler.wrap_s(),
    //             wrap_t: sampler.wrap_t(),
    //         },
    //     };
    //     textures.push(texture);
    // }
    // (
    //     meshs,
    //     TextureStorage {
    //         texture_id_map: textures
    //             .into_iter()
    //             .map(|textrue| (textrue.id, textrue))
    //             .collect(),
    //     },
    // )
}
