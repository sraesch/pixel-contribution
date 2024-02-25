use std::error::Error;

pub use winit::event::MouseButton;
pub use winit::keyboard::{Key, NamedKey};

/// The trait for a handling events during rendering.
pub trait EventHandler {
    /// Callback for initializing the OpenGL setup. This is called once before the first frame.
    /// Returns an error message if the setup failed.
    ///
    /// # Arguments
    ///* `w` - The width of the rendering buffer
    ///* `h` - The height of the rendering buffer
    fn setup(&mut self, width: u32, height: u32) -> Result<(), Box<dyn Error>>;

    /// Callback if the event loop quit
    fn stop(&mut self);

    /// Render the next frame
    fn next_frame(&mut self);

    /// Resizing the rendering buffer
    ///
    /// # Arguments
    ///
    ///* `w` - The width of the rendering buffer
    ///* `h` - The height of the rendering buffer
    fn resize(&mut self, w: u32, h: u32);

    /// Callback for logical cursor position
    ///
    ///* `x` - The x coordinate of the cursor in logical coordinates
    ///* `y` - The y coordinate of the cursor in logical coordinates
    fn cursor_move(&mut self, x: f64, y: f64);

    /// Callback for mouse button event.
    ///
    /// # Arguments
    ///
    ///* `x` - The x coordinate of the cursor in logical coordinates
    ///* `y` - The y coordinate of the cursor in logical coordinates
    ///* `button` - The pressed/released mouse button
    ///* `pressed` - If true the mouse button was pressed and released otherwise.
    fn mouse_button(&mut self, x: f64, y: f64, button: MouseButton, pressed: bool);

    /// Is called when a key is either pressed or released.
    ///
    /// # Arguments
    ///
    /// * `key` - The key pressed or released.
    /// * `pressed` - Determines if the key was pressed or released.
    fn keyboard_event(&mut self, key: Key, pressed: bool);
}
