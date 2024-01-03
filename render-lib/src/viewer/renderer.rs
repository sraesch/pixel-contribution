use winit::keyboard::KeyCode;

pub type Key = KeyCode;

/// The trait for a renderer.
pub trait Renderer {
    /// Render the next frame
    fn next_frame(&mut self);

    /// Resizing the rendering buffer
    ///
    ///* `w` - The width of the rendering buffer
    ///* `h` - The height of the rendering buffer
    fn resize(&mut self, w: u32, h: u32);

    /// Update the internal rendering view
    ///
    /// # Arguments
    /// * `width` - The width of the view.
    /// * `height` - The height of the view.
    fn update_view(&mut self, width: u32, height: u32);

    /// Is called when a key is either pressed or released.
    ///
    /// # Arguments
    ///
    /// * `key` - The key pressed or released.
    /// * `pressed` - Determines if the key was pressed or released.
    fn keyboard_event(&mut self, key: Key, pressed: bool);
}
