#![allow(non_snake_case)]

use glam::*;

// Default camera values
pub const YAW: f32 = -90.0;
pub const PITCH: f32 = 0.0;
pub const SPEED: f32 = 2.5;
pub const SENSITIVITY: f32 = 0.1;
pub const ZOOM: f32 = 45.0;

// Defines several possible options for camera movement. Used as abstraction
// to stay away from window-system specific input methods
pub enum CameraMovement {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

#[derive(Default)]
pub struct Camera {
    // camera Attributes
    pub Position: Vec3,
    pub Front: Vec3,
    pub WorldUp: Vec3,
    pub Up: Vec3,
    pub Right: Vec3,
    // euler Angles
    pub Yaw: f32,
    pub Pitch: f32,
    // camera options
    pub MovementSpeed: f32,
    pub MouseSensitivity: f32,
    pub Zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            Position: vec3(0.0, 0.0, 3.0),
            Front: vec3(0.0, 0.0, -1.0),
            WorldUp: vec3(0.0, 1.0, 0.0),
            Up: vec3(0.0, 1.0, 0.0),
            Right: Default::default(),
            Yaw: YAW,
            Pitch: PITCH,
            MovementSpeed: SPEED,
            MouseSensitivity: SENSITIVITY,
            Zoom: ZOOM,
        }
    }

    // constructor with vectors
    pub fn camera_vec3(position: Vec3) -> Camera {
        let mut camera = Camera::new();
        camera.Position = position;
        camera.updateCameraVectors();
        camera
    }

    pub fn camera_vec3_up_yaw_pitch(position: Vec3, up: Vec3, yaw: f32, pitch: f32) -> Camera {
        let mut camera = Camera::new();
        camera.Position = position;
        camera.WorldUp = up;
        camera.Yaw = yaw;
        camera.Pitch = pitch;
        camera.updateCameraVectors();
        camera
    }

    // constructor with scalar values
    pub fn camera_scalar(posX: f32, posY: f32, posZ: f32, upX: f32, upY: f32, upZ: f32, yaw: f32, pitch: f32) -> Camera {
        let mut camera = Camera::new();
        camera.Position = vec3(posX, posY, posZ);
        camera.WorldUp = vec3(upX, upY, upZ);
        camera.Yaw = yaw;
        camera.Pitch = pitch;
        camera.updateCameraVectors();
        camera
    }

    // returns the view matrix calculated using Euler Angles and the LookAt Matrix
    pub fn GetViewMatrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.Position, self.Position + self.Front, self.Up)
    }

    // processes input received from any keyboard-like input system. Accepts input parameter
    // in the form of camera defined ENUM (to abstract it from windowing systems)
    pub fn ProcessKeyboard(&mut self, direction: CameraMovement, deltaTime: f32) {
        let velocity: f32 = self.MovementSpeed * deltaTime;

        match direction {
            CameraMovement::FORWARD => self.Position += self.Front * velocity,
            CameraMovement::BACKWARD => self.Position -= self.Front * velocity,
            CameraMovement::LEFT => self.Position -= self.Right * velocity,
            CameraMovement::RIGHT => self.Position += self.Right * velocity,
            CameraMovement::UP => self.Position += self.Up * velocity,
            CameraMovement::DOWN => self.Position -= self.Up * velocity,
        }

        // For FPS: make sure the user stays at the ground level
        // self.Position.y = 0.0; // <-- this one-liner keeps the user at the ground level (xz plane)
    }

    // processes input received from a mouse input system. Expects the offset value in both the x and y direction.
    pub fn ProcessMouseMovement(&mut self, mut xoffset: f32, mut yoffset: f32, constrainPitch: bool) {
        xoffset *= self.MouseSensitivity;
        yoffset *= self.MouseSensitivity;

        self.Yaw += xoffset;
        self.Pitch += yoffset;

        // make sure that when pitch is out of bounds, screen doesn't get flipped
        if constrainPitch {
            if self.Pitch > 89.0 {
                self.Pitch = 89.0;
            }
            if self.Pitch < -89.0 {
                self.Pitch = -89.0;
            }
        }

        // update Front, Right and Up Vectors using the updated Euler angles
        self.updateCameraVectors();
    }

    // processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn ProcessMouseScroll(&mut self, yoffset: f32) {
        self.Zoom -= yoffset;
        if self.Zoom < 1.0 {
            self.Zoom = 1.0;
        }
        if self.Zoom > 45.0 {
            self.Zoom = 45.0;
        }
    }

    // calculates the front vector from the Camera's (updated) Euler Angles
    fn updateCameraVectors(&mut self) {
        // calculate the new Front vector
        let front = vec3(
            self.Yaw.to_radians().cos() * self.Pitch.to_radians().cos(),
            self.Pitch.to_radians().sin(),
            self.Yaw.to_radians().sin() * self.Pitch.to_radians().cos(),
        );

        self.Front = front.normalize();

        // also re-calculate the Right and Up vector
        // normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.Right = self.Front.cross(self.WorldUp).normalize();
        self.Up = self.Right.cross(self.Front).normalize();
    }
}
