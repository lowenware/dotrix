use dotrix::assets::Mesh;

pub fn cube(width: f32) -> Mesh {
    let half_width = width / 2.;

    let verticies: Vec<[f32; 3]> = vec![
        [-half_width, -half_width, -half_width],
        [half_width, -half_width, -half_width],
        [half_width, half_width, -half_width],
        [-half_width, half_width, -half_width],
        [-half_width, -half_width, half_width],
        [half_width, -half_width, half_width],
        [half_width, half_width, half_width],
        [-half_width, half_width, half_width],
    ];

    let indicies: Vec<u32> = vec![
        0, 2, 1, 0, 3, 2, // front
        1, 6, 5, 1, 2, 6, // right
        5, 7, 4, 5, 6, 7, // back
        4, 3, 0, 4, 7, 3, // left
        3, 6, 2, 3, 7, 6, // top
        4, 1, 5, 4, 0, 1, // bottom
    ];

    let mut mesh = Mesh::default();

    mesh.with_vertices(&verticies);
    mesh.with_indices(&indicies);

    mesh
}
