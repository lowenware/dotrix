use crate::MeshAttribute;
use dotrix_core::assets::Mesh;

/// Cube primitive
pub struct Cube {
    /// Mesh attributes
    pub mesh_attributes: Vec<MeshAttribute>,
    /// Cube size
    pub size: f32,
    /// Texture UVs
    pub uvs: Option<Vec<[f32; 2]>>,
}

impl Cube {
    /// Creates new mesh builder
    pub fn builder(size: f32) -> Self {
        Self {
            mesh_attributes: Vec::new(),
            size,
            uvs: None,
        }
    }

    /// Add vertices to mesh
    #[must_use]
    pub fn with_positions(mut self) -> Self {
        self.mesh_attributes.push(MeshAttribute::Positions);
        self
    }

    /// Add normals to mesh
    #[must_use]
    pub fn with_normals(mut self) -> Self {
        self.mesh_attributes.push(MeshAttribute::Normals);
        self
    }

    /// Add tangents bitangents to mesh
    #[must_use]
    pub fn with_tangents_bitangents(mut self) -> Self {
        self.mesh_attributes.push(MeshAttribute::TangentsBitangents);
        self
    }

    /// Add texture UVs
    #[must_use]
    pub fn with_uvs(mut self, uvs: Vec<[f32; 2]>) -> Self {
        self.mesh_attributes.push(MeshAttribute::UVs);
        self.uvs = Some(uvs);
        self
    }

    /// Returns cube vertices positions
    pub fn positions(size: f32) -> Vec<[f32; 3]> {
        let half_width = size / 2.0;
        vec![
            [-half_width, -half_width, -half_width],
            [half_width, -half_width, -half_width],
            [half_width, half_width, -half_width],
            [-half_width, half_width, -half_width],
            [-half_width, -half_width, half_width],
            [half_width, -half_width, half_width],
            [half_width, half_width, half_width],
            [-half_width, half_width, half_width],
        ]
    }

    /// Returns cube indices
    pub fn indices() -> Vec<u32> {
        vec![
            0, 2, 1, 0, 3, 2, // front
            1, 6, 5, 1, 2, 6, // right
            5, 7, 4, 5, 6, 7, // back
            4, 3, 0, 4, 7, 3, // left
            3, 6, 2, 3, 7, 6, // top
            4, 1, 5, 4, 0, 1, // bottom
        ]
    }

    /// Generates cube mesh
    pub fn mesh(&self) -> Mesh {
        let positions = Self::positions(self.size);
        let indices = Self::indices();
        let mut mesh = Mesh::default();

        for attr in self.mesh_attributes.iter() {
            match attr {
                MeshAttribute::Positions => {
                    mesh.with_vertices(&positions);
                    mesh.with_indices(&indices);
                }
                MeshAttribute::Normals => {
                    mesh.with_vertices(&Mesh::calculate_normals(&positions, Some(&indices)))
                }
                MeshAttribute::TangentsBitangents => {
                    let uvs = self
                        .uvs
                        .as_ref()
                        .expect("UVs required to construct Cube mesh");
                    let (tangents, bitangents) =
                        Mesh::calculate_tangents_bitangents(&positions, uvs, Some(&indices));
                    mesh.with_vertices(&tangents);
                    mesh.with_vertices(&bitangents);
                }
                MeshAttribute::UVs => {
                    let uvs = self
                        .uvs
                        .as_ref()
                        .expect("UVs required to construct Cube mesh");
                    mesh.with_vertices(uvs);
                }
            }
        }
        mesh
    }
}
