#![feature(is_sorted)]
#![feature(extract_if)]
#![feature(offset_of)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod base_entity;
mod c2d_matrix;
mod cell_space_partition;
mod constants;
mod entity_functions;
mod game_world;
mod inverted_aab_box_2d;
mod moving_entity;
mod param_loader;
mod path;
mod shapes;
mod smoother;
mod steering_behavior;
mod support;
mod transformations;
mod utils;
mod vehicle;
mod wall_2d;

extern crate glfw;

use crate::game_world::GameWorld;
use crate::shapes::line_box::LineBox;
use crate::shapes::small_fish::SmallFish;
use crate::shapes::triangle::Triangle;
use crate::support::camera::{Camera, CameraMovement};
use crate::support::shader::Shader;
use crate::support::SIZE_OF_FLOAT;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::{vec3, Mat4, Vec3};
use glfw::{Action, Context, Key};
use log::error;
use std::ptr;
use crate::shapes::plane::Plane;
use crate::support::texture::load_texture;

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

    // let camera = Camera::camera_vec3(vec3(300.0, 300.0, 500.0));
    let camera = Camera::camera_vec3(vec3(0.0, 0.0, 55.0)); // for ortho perspective

    // Initialize the world state
    let mut state = State {
        camera,
        deltaTime: 0.0,
        lastFrame: 0.0,
        firstMouse: true,
        lastX: SCR_WIDTH / 2.0,
        lastY: SCR_HEIGHT / 2.0,
    };

    let game_world = GameWorld::new(600, 600);

    let shader = Shader::new("assets/shaders/camera.vert", "assets/shaders/camera.frag", None).unwrap();
    let shader_texture = Shader::new("assets/shaders/camera_texture.vert", "assets/shaders/camera_texture.frag", None).unwrap();

    let fish_texture = load_texture("assets/images/fish_3.png", false, true);
    let plane_texture = load_texture("assets/images/water_texture.png", false, false);

    let line_box = LineBox::new();
    let triangle = Triangle::new();
    let fish = SmallFish::new(fish_texture);
    let plane = Plane::new(plane_texture);

    // let mut VAO: GLuint = 0;
    // let mut VBO: GLuint = 0;
    // let mut line_VAO: GLuint = 0;
    // let mut line_VBO: GLuint = 0;
    //
    // #[rustfmt::skip]
    // let vertices: [f32; 9] = [
    //     -1.0,  0.6,  0.0,
    //      1.0,  0.0,  0.0,
    //     -1.0, -0.6,  0.0
    // ];
    //
    // unsafe {
    //     // gl::Enable(gl::DEPTH_TEST);
    //
    //     gl::GenVertexArrays(1, &mut VAO);
    //     gl::GenBuffers(1, &mut VBO);
    //     gl::BindVertexArray(VAO);
    //
    //     gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
    //     gl::BufferData(
    //         gl::ARRAY_BUFFER,
    //         (vertices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
    //         vertices.as_ptr() as *const GLvoid,
    //         gl::STATIC_DRAW,
    //     );
    //
    //     gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * SIZE_OF_FLOAT) as GLsizei, ptr::null());
    //     gl::EnableVertexAttribArray(0);
    //
    //     gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    //     gl::BindVertexArray(0);
    // }

    let view = state.camera.get_view_matrix();

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

            // let projection = Mat4::perspective_rh_gl(state.camera.Zoom.to_radians(), SCR_WIDTH / SCR_HEIGHT, 0.1, 1000.0);
            let projection = Mat4::orthographic_rh_gl(0.0, 600.0, 0.0, 600.0, 0.1, 100.0);

            // let mut model_transform = Mat4::from_translation(Vec3::new(400.0, 400.0, 0.0));
            // model_transform *= Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), glfw.get_time() as f32);
            // model_transform *= Mat4::from_scale(Vec3::new(10.0, 10.0, 1.0));


            // shader.setMat4("model", &model_transform);
            // shader.setVec3("color", &vec3(1.0, 0.5, 0.2));
            // gl::BindVertexArray(VAO);
            // gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // gl::BindVertexArray(0);

            GameWorld::Update(&game_world, state.deltaTime);

            shader_texture.use_shader_with(&projection, &view);
            shader_texture.setInt("texture1", 0);

            // fish.render(&shader_texture, vec3(300.0, 300.0, 0.0), 0.0, vec3(20.0, 20.0, 1.0));
            //
            // // shader.use_shader_with(&projection, &view);
            plane.render(&shader_texture, vec3(300.0, 300.0, 0.0), 0.0, vec3(300.0, 300.0, 1.0));

            game_world.borrow().render(&shader_texture, &fish);
            // // line_box.render(&shader, vec3(200.0, 200.0, 0.0), vec3(50.0, 50.0, 1.0), &vec3(1.0, 1.0, 1.0));


        }

        window.swap_buffers();
    }

    unsafe {
        gl::DeleteShader(shader.id);
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
            state.camera.ProcessKeyboard(CameraMovement::Forward, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Backward, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Left, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Right, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Up, state.deltaTime);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Down, state.deltaTime);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        _evt => {
            println!("WindowEvent: {:?}", _evt);
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

    state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.camera.ProcessMouseScroll(yoffset as f32);
}
