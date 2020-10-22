use dotrix::{Dotrix, window::Window};

fn main() {
    /*
    // look into crates\dotrix_core\src\input\input.rs

    // Example of use:

    let jump = input.get_button_down(InputAction::Jump);
    let shoot = input.get_button(InputAction::Jump);
    let zoom = input.get_scroll();

    if jump {
        println!("character jumped");
    }
    if shoot {
        println!("shooting every frame")
    }
    if zoom > 0.0{
        println!("zoom-in");
    }
    if zoom < 0.0 {
        println!("zoom-out");
    }
    */

    Dotrix::init()
        .window(Window {})
        .run();
}
