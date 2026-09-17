pub enum PositionConstraint {
    Pixel(f32),
    Percent(f32),
    Center,
}

pub enum SizeConstraint {
    Pixel(f32),
    Percent(f32),
}

pub struct View {
    pub x: PositionConstraint,
    pub y: PositionConstraint,
    pub width: SizeConstraint,
    pub height: SizeConstraint,
}
