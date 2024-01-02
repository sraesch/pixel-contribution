/// Trait for binding an OpenGL resource
pub trait Bind {
    /// Returns the resource id
    fn get_id(&self) -> u32;

    /// Binds the OpenGL resource
    fn bind(&self);

    // Unbinds the OpenGL resource
    fn unbind(&self);
}

/// Trait for an OpenGL resource
pub trait Resource {
    /// Returns true if the resource is valid
    fn is_valid(&self) -> bool;

    /// Releases the resource
    fn release(&mut self);
}
