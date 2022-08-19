pub mod renderer;
pub use renderer::{
    ClearFrame, CommandEncoder, Commands, CreateFrame, Descriptor, Frame, PresentFrame, Renderer,
    ResizeSurface, SubmitCommands, SurfaceSize,
};
//pub use tasks::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
