
pub struct Camera {
    distance: f32,
    angle: f32,
    height: f32,
    view: cgmath::Matrix4<f32>,
}

impl Camera {
    pub fn new(distance: f32, angle: f32, height: f32) -> Self {
        Self {
            distance,
            angle,
            height,
            view: Self::matrix(cgmath::Point3::new(0.0, 0.0, 0.0), distance, angle, height),
        }
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn view(&self) -> &cgmath::Matrix4<f32> {
        &self.view
    }

    pub fn set(&mut self, target: cgmath::Point3<f32>, distance: f32, angle: f32, height: f32) {
        self.distance = distance;
        self.angle = angle;
        self.height = height;
        self.view = Self::matrix(target, distance, angle, height);
    }

    pub fn look_at(&mut self, target: cgmath::Point3<f32>) {
        self.view = Self::matrix(target, self.distance, self.angle, self.height);
    }

    fn matrix(
        target: cgmath::Point3<f32>,
        distance: f32,
        angle: f32,
        height: f32
    ) -> cgmath::Matrix4<f32> {
        let zx = (distance.powi(2)  - height.powi(2)).sqrt();
        let dx = zx * angle.cos();
        let dz = zx * angle.sin();
        cgmath::Matrix4::look_at(
            cgmath::Point3::new(target.x + dx, height, target.z + dz),
            target,
            cgmath::Vector3::unit_y(),
        )
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(5.0, 3.14 / 2.0, 3.0)
    }
}
