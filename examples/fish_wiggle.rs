#![feature(is_sorted)]
#![feature(extract_if)]
#![feature(offset_of)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

extern crate glfw;

use glam::{vec3, Mat4};
use glfw::{Action, Context, Key};
use log::error;
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::gl;
use small_gl_core::mesh::{Color, Mesh};
use small_gl_core::model::ModelBuilder;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use std::rc::Rc;

const SCR_WIDTH: f32 = 1000.0;
const SCR_HEIGHT: f32 = 800.0;

struct State {
    camera: Camera,
    run: bool,
    delta_time: f32,
    frame_time: f32,
    firstMouse: bool,
    lastX: f32,
    lastY: f32,
}

fn error_callback(err: glfw::Error, description: String) {
    error!("GLFW error {:?}: {:?}", err, description);
}

fn main() {
    let mut glfw = glfw::init(error_callback).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    // for Apple
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(SCR_WIDTH as u32, SCR_HEIGHT as u32, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Turn on all GLFW polling so that we can receive all WindowEvents
    window.set_all_polling(true);
    window.make_current();

    // Initialize glad: load all OpenGL function pointers
    // --------------------------------------------------
    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    // perspective setting
    // //let camera = Camera::camera_vec3(vec3(300.0, 300.0, 500.0));
    let camera = Camera::camera_vec3_up_yaw_pitch(
        // vec3(400.0, 400.0, 700.0), for current x,y world
        vec3(0.0, 50.0, 100.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    // let camera = Camera::camera_vec3_up_yaw_pitch(
    //     vec3(400.0, -200.0, 50.0),
    //     vec3(0.0, 1.0, 0.0),
    //     -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
    //     90.0);

    // for ortho perspective
    // let camera = Camera::camera_vec3(vec3(0.0, 0.0, 55.0));

    // Initialize the world state
    let mut state = State {
        camera,
        run: true,
        delta_time: 0.0,
        frame_time: 0.0,
        firstMouse: true,
        lastX: SCR_WIDTH / 2.0,
        lastY: SCR_HEIGHT / 2.0,
    };

    let shader_texture = Shader::new("assets/shaders/camera_texture.vert", "assets/shaders/camera_texture.frag").unwrap();

    let wavy_shader = Shader::new("assets/shaders/wavy_texture.vert", "assets/shaders/wavy_texture.frag").unwrap();

    let model_shader = Shader::new("assets/shaders/basic_model.vert", "assets/shaders/basic_model.frag").unwrap();
    let wiggle_shader = Shader::new("assets/shaders/wiggle_shader.vert", "assets/shaders/wiggle_shader.frag").unwrap();

    let model_shader = Rc::new(model_shader);

    let big_fish = "assets/models/BarramundiFish/glTF/BarramundiFish.gltf";
    let fish_model = ModelBuilder::new("big_fish", big_fish).build().unwrap();

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // render loop
    while !window.should_close() {
        let current_time = glfw.get_time() as f32;
        if state.run {
            state.delta_time = current_time - state.frame_time;
        } else {
            state.delta_time = 0.0;
        }
        state.frame_time = current_time;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        unsafe {
            gl::ClearColor(0.2, 0.50, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let view = state.camera.get_view_matrix();
        // let view = Mat4::look_at_rh(state.camera.position, vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
        let projection = Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), SCR_WIDTH / SCR_HEIGHT, 0.1, 2000.0);
        // let projection = Mat4::orthographic_rh_gl(0.0, 600.0, 0.0, 600.0, 0.1, 100.0);
        // let projection = Mat4::orthographic_rh_gl(0.0, 1000.0, 0.0, 1000.0, 0.0, 1000.0);

        // fish
        wiggle_shader.use_shader_with(&projection, &view);
        wiggle_shader.set_vec3("nosePos", &vec3(0.0, 0.0, -0.3));
        wiggle_shader.set_float("time", state.frame_time);

        let mut model = Mat4::from_translation(vec3(0.0, 0.0, 0.0));
        model *= Mat4::from_scale(vec3(20.0, 20.0, 20.0));

        wiggle_shader.set_mat4("model", &model);

        fish_model.render(&wiggle_shader);

        window.swap_buffers();
    }
}

//
// GLFW maps callbacks to events.
//
fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, state: &mut State) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width, height) => {
            framebuffer_size_event(window, width, height);
        }
        glfw::WindowEvent::Key(Key::W, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Forward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Backward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Left, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Right, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Up, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state.camera.process_keyboard(CameraMovement::Down, state.delta_time);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        _evt => {
            // println!("WindowEvent: {:?}", _evt);
        }
    }
}

// glfw: whenever the window size changed (by OS or user resize) this event fires.
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_event(_window: &mut glfw::Window, width: i32, height: i32) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    // println!("Framebuffer size: {}, {}", width, height);
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

fn mouse_handler(state: &mut State, xposIn: f64, yposIn: f64) {
    let xpos = xposIn as f32;
    let ypos = yposIn as f32;

    if state.firstMouse {
        state.lastX = xpos;
        state.lastY = ypos;
        state.firstMouse = false;
    }

    let xoffset = xpos - state.lastX;
    let yoffset = state.lastY - ypos; // reversed since y-coordinates go from bottom to top

    state.lastX = xpos;
    state.lastY = ypos;

    state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.camera.process_mouse_scroll(yoffset as f32);
}
