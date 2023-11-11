#![feature(is_sorted)]
#![feature(extract_if)]
#![feature(offset_of)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod c2d_matrix;
mod cell_space_partition;
mod configuration;
mod constants;
mod core;
mod entity_functions;
mod entity_traits;
mod game_world;
mod inverted_aab_box_2d;
mod path;
mod shapes;
mod smoother;
mod steering_behavior;
mod transformations;
mod utils;
mod vehicle;
mod wall_2d;

extern crate glfw;

use crate::core::camera::{Camera, CameraMovement};
use crate::core::model::ModelBuilder;
use crate::core::shader::Shader;
use crate::core::texture::{Texture, TextureConfig, TextureFilter, TextureType};
use crate::game_world::GameWorld;
use glad_gl::gl;
use glam::{vec3, Mat4};
use glfw::{Action, Context, Key};
use log::error;
use std::path::PathBuf;
use std::rc::Rc;
use crate::core::mesh::{Color, Mesh};
use crate::shapes::mesh_plane::build_vertexes_and_indices;

const SCR_WIDTH: f32 = 1000.0;
const SCR_HEIGHT: f32 = 800.0;

struct State {
    camera: Camera,
    delta_time: f32,
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

    // perspective setting
    // //let camera = Camera::camera_vec3(vec3(300.0, 300.0, 500.0));
    let camera = Camera::camera_vec3_up_yaw_pitch(
        // vec3(400.0, 400.0, 700.0), for current x,y world
        vec3(0.0, 170.0, 500.0), // for xz world
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
        delta_time: 0.0,
        lastFrame: 0.0,
        firstMouse: true,
        lastX: SCR_WIDTH / 2.0,
        lastY: SCR_HEIGHT / 2.0,
    };

    let shader_texture = Shader::new(
        "assets/shaders/camera_texture.vert",
        "assets/shaders/camera_texture.frag",
        None::<String>,
    )
    .unwrap();

    let wavy_shader = Shader::new(
        "assets/shaders/wavy_texture.vert",
        "assets/shaders/wavy_texture.frag",
        None::<String>,
    )
    .unwrap();

    let model_shader = Shader::new("assets/shaders/basic_model.vert", "assets/shaders/basic_model.frag", None::<String>).unwrap();

    let model_shader = Rc::new(model_shader);

    let water_texture = Rc::new(
        Texture::new(
            PathBuf::from("assets/images/water_texture.png"),
            &TextureConfig {
                flip_v: true,
                gamma_correction: false,
                filter: TextureFilter::Linear,
                texture_type: TextureType::Diffuse,
            },
        )
        .unwrap(),
    );

    let sand_texture = Rc::new(
        Texture::new(
            PathBuf::from("assets/images/ground_0024_color_1k.jpg"),
            &TextureConfig {
                flip_v: false,
                gamma_correction: false,
                filter: TextureFilter::Linear,
                texture_type: TextureType::Diffuse,
            },
        )
            .unwrap(),
    );

    let (vertices, indices) = build_vertexes_and_indices(500, 500, Color::white());
    let surface_mesh = Mesh::new(vertices.clone(), indices.clone(), &water_texture, false);
    let bottom_mesh = Mesh::new(vertices, indices, &sand_texture, false);

    let big_fish = "assets/models/BarramundiFish/glTF/BarramundiFish.gltf";
    let fish_model = ModelBuilder::new(big_fish, model_shader.clone(), big_fish).build().unwrap();

    let game_world = GameWorld::new(SCR_WIDTH as i32, SCR_HEIGHT as i32, fish_model.clone());

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // render loop
    while !window.should_close() {
        let current_time = glfw.get_time() as f32;
        state.delta_time = current_time - state.lastFrame;
        state.lastFrame = current_time;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        unsafe {
            // render
            gl::ClearColor(0.1, 0.5, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let view = state.camera.get_view_matrix();
        let projection = Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), SCR_WIDTH / SCR_HEIGHT, 0.1, 2000.0);
        // let projection = Mat4::orthographic_rh_gl(0.0, 600.0, 0.0, 600.0, 0.1, 100.0);
        // let projection = Mat4::orthographic_rh_gl(0.0, 1000.0, 0.0, 1000.0, 0.0, 1000.0);

        GameWorld::Update(&game_world, state.delta_time);

        // bottom
        shader_texture.use_shader_with(&projection, &view);
        shader_texture.setFloat("alpha", 1.0);
        bottom_mesh.render(&shader_texture, vec3(-750.0, -5.0, -750.0), 0.0, vec3(3.0, 1.0, 3.0));

        // fish
        model_shader.use_shader_with(&projection, &view);
        game_world.borrow().render(state.delta_time);

        // surface
        wavy_shader.use_shader_with(&projection, &view);
        wavy_shader.setFloat("alpha", 0.4);
        wavy_shader.setFloat("current_time", current_time);
        surface_mesh.render(&wavy_shader, vec3(-750.0, 100.0, -750.0), 0.0, vec3(3.0, 1.0, 3.0));

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
            state.camera.ProcessKeyboard(CameraMovement::Forward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::S, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Backward, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::A, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Left, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::D, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Right, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Up, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state.camera.ProcessKeyboard(CameraMovement::Down, state.delta_time);
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
    state.camera.ProcessMouseScroll(yoffset as f32);
}
