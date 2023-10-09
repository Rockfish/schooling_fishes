#![feature(is_sorted)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod base_entity;
mod camera;
mod cell_space_partition;
mod constants;
mod fish_main;
mod game_world;
mod inverted_aab_box_2d;
mod moving_entity;
mod param_loader;
mod path;
mod smoother;
mod steering_behavior;
mod utils;
mod vehicle;
mod wall_2d;
mod transformations;

extern crate glfw;

use camera::{Camera, CameraMovement};
use glad_gl::gl;
use glad_gl::gl::{GLsizeiptr, GLuint, GLvoid};
use glam::{vec3, Mat4};
use glfw::{Action, Context, Key};
// use learn_opengl_with_rust::model::{FlipV, Gamma, Model};
// use learn_opengl_with_rust::shader::Shader;
use log::error;
use rand::prelude::*;
use std::mem;

const SCR_WIDTH: f32 = 800.0;
const SCR_HEIGHT: f32 = 800.0;

struct State {
    camera: Camera,
    deltaTime: f32,
    lastFrame: f32,
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

    let camera = Camera::camera_vec3(vec3(0.0, 0.0, 55.0));

    // Initialize the world state
    let mut state = State {
        camera,
        deltaTime: 0.0,
        lastFrame: 0.0,
        firstMouse: true,
        lastX: SCR_WIDTH / 2.0,
        lastY: SCR_HEIGHT / 2.0,
    };

    unsafe {
        // configure global opengl state
        // -----------------------------
        gl::Enable(gl::DEPTH_TEST);
    }

    // generate a large list of semi-random model transformation matrices
    // ------------------------------------------------------------------
    let amount: i32 = 100000;
    let radius: f32 = 150.0;
    let offset: f32 = 25.0;
    let mut rng = rand::thread_rng();
    let mut modelMatrices: Vec<Mat4> = vec![];

    for i in 0..amount {
        // 1. translation: displace along circle with 'radius' in range [-offset, offset]
        let angle = (i as f32) / (amount as f32) * 360.0;
        let displacement: f32 = rng.gen::<f32>() * (2.0 * offset * 100.0) / 100.0 - offset;
        let x = angle.sin() * radius + displacement;
        let displacement: f32 = rng.gen::<f32>() * (2.0 * offset * 100.0) / 100.0 - offset;
        let y = displacement * 4.0; // keep height of asteroid field smaller compared to width of x and z
        let displacement: f32 = rng.gen::<f32>() * (2.0 * offset * 100.0) / 100.0 - offset;
        let z = angle.cos() * radius + displacement;
        let mut model = Mat4::from_translation(vec3(x, y, z));

        // 2. scale: Scale between 0.05 and 0.25f
        let scale = rng.gen_range(0.0..20.0) / 100.0 + 0.05;
        model *= Mat4::from_scale(vec3(scale, scale, scale));

        // 3. rotation: add random rotation around a (semi)randomly picked rotation axis vector
        let rot_angle = rng.gen_range(0..360) as f32;
        model *= Mat4::from_axis_angle(vec3(0.4, 0.6, 0.8), rot_angle.to_radians());

        // 4. now add to list of matrices
        modelMatrices.push(model);
    }

    // Buffer
    let mut instanceVBO: GLuint = 0;

    unsafe {
        // configure instanced array
        // -------------------------
        gl::GenBuffers(1, &mut instanceVBO);
        gl::BindBuffer(gl::ARRAY_BUFFER, instanceVBO);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<Mat4>() * amount as usize) as GLsizeiptr,
            modelMatrices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
    }

    // render loop
    while !window.should_close() {
        let currentFrame = glfw.get_time() as f32;
        state.deltaTime = currentFrame - state.lastFrame;
        state.lastFrame = currentFrame;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        unsafe {
            // render
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT); //  | gl::DEPTH_BUFFER_BIT);

            gl::MatrixMode(gl::PROJECTION);
            gl::LoadIdentity();
            gl::Ortho(0.0, 400.0, 0.0, 600.0, -1.0, 1.0);
        }

        window.swap_buffers();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    // unsafe {
        // gl::DeleteShader(asteroidShader.id);
    // }
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
            state.camera.ProcessKeyboard(CameraMovement::FORWARD, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::BACKWARD, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::LEFT, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::RIGHT, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::UP, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::DOWN, state.deltaTime);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        _evt => {
            // println!("WindowEvent: {:?}", evt);
        }
    }
}

// glfw: whenever the window size changed (by OS or user resize) this event fires.
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_event(_window: &mut glfw::Window, width: i32, height: i32) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    println!("Framebuffer size: {}, {}", width, height);
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

    state.camera.ProcessMouseMovement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.camera.ProcessMouseScroll(yoffset as f32);
}
