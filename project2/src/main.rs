use anyhow::Result;
use bevy::{input::mouse::*, prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPlugin};

use crate::{
    ai::AiPlugin,
    country::CountryPlugin,
    map::{MapPlugin, resources::MapSettings},
    player::PlayerPlugin,
    ui::UiPlugin,
};

mod ai;
mod common;
mod country;
mod map;
mod player;
mod ui;

fn log_error(In(result): In<Result<()>>) {
    if let Err(e) = result {
        error!("Error occured: {}", e);
    }
}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum GameState {
    #[default]
    Menu,
    CountrySelection,
    InGame,
}

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::InGame)]
pub enum InGameStates {
    #[default]
    Idle,
    MovingArmy,
}

fn main() {
    let _app = App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            MapPlugin {},
            CountryPlugin {},
            UiPlugin {},
            PlayerPlugin {},
            AiPlugin {},
        ))
        .init_state::<GameState>()
        .add_sub_state::<InGameStates>()
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, camera_zoom))
        .add_systems(PostUpdate, constraint_camera_movement)
        .run();
}

fn camera_movement(
    mut egui_contexts: EguiContexts,
    mut transform_query: Single<&mut Transform, With<Camera2d>>,
    projection_query: Single<&Projection, With<Camera2d>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion_evr: MessageReader<MouseMotion>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    if let Ok(ctx) = egui_contexts.ctx_mut()
        && ctx.is_pointer_over_area()
    {
        return;
    }

    let scale = if let Projection::Orthographic(o) = projection_query.into_inner() {
        o.scale
    } else {
        1.0
    };

    for ev in motion_evr.read() {
        transform_query.translation.x -= ev.delta.x * scale;
        transform_query.translation.y += ev.delta.y * scale;
    }
}

fn camera_zoom(
    mut projection: Single<&mut Projection, With<Camera2d>>,
    mut evr: MessageReader<MouseWheel>,
) {
    for event in evr.read() {
        if let Projection::Orthographic(o) = projection.as_mut() {
            o.scale -= event.y * 0.1;
            o.scale = o.scale.clamp(0.2, 5.0);
        }
    }
}

fn constraint_camera_movement(
    mut transform_query: Single<&mut Transform, With<Camera2d>>,
    mut projection_query: Single<&mut Projection, With<Camera2d>>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    map_settings: Res<MapSettings>,
) {
    let window = window_query.into_inner();
    let map_width = map_settings.width * map_settings.tile_size;
    let map_height = map_settings.height * map_settings.tile_size;

    let scale = if let Projection::Orthographic(o) = projection_query.as_mut() {
        let max_scale_x = map_width as f32 / window.width();
        let max_scale_y = map_height as f32 / window.height();

        let max_scale = max_scale_x.min(max_scale_y);

        o.scale = max_scale.min(o.scale);

        o.scale
    } else {
        1.0
    };

    let visible_height = window.height() * scale;
    let visible_width = window.width() * scale;

    let half_map_width = map_width as f32 / 2.0;
    let half_map_height = map_height as f32 / 2.0;

    let half_visible_width = visible_width / 2.0;
    let half_visible_height = visible_height / 2.0;

    let min_x = -half_map_width + half_visible_width;
    let max_x = half_map_width - half_visible_width;

    let min_y = -half_map_height + half_visible_height;
    let max_y = half_map_height - half_visible_height;

    let transform = transform_query.as_mut();

    if visible_width > map_width as f32 {
        transform.translation.x = 0.0;
    } else {
        transform.translation.x = transform.translation.x.clamp(min_x, max_x);
    }

    if visible_height > map_height as f32 {
        transform.translation.y = 0.0;
    } else {
        transform.translation.y = transform.translation.y.clamp(min_y, max_y);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection::default_2d()),
    ));
}
