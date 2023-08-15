use bytemuck::cast_slice;
use renderer::VertexFormat;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MeshVertexAttributeId(usize);
#[derive(Debug, Clone)]

struct MeshAttributeData {
    attribute: MeshVertexAttribute,
    values: VertexAttributeValues,
}

#[derive(Debug, Clone)]
pub struct MeshVertexAttribute {
    /// The friendly name of the vertex attribute
    pub name: &'static str,

    /// The _unique_ id of the vertex attribute. This will also determine sort ordering
    /// when generating vertex buffers. Built-in / standard attributes will use "close to zero"
    /// indices. When in doubt, use a random / very large usize to avoid conflicts.
    pub id: MeshVertexAttributeId,

    /// The format of the vertex attribute.
    pub format: VertexFormat,
}
impl MeshVertexAttribute {
    pub const fn new(name: &'static str, id: usize, format: VertexFormat) -> Self {
        Self {
            name,
            id: MeshVertexAttributeId(id),
            format,
        }
    }
}

pub struct Mesh {
    /// `std::collections::BTreeMap` with all defined vertex attributes (Positions, Normals, ...)
    /// for this mesh. Attribute ids to attribute values.
    /// Uses a BTreeMap because, unlike HashMap, it has a defined iteration order,
    /// which allows easy stable VertexBuffers (i.e. same buffer order)
    attributes: BTreeMap<MeshVertexAttributeId, MeshAttributeData>,
}

impl Mesh {
    /// Where the vertex is located in space. Use in conjunction with [`Mesh::insert_attribute`]
    pub const ATTRIBUTE_POSITION: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x3);

    /// The direction the vertex normal is facing in.
    /// Use in conjunction with [`Mesh::insert_attribute`]
    pub const ATTRIBUTE_NORMAL: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Normal", 1, VertexFormat::Float32x3);

    /// Texture coordinates for the vertex. Use in conjunction with [`Mesh::insert_attribute`]
    pub const ATTRIBUTE_UV_0: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Uv", 2, VertexFormat::Float32x2);

    /// The direction of the vertex tangent. Used for normal mapping
    pub const ATTRIBUTE_TANGENT: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Tangent", 3, VertexFormat::Float32x4);

    /// Per vertex coloring. Use in conjunction with [`Mesh::insert_attribute`]
    pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Color", 4, VertexFormat::Float32x4);

    /// Per vertex joint transform matrix weight. Use in conjunction with [`Mesh::insert_attribute`]
    pub const ATTRIBUTE_JOINT_WEIGHT: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_JointWeight", 5, VertexFormat::Float32x4);

    pub fn new() -> Self {
        Mesh {
            attributes: Default::default(),
        }
    }

    #[inline]
    pub fn insert_attribute(
        &mut self,
        attribute: MeshVertexAttribute,
        values: impl Into<VertexAttributeValues>,
    ) {
        let mut values = values.into();
        let values_format = VertexFormat::from(&values);
        if values_format != attribute.format {
            panic!(
                    "Failed to insert attribute. Invalid attribute format for {}. Given format is {values_format:?} but expected {:?}",
                    attribute.name, attribute.format
                );
        }

        // validate attributes
        if attribute.id == Self::ATTRIBUTE_JOINT_WEIGHT.id {
            let VertexAttributeValues::Float32x4(ref mut values) = values else {
                    unreachable!() // we confirmed the format above
                };
            for value in values.iter_mut().filter(|v| *v == &[0.0, 0.0, 0.0, 0.0]) {
                // zero weights are invalid
                value[0] = 1.0;
            }
        }

        self.attributes
            .insert(attribute.id, MeshAttributeData { attribute, values });
    }

    /// Counts all vertices of the mesh.
    ///
    /// # Panics
    /// Panics if the attributes have different vertex counts.
    pub fn count_vertices(&self) -> usize {
        let mut vertex_count: Option<usize> = None;
        for (attribute_id, attribute_data) in &self.attributes {
            let attribute_len = attribute_data.values.len();
            if let Some(previous_vertex_count) = vertex_count {
                assert_eq!(previous_vertex_count, attribute_len,
                      "{attribute_id:?} has a different vertex count ({attribute_len}) than other attributes ({previous_vertex_count}) in this mesh.");
            }
            vertex_count = Some(attribute_len);
        }

        vertex_count.unwrap_or(0)
    }

    /// Computes and returns the vertex data of the mesh as bytes.
    /// Therefore the attributes are located in the order of their [`MeshVertexAttribute::id`].
    /// This is used to transform the vertex data into a GPU friendly format.
    ///
    /// # Panics
    /// Panics if the attributes have different vertex counts.
    pub fn get_vertex_buffer_data(&self) -> Vec<u8> {
        let vertex_size: usize = self
            .attributes
            .values()
            .fold(0, |acc, v| acc + v.attribute.format.size());

        let vertex_count = self.count_vertices();
        let mut attributes_interleaved_buffer = vec![0; vertex_count * vertex_size];
        // bundle into interleaved buffers
        let mut attribute_offset = 0;
        for attribute_data in self.attributes.values() {
            let attribute_size = attribute_data.attribute.format.size();
            let attributes_bytes = attribute_data.values.get_bytes();
            for (vertex_index, attribute_bytes) in
                attributes_bytes.chunks_exact(attribute_size).enumerate()
            {
                let offset = vertex_index * vertex_size + attribute_offset;
                attributes_interleaved_buffer[offset..offset + attribute_size]
                    .copy_from_slice(attribute_bytes);
            }

            attribute_offset += attribute_size;
        }

        attributes_interleaved_buffer
    }

    pub fn get_vertex_buffer_layout(&self) -> Vec<VertexFormat> {
        self.attributes.values().fold(Vec::new(), |mut acc, v| {
            acc.push(v.attribute.format);
            acc
        })
    }
}
/// Contains an array where each entry describes a property of a single vertex.
/// Matches the [`VertexFormats`](VertexFormat).
#[derive(Clone, Debug)]
pub enum VertexAttributeValues {
    Float32(Vec<f32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
}
impl VertexAttributeValues {
    pub fn len(&self) -> usize {
        match self {
            VertexAttributeValues::Uint32(v) => v.len(),
            VertexAttributeValues::Float32(v) => v.len(),
            VertexAttributeValues::Float32x2(v) => v.len(),
            VertexAttributeValues::Float32x3(v) => v.len(),
            VertexAttributeValues::Float32x4(v) => v.len(),
        }
    }
    pub fn get_bytes(&self) -> &[u8] {
        match self {
            VertexAttributeValues::Uint32(v) => cast_slice(v),
            VertexAttributeValues::Float32(v) => cast_slice(v),
            VertexAttributeValues::Float32x2(v) => cast_slice(v),
            VertexAttributeValues::Float32x3(v) => cast_slice(v),
            VertexAttributeValues::Float32x4(v) => cast_slice(v),
        }
    }
}

impl From<&VertexAttributeValues> for VertexFormat {
    fn from(values: &VertexAttributeValues) -> Self {
        match values {
            VertexAttributeValues::Uint32(_) => VertexFormat::Uint32,
            VertexAttributeValues::Float32(_) => VertexFormat::Float32,
            VertexAttributeValues::Float32x2(_) => VertexFormat::Float32x2,
            VertexAttributeValues::Float32x3(_) => VertexFormat::Float32x3,
            VertexAttributeValues::Float32x4(_) => VertexFormat::Float32x4,
        }
    }
}
