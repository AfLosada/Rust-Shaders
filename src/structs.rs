use beryllium::{
    events::Event,
    init::InitFlags,
    video::{CreateWinArgs, GlContextFlags, GlProfile},
    *,
};

use gl33::{
    self, GL_ARRAY_BUFFER, GL_COMPILE_STATUS, GL_ELEMENT_ARRAY_BUFFER, GL_FRAGMENT_SHADER,
    GL_INFO_LOG_LENGTH, GL_LINK_STATUS, GL_TRUE, GL_VERTEX_SHADER, GlFns,
};

use gl33::gl_core_types::*;

/// Basic wrapper for a [Vertex Array
/// Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
pub struct VertexArray<'a>(&'a GlFns, pub u32);

impl<'gl_lifetime> VertexArray<'gl_lifetime> {
    /// Creates a new vertex array object
    pub fn new(gl_fns: &'gl_lifetime GlFns) -> Option<Self> {
        let mut vao = 0;
        unsafe { gl_fns.GenVertexArrays(1, &mut vao) };
        if vao != 0 {
            Some(Self(gl_fns, vao))
        } else {
            None
        }
    }

    /// Bind this vertex array as the current vertex array object
    pub fn bind(&self) {
        let _ = &self.0.BindVertexArray(self.1);
    }

    /// Clear the current vertex array object binding.
    pub fn clear_binding(&self) {
        let _ = &self.0.BindVertexArray(0);
    }
}

/// The types of buffer object that you can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Array Buffers holds arrays of vertex data for drawing.
    Array = GL_ARRAY_BUFFER.0 as isize,
    /// Element Array Buffers hold indexes of what vertexes to use for drawing.
    ElementArray = GL_ELEMENT_ARRAY_BUFFER.0 as isize,
}

/// Basic wrapper for a [Buffer
/// Object](https://www.khronos.org/opengl/wiki/Buffer_Object).
pub struct Buffer<'gl_lifetime>(&'gl_lifetime GlFns, pub u32);
impl<'gl_lifetime> Buffer<'gl_lifetime> {
    /// Makes a new vertex buffer
    pub fn new(gl_fns: &'gl_lifetime GlFns) -> Option<Self> {
        let mut vao = 0;
        unsafe { gl_fns.GenBuffers(1, &mut vao) };
        if vao != 0 {
            Some(Self(gl_fns, vao))
        } else {
            None
        }
    }

    /// Bind this vertex buffer for the given type
    pub fn bind(&self, ty: BufferType) {
        unsafe {
            let _ = &self.0.BindBuffer(GLenum(ty as u32), self.1);
        }
    }

    /// Clear the current vertex buffer binding for the given type.
    pub fn clear_binding(&self, ty: BufferType) {
        unsafe {
            let _ = &self.0.BindBuffer(GLenum(ty as u32), 0);
        }
    }

    /// Places a slice of data into a previously-bound buffer.
    pub fn buffer_data(&self, ty: BufferType, data: &[u8], usage: GLenum) {
        unsafe {
            let _ = &self.0.BufferData(
                GLenum(ty as u32),
                data.len().try_into().unwrap(),
                data.as_ptr().cast(),
                usage,
            );
        }
    }
}

/// The types of shader object.
pub enum ShaderType {
    /// Vertex shaders determine the position of geometry within the screen.
    Vertex = GL_VERTEX_SHADER.0 as isize,
    /// Fragment shaders determine the color output of geometry.
    ///
    /// Also other values, but mostly color.
    Fragment = GL_FRAGMENT_SHADER.0 as isize,
}

/// A handle to a [Shader
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Shader_objects)
pub struct Shader<'gl_lifetime>(&'gl_lifetime GlFns, pub u32);
impl<'gl_lifetime> Shader<'gl_lifetime> {
    /// Makes a new shader.
    ///
    /// Prefer the [`Shader::from_source`](Shader::from_source) method.
    ///
    /// Possibly skip the direct creation of the shader object and use
    /// [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag).
    pub fn new(gl_fns: &'gl_lifetime GlFns, ty: ShaderType) -> Option<Self> {
        let shader = gl_fns.CreateShader(GLenum(ty as u32));
        if shader != 0 {
            Some(Self(gl_fns, shader))
        } else {
            None
        }
    }

