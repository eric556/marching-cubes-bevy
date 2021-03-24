use bevy::prelude::*;

pub struct MovementSettings {
    pub sensitivity_vertical: f32,
    pub sensitivity_rotational: f32,
    pub sensitivity_scroll: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity_vertical: 1.0,
            sensitivity_rotational: 0.3,
            sensitivity_scroll: 100.0,
        }
    }
}

pub struct CameraSettings {
    pub zoom_lerp: f32,
    pub top_down_speed: f32,
    pub top_down_rotation_amount: f32,
    pub top_down_zoom_speed: f32
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            zoom_lerp: 0.5,
            top_down_speed: 15.0,
            top_down_rotation_amount: 200.0,
            top_down_zoom_speed: 10.0
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin{
    fn build(&self, app: &mut bevy::prelude::AppBuilder) { 
        app
            .init_resource::<MovementSettings>()
            .init_resource::<CameraSettings>();
    }
}