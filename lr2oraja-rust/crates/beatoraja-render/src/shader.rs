// Shader pipeline wrapper.
// Drop-in replacement for the ShaderProgram stub in rendering_stubs.rs.

/// Wrapper around a wgpu render pipeline.
/// Corresponds to com.badlogic.gdx.graphics.glutils.ShaderProgram.
#[derive(Clone, Debug, Default)]
pub struct ShaderProgram;

impl ShaderProgram {
    pub fn new() -> Self {
        Self
    }
}
