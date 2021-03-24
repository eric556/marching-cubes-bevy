use interpolation::{lerp, Lerp};
use crate::settings::{MovementSettings, CameraSettings};
use bevy::{input::mouse::MouseMotion, input::mouse::MouseWheel, prelude::*, render::camera::Camera, render::camera::PerspectiveProjection, render::camera::VisibleEntities, render::render_graph::base};
use bevy::prelude::Vec3;

pub const CAMERA_UPDATE: &str = "camera_update";
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub vertical_offset: f32,
    pub y_axis_rotation_offset: f32,
}

#[derive(Default)]
pub struct FollowTarget;

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            distance: 10.0,
            vertical_offset: 10.0,
            y_axis_rotation_offset: 0.0,
        }
    }
}

#[derive(Bundle)]
pub struct ThirdPerson3DCameraBundle {
    pub perspective_projection: PerspectiveProjection,
    pub visible_entities: VisibleEntities,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub third_person_camera: ThirdPersonCamera,
    pub camera: Camera

}

impl Default for ThirdPerson3DCameraBundle {
    fn default() -> Self {
        ThirdPerson3DCameraBundle {
            camera: Camera {
                name: Some(base::camera::CAMERA_3D.to_string()),
                ..Default::default()
            },
            perspective_projection: Default::default(),
            visible_entities: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            third_person_camera: ThirdPersonCamera::default()
        }
    }
}

fn follow_target(time: Res<Time>, mut player_query: Query<(&FollowTarget, &Transform)>, mut camera_query: Query<(&mut Transform, &ThirdPersonCamera)>) {
    for (_, p_transform) in &mut player_query.iter() {
        for (mut c_transform, camera) in camera_query.iter_mut() {
            let vertical = p_transform.translation.y + camera.vertical_offset;
            let horizontal = ((camera.distance * camera.distance) - (vertical * vertical)).sqrt();
            let player_rotation = camera.y_axis_rotation_offset;
            let camera_position = Vec3::new(
                horizontal * player_rotation.sin() + p_transform.translation.x, 
                vertical, 
                horizontal * player_rotation.cos() + p_transform.translation.z);

            // let temp = Transform::from_matrix(Mat4::face_toward(
            //     camera_position,             // The player position plus the cameras offset
            //     // Vec3::new(20.0, 20.0, 20.0),
            //     p_transform.translation,            // The Player Position
            //     Vec3::new(0.0, 1.0, 0.0),
            // ));

            c_transform.translation = camera_position;
            println!("Pos: {:?}", camera_position);
            c_transform.look_at(p_transform.translation, Vec3::unit_y());

            // if !temp.translation.x.is_nan() && !temp.translation.y.is_nan() && !temp.translation.z.is_nan() {
            //     c_transform.translation = temp.translation;
            //     c_transform.rotation = temp.rotation;
            // }   
        }
    }
}

fn mouse_motion_system(
    time: Res<Time>,
    mut state: ResMut<State>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    movement_settings: Res<MovementSettings>,
    camera_settings: Res<CameraSettings>,
    mut query: Query<&mut ThirdPersonCamera>,
) {
    let mut delta: Vec2 = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        delta += event.delta;
    }

    let mut zoom: f32 = 0.0;
    for event in state.mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        zoom += event.y;
    }

    if delta == Vec2::zero() && zoom == 0.0 {
        return;
    }

    let delta_vertical = delta.y * movement_settings.sensitivity_vertical * time.delta_seconds();
    let delta_rotation = delta.x * movement_settings.sensitivity_rotational * time.delta_seconds();
    let delta_distance = zoom * movement_settings.sensitivity_scroll * time.delta_seconds();

    for mut camera in query.iter_mut() {
        camera.vertical_offset += delta_vertical;
        camera.y_axis_rotation_offset -= delta_rotation;
        let target_distance = camera.distance - delta_distance;
        camera.distance = lerp(&camera.distance, &target_distance, &camera_settings.zoom_lerp);
    }
}

#[derive(Default)]
struct State {
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

pub struct ThirdPersonCameraPlugin;

impl Plugin for ThirdPersonCameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<State>()
            .add_system_to_stage(stage::PRE_UPDATE, mouse_motion_system.system())
            .add_system_to_stage(stage::LAST, follow_target.system());
    }
}
