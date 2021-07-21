use cgmath::{ortho, point2, point3, vec3, Matrix4, Point2, Vector3};

pub struct Camera {
    pub position: Point2<f32>,
}

static CAMERA_DIRECTION: Vector3<f32> = vec3(0.0, 0.0, -1.0);
static CAMERA_UP: Vector3<f32> = vec3(0.0, 1.0, 0.0);

impl Camera {
    pub fn new() -> Self {
        Self {
            position: point2(0.0, 0.0),
        }
    }

    pub fn move_horizontal(&mut self, amount: f32) {
        self.position.x += amount;
    }

    pub fn move_vertical(&mut self, amount: f32) {
        self.position.y += amount;
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            point3(self.position.x, self.position.y, 1.0),
            CAMERA_DIRECTION,
            CAMERA_UP,
        )
    }

    pub fn projection_matrix(&self, width: f32, height: f32, zoom: f32) -> Matrix4<f32> {
        ortho(
            -width / zoom / 2.0,
            width / zoom / 2.0,
            -height / zoom / 2.0,
            height / zoom / 2.0,
            1.0,
            -1.0,
        )
    }
}
