use dotrix::egui::{Egui, TopBottomPanel};
use dotrix::overlay::Overlay;
use dotrix::{
    camera,
    ecs::{Const, Mut},
    egui, overlay, Camera, Dotrix, System, Transform, World,
};
use dotrix_voxel::{Grid, Light, TexSdf, VoxelJumpFlood};
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

fn startup(mut camera: Mut<Camera>, mut world: Mut<World>) {
    camera.target = [0., 0., 0.].into();
    camera.distance = 30.0;
    camera.tilt = 0.0;

    let mut grid = Grid::default();

    randomize_grid(&mut grid);
    world.spawn(vec![(
        grid,
        VoxelJumpFlood::default(),
        TexSdf::default(),
        Transform::builder()
            .with_translate([2., 2., 2.].into())
            .with_scale([2., 2., 2.].into())
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
        if ui.button("Randomize").clicked() {
            for (grid,) in world.query::<(&mut Grid,)>() {
                randomize_grid(grid);
            }
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
}