    /// Assigns a source string to the shader.
    ///
    /// Replaces any previously assigned source.
    pub fn set_source(&self, src: &str) {
        unsafe {
            self.0.ShaderSource(
                self.1,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    /// Compiles the shader based on the current source.
    pub fn compile(&self) {
        self.0.CompileShader(self.1);
    }

    /// Checks if the last compile was successful or not.
    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe {
            self.0.GetShaderiv(self.1, GL_COMPILE_STATUS, &mut compiled);
        }
        compiled == GL_TRUE.0 as i32
    }

    /// Gets the info log for the shader.
    ///
    /// Usually you use this to get the compilation log when a compile failed.
    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe {
            self.0
                .GetShaderiv(self.1, GL_INFO_LOG_LENGTH, &mut needed_len);
        }
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            self.0.GetShaderInfoLog(
                self.1,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Marks a shader for deletion.
    ///
    /// Note: This _does not_ immediately delete the shader. It only marks it for
    /// deletion. If the shader has been previously attached to a program then the
    /// shader will stay allocated until it's unattached from that program.
    pub fn delete(self) {
        self.0.DeleteShader(self.1);
    }

    /// Takes a shader type and source string and produces either the compiled
    /// shader or an error message.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    pub fn from_source(
        gl_fns: &'gl_lifetime GlFns,
        ty: ShaderType,
        source: &str,
    ) -> Result<Self, String> {
        let id = Self::new(gl_fns, ty).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        id.set_source(source);
        id.compile();
        if id.compile_success() {
            Ok(id)
        } else {
            let out = id.info_log();
            id.delete();
            Err(out)
        }
    }
}

/// A handle to a [Program
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects)
pub struct ShaderProgram<'gl_lifetime>(pub &'gl_lifetime GlFns, pub u32);
impl<'gl_lifetime> ShaderProgram<'gl_lifetime> {
    /// Allocates a new program object.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    pub fn new(gl_fns: &'gl_lifetime GlFns) -> Option<Self> {
        let prog = gl_fns.CreateProgram();
        if prog != 0 {
            Some(Self(gl_fns, prog))
        } else {
            None
        }
    }

    /// Attaches a shader object to this program object.
    pub fn attach_shader(&self, shader: &Shader) {
        &self.0.AttachShader(self.1, shader.1);
    }

    /// Links the various attached, compiled shader objects into a usable program.
    pub fn link_program(&self) {
        self.0.LinkProgram(self.1)
    }

    /// Checks if the last linking operation was successful.
    pub fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { self.0.GetProgramiv(self.1, GL_LINK_STATUS, &mut success) };
        success == GL_TRUE.0 as i32
    }

    /// Gets the log data for this program.
    ///
    /// This is usually used to check the message when a program failed to link.
    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe {
            self.0
                .GetProgramiv(self.1, GL_INFO_LOG_LENGTH, &mut needed_len);
        };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            self.0.GetProgramInfoLog(
                self.1,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Sets the program as the program to use when drawing.
    pub fn use_program(&self) {
        self.0.UseProgram(self.1);
    }

    /// Marks the program for deletion.
    ///
    /// Note: This _does not_ immediately delete the program. If the program is
    /// currently in use it won't be deleted until it's not the active program.
    /// When a program is finally deleted and attached shaders are unattached.
    pub fn delete(self) {
        self.0.DeleteProgram(self.1);
    }

    /// Takes a vertex shader source string and a fragment shader source string
    /// and either gets you a working program object or gets you an error message.
    ///
    /// This is the preferred way to create a simple shader program in the common
    /// case. It's just less error prone than doing all the steps yourself.
    pub fn from_vert_frag(
        gl_fns: &'gl_lifetime GlFns,
        vert: &str,
        frag: &str,
    ) -> Result<Self, String> {
        let p = Self::new(gl_fns).ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(gl_fns, ShaderType::Vertex, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(gl_fns, ShaderType::Fragment, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(&v);
        p.attach_shader(&f);
        p.link_program();
        v.delete();
        f.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }
}
