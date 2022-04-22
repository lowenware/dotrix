use dotrix::egui::{DragValue, Egui, TopBottomPanel};
use dotrix::overlay::Overlay;
use dotrix::{
    assets::Texture,
    camera,
    ecs::{Const, Mut},
    egui, overlay, Assets, Camera, Dotrix, System, Transform, World,
};
use dotrix_pbr::Material;
use dotrix_voxel::{Grid, Light, MaterialSet, TexSdf, VoxelJumpFlood};
use rand::Rng;

fn main() {
    Dotrix::application("Dotrix: Voxel SDF")
        .with(System::from(startup))
        .with(System::from(camera::control))
        .with(System::from(self::ui))
        .with(overlay::extension)
        .with(egui::extension)
        .with(dotrix_voxel::extension)
        .run();
}

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>, mut assets: Mut<Assets>) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 30.0;
    camera.tilt = 0.0;

    let mut grid = Grid::default();
    randomize_grid(&mut grid);

    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_AmbientOcclusion.jpg");
    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_Color.jpg");
    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_NormalDX.jpg");
    assets.import("assets/textures/mossy_bricks/Bricks076C_1K_Roughness.jpg");

    let ao = assets.register::<Texture>("Bricks076C_1K_AmbientOcclusion");
    let albedo = assets.register::<Texture>("Bricks076C_1K_Color");
    let normal = assets.register::<Texture>("Bricks076C_1K_NormalDX");
    let roughness = assets.register::<Texture>("Bricks076C_1K_Roughness");
    let mut material_set = MaterialSet::default();
    material_set.set_material(
        0,
        Material {
            texture: albedo,
            albedo: [0.5, 0.5, 0.5].into(),
            roughness_texture: roughness,
            ao_texture: ao,
            normal_texture: normal,
            metallic: 0.0,
            ..Default::default()
        },
    );

    assets.import("assets/textures/PaintedPlaster010/PaintedPlaster010_1K_AmbientOcclusion.png");
    assets.import("assets/textures/PaintedPlaster010/PaintedPlaster010_1K_Color.png");
    assets.import("assets/textures/PaintedPlaster010/PaintedPlaster010_1K_NormalDX.png");
    assets.import("assets/textures/PaintedPlaster010/PaintedPlaster010_1K_Roughness.png");

    let ao = assets.register::<Texture>("PaintedPlaster010_1K_AmbientOcclusion");
    let albedo = assets.register::<Texture>("PaintedPlaster010_1K_Color");
    let normal = assets.register::<Texture>("PaintedPlaster010_1K_NormalDX");
    let roughness = assets.register::<Texture>("PaintedPlaster010_1K_Roughness");
    material_set.set_material(
        1,
        Material {
            texture: albedo,
            albedo: [0.5, 0.5, 0.5].into(),
            roughness_texture: roughness,
            ao_texture: ao,
            normal_texture: normal,
            metallic: 0.0,
            ..Default::default()
        },
    );

    assets.import("assets/textures/Bricks075B/Bricks075B_1K_AmbientOcclusion.jpg");
    assets.import("assets/textures/Bricks075B/Bricks075B_1K_Color.jpg");
    assets.import("assets/textures/Bricks075B/Bricks075B_1K_NormalDX.jpg");
    assets.import("assets/textures/Bricks075B/Bricks075B_1K_Roughness.jpg");

    let ao = assets.register::<Texture>("Bricks075B_1K_AmbientOcclusion");
    let albedo = assets.register::<Texture>("Bricks075B_1K_Color");
    let normal = assets.register::<Texture>("Bricks075B_1K_NormalDX");
    let roughness = assets.register::<Texture>("Bricks075B_1K_Roughness");
    material_set.set_material(
        2,
        Material {
            texture: albedo,
            albedo: [0.5, 0.5, 0.5].into(),
            roughness_texture: roughness,
            ao_texture: ao,
            normal_texture: normal,
            metallic: 0.0,
            ..Default::default()
        },
    );

    assets.import("assets/textures/PavingStones113/PavingStones113_1K_AmbientOcclusion.jpg");
    assets.import("assets/textures/PavingStones113/PavingStones113_1K_Color.jpg");
    assets.import("assets/textures/PavingStones113/PavingStones113_1K_NormalDX.jpg");
    assets.import("assets/textures/PavingStones113/PavingStones113_1K_Roughness.jpg");

    let ao = assets.register::<Texture>("PavingStones113_1K_AmbientOcclusion");
    let albedo = assets.register::<Texture>("PavingStones113_1K_Color");
    let normal = assets.register::<Texture>("PavingStones113_1K_NormalDX");
    let roughness = assets.register::<Texture>("PavingStones113_1K_Roughness");
    material_set.set_material(
        3,
        Material {
            texture: albedo,
            albedo: [0.5, 0.5, 0.5].into(),
            roughness_texture: roughness,
            ao_texture: ao,
            normal_texture: normal,
            metallic: 0.0,
            ..Default::default()
        },
    );

    world.spawn(vec![(
        grid,
        material_set,
        // Instruct it to use the JumpFlood algorithm to convert the Voxel to an SDF
        VoxelJumpFlood::default(),
        // Render as a 3D texture based SDF
        TexSdf::default(),
        // Transform the voxel where you like
        Transform::builder()
            // .with_translate([2.,2.,2.].into())
            .with_scale([1., 3., 1.].into())
            .build(),
    )]);

    world.spawn(Some((Light::Ambient {
        color: [0., 0., 0.].into(),
        intensity: 0.,
    },)));
    world.spawn(Some((Light::Directional {
        color: [1., 1., 1.].into(),
        direction: [100., -100., -100.].into(),
        intensity: 1.,
        enabled: true,
    },)));
}

pub fn ui(overlay: Mut<Overlay>, world: Const<World>) {
    let egui = overlay
        .get::<Egui>()
        .expect("Renderer does not contain an Overlay instance");
    TopBottomPanel::bottom("my_panel").show(&egui.ctx, |ui| {
        for (grid, transform) in world.query::<(&mut Grid, &mut Transform)>() {
            if ui.button("Randomize").clicked() {
                randomize_grid(grid);
            }
            ui.add(
                DragValue::new(&mut transform.scale[0])
                    .speed(0.1)
                    .prefix("X:"),
            );
            ui.add(
                DragValue::new(&mut transform.scale[1])
                    .speed(0.1)
                    .prefix("Y:"),
            );
            ui.add(
                DragValue::new(&mut transform.scale[2])
                    .speed(0.1)
                    .prefix("Z:"),
            );
        }
    });
}

fn randomize_grid(grid: &mut Grid) {
    let dims = grid.get_dimensions();
    let total_size: usize = dims.iter().fold(1usize, |acc, &item| acc * (item as usize));
    let values: Vec<u8> = vec![0u8; total_size]
        .iter()
        .map(|_v| {
            let chance: u8 = rand::thread_rng().gen();
            if chance > 128 {
                1
            } else {
                0
            }
        })
        .collect();

    grid.set_values(values);

    let material_values: Vec<u8> = vec![0u8; total_size]
        .iter()
        .map(|_v| {
            let chance: u8 = rand::thread_rng().gen();
            if chance > 192 {
                3
            } else if chance > 128 {
                2
            } else if chance > 64 {
                1
            } else {
                0
            }
        })
        .collect();
    grid.set_materials(material_values);
}
