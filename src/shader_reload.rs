use bevy::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref SHADER_RELOAD_TRIGGER: Mutex<Option<String>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn reload_shader(shader_path: &str) {
    let mut trigger = SHADER_RELOAD_TRIGGER.lock().unwrap();
    *trigger = Some(shader_path.to_string());
}

fn should_reload_shader() -> Option<String> {
    let mut trigger = SHADER_RELOAD_TRIGGER.lock().unwrap();
    trigger.take()
}

fn check_for_shader_reload(world: &mut World) {
    if let Some(shader_path) = should_reload_shader() {
        let asset_server = world.resource::<AssetServer>();
        asset_server.reload(&shader_path);
    }
}

pub struct ShaderReloadPlugin;

impl Plugin for ShaderReloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_for_shader_reload);
    }
}
