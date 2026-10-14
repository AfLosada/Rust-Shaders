#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::single_match)]

pub mod structs;

const WINDOW_TITLE: &str = "Hello Window";

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;

  void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 final_color;

  void main() {
    final_color = vec4(1.0, 0.5, 0.2, 1.0);
  }
"#;

use beryllium::{
    events::Event,
    init::InitFlags,
    video::{CreateWinArgs, GlContextFlags, GlProfile},
    *,
};

use gl33::{
    self, GL_ARRAY_BUFFER, GL_COLOR_BUFFER_BIT, GL_COMPILE_STATUS, GL_FLOAT, GL_FRAGMENT_SHADER,
    GL_LINK_STATUS, GL_STATIC_DRAW, GL_TRIANGLES, GL_VERTEX_SHADER, GlFns,
};

use crate::structs::{Buffer, BufferType, ShaderProgram, VertexArray};

pub fn clear_color(gl_fn: &GlFns, r: f32, g: f32, b: f32, a: f32) {
    unsafe { gl_fn.ClearColor(r, g, b, a) }
}

fn main() {
    let sdl = Sdl::init(InitFlags::EVERYTHING);
    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_context_minor_version(3).unwrap();
    sdl.set_gl_profile(GlProfile::Core).unwrap();
    let flags = GlContextFlags::default();
    sdl.set_gl_context_flags(flags).unwrap();

    let _win = sdl
        .create_gl_window(CreateWinArgs {
            title: WINDOW_TITLE,
            width: 800,
            height: 600,
            ..Default::default()
        })
        .expect("couldn't make a window and context");
    let _ = _win
        .set_swap_interval(video::GlSwapInterval::Vsync)
        .unwrap();

    let gl_fns = unsafe { gl33::GlFns::load_from(&|f| _win.get_proc_address(f)).unwrap() };

    clear_color(&gl_fns, 0.3, 0.4, 0.4, 1.0);

    let vao = VertexArray::new(&gl_fns).unwrap();
    let vbo = Buffer::new(&gl_fns).unwrap();
    vao.bind();
    vbo.bind(structs::BufferType::Array);

    draw_rectangle(&gl_fns);

    'main_loop: loop {
        // handle events this frame
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                Event::Quit => break 'main_loop,
                _ => (),
            }
        }
        // now the events are clear.

        // here's where we could change the world state and draw.
        unsafe {
            gl_fns.Clear(GL_COLOR_BUFFER_BIT);
            gl_fns.DrawArrays(GL_TRIANGLES, 0, 3);
        }
        _win.swap_window();
    }
}

fn draw_rectangle(gl_fns: &GlFns) {
    type Vertex = [f32; 3];
    type TriIndexes = [u32; 3];

    const VERTICES: [Vertex; 4] = [
        [0.5, 0.5, 0.0],
        [0.5, -0.5, 0.0],
        [-0.5, -0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];

    const INDICES: [TriIndexes; 2] = [[0, 1, 3], [1, 2, 3]];

    let vertices_buffer = Buffer::new(&gl_fns).unwrap();
    vertices_buffer.bind(structs::BufferType::ElementArray);
    let ebo = Buffer::new(gl_fns).expect("Couldn't make the element buffer.");
    ebo.bind(BufferType::ElementArray);
    ebo.buffer_data(
        BufferType::ElementArray,
        bytemuck::cast_slice(&INDICES),
        GL_STATIC_DRAW,
    );
}

fn draw_triangle(gl_fns: &GlFns) {
    type Vertex = [f32; 3];
    const VERTICES: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

    let vertices_buffer = Buffer::new(&gl_fns).unwrap();
    vertices_buffer.bind(structs::BufferType::ElementArray);

    vertices_buffer.buffer_data(
        BufferType::Array,
        bytemuck::cast_slice(&VERTICES),
        GL_STATIC_DRAW,
    );

    unsafe {
        gl_fns.VertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            0,
            size_of::<Vertex>().try_into().unwrap(),
            0 as *const _,
        );
        gl_fns.EnableVertexAttribArray(0);
    }

    let vertex_shader = gl_fns.CreateShader(GL_VERTEX_SHADER);
    assert_ne!(vertex_shader, 0);

    let shader_program = ShaderProgram::from_vert_frag(&gl_fns, VERT_SHADER, FRAG_SHADER).unwrap();
    shader_program.use_program();
}
