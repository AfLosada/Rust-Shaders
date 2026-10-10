#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::single_match)]

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
    GL_LINK_STATUS, GL_STATIC_DRAW, GL_TRIANGLES, GL_VERTEX_SHADER,
};

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
    unsafe { gl_fns.ClearColor(0.3, 0.4, 0.4, 1.0) };

    unsafe {
        let mut vao = 0;
        gl_fns.GenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);
        gl_fns.BindVertexArray(vao);
    }

    unsafe {
        let mut vbo = 0;
        gl_fns.GenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);

        gl_fns.BindBuffer(GL_ARRAY_BUFFER, vbo);
    }
    type Vertex = [f32; 3];
    const VERTICES: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];
    unsafe {
        gl_fns.BufferData(
            GL_ARRAY_BUFFER,
            size_of_val(&VERTICES) as isize,
            VERTICES.as_ptr().cast(),
            GL_STATIC_DRAW,
        );
    }

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

    unsafe {
        gl_fns.ShaderSource(
            vertex_shader,
            1,
            &(VERT_SHADER.as_bytes().as_ptr().cast()),
            &(VERT_SHADER.len().try_into().unwrap()),
        );
        gl_fns.CompileShader(vertex_shader);
        let mut success = 0;
        gl_fns.GetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl_fns.GetShaderInfoLog(vertex_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }
    }

    let fragment_shader = gl_fns.CreateShader(GL_FRAGMENT_SHADER);
    assert_ne!(fragment_shader, 0);

    unsafe {
        gl_fns.ShaderSource(
            fragment_shader,
            1,
            &(FRAG_SHADER.as_bytes().as_ptr().cast()),
            &(FRAG_SHADER.len().try_into().unwrap()),
        );
        gl_fns.CompileShader(fragment_shader);
        let mut success = 0;
        gl_fns.GetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl_fns.GetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }
    }

    let shader_program = gl_fns.CreateProgram();
    gl_fns.AttachShader(shader_program, vertex_shader);
    gl_fns.AttachShader(shader_program, fragment_shader);
    gl_fns.LinkProgram(shader_program);

    unsafe {
        let mut success = 0;
        gl_fns.GetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl_fns.GetProgramInfoLog(shader_program, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
        }
    }

    gl_fns.DeleteShader(vertex_shader);
    gl_fns.DeleteShader(fragment_shader);

    gl_fns.UseProgram(shader_program);

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
