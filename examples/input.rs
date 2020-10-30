use dotrix::{Dotrix, window::Window};

fn main() {
    /*
    * Add new Input action
    * 1) Register action in 'crates\dotrix_core\src\input\manager.rs'
    * 2) (optional) Bind action to key 'crates\dotrix_core\src\input\config.rs'
    * Done
    *
    *
    // Example of use:
    let jump = input_manager.get_button_down(input::Action::Jump);
    let forward = input_manager.get_button(input::Action::MoveForward);
    let zoom = input_manager.get_scroll();
    if jump {
        println!("character jumped");
    }
    if forward {
        println!("moving forward");
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
