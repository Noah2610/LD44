extern crate amethyst;
#[cfg(feature = "debug")]
extern crate backtrace;
#[cfg(feature = "encrypt_savefile")]
extern crate base64;
#[cfg(feature = "debug")]
extern crate chrono;
extern crate climer;
extern crate deathframe;
extern crate json;
extern crate regex;
#[macro_use]
extern crate serde;

mod bullet_creator;
mod components;
mod level_manager;
mod misc;
mod resolution_parser;
mod resource_helpers;
mod settings;
mod solid_tag;
mod states;
mod systems;
mod world_helpers;

use std::env;
use std::time::Duration;

use amethyst::audio::AudioBundle;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{
    ColorMask,
    DepthMode,
    DisplayConfig,
    DrawFlat2D,
    Pipeline,
    RenderBundle,
    Stage,
    ALPHA,
};
use amethyst::ui::{DrawUi, UiBundle};
use amethyst::utils::fps_counter::FPSCounterBundle;
use amethyst::{LogLevelFilter, LoggerConfig};

use deathframe::custom_game_data::prelude::*;
use deathframe::handlers::AudioHandles;

use resource_helpers::*;
use systems::prelude::*;

pub mod meta {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const NAME: &str = env!("CARGO_PKG_NAME");
}

const FPS: u32 = 60;
const SLEEP_AND_YIELD_MS: u64 = 2;

#[derive(Clone)]
pub struct CustomData {
    pub display_config: DisplayConfig,
}

fn main() -> Result<(), String> {
    #[cfg(feature = "debug")]
    std::panic::set_hook(Box::new(misc::on_panic));

    print_welcome_message();

    maybe_exit();

    init_game().map_err(|e| e.to_string())
}

fn print_welcome_message() {
    let name_and_version = format!("{} v{}", meta::NAME, meta::VERSION);
    println!(
        "{}\n{}\nThanks for playing our game! <3",
        name_and_version,
        "-".repeat(name_and_version.len()),
    );
}

fn maybe_exit() {
    // Exit game if environment variable `EXIT` is set.
    // Used for validating a correct build, without a graphical environment.
    const EXIT_VAR_NAME: &str = "STABMAN_EXIT";
    if env::vars()
        .any(|(key, val)| key == EXIT_VAR_NAME && !val.is_empty() && val != "0")
    {
        println!("Environment variable `{}` is set, exiting.", EXIT_VAR_NAME);
        std::process::exit(0);
    }
}

fn init_game() -> amethyst::Result<()> {
    start_logger();

    let game_data = build_game_data()?;

    let mut game: amethyst::CoreApplication<CustomGameData<CustomData>> =
        Application::build("./", states::Startup::default())?
            // https://docs-src.amethyst.rs/stable/amethyst_core/frame_limiter
            .with_frame_limit(
                FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(
                    SLEEP_AND_YIELD_MS,
                )),
                FPS,
            )
            .build(game_data)?;
    game.run();

    Ok(())
}

fn start_logger() {
    amethyst::start_logger(LoggerConfig {
        level_filter: LogLevelFilter::Error,
        ..Default::default()
    });
}

fn build_game_data<'a, 'b>(
) -> amethyst::Result<CustomGameDataBuilder<'a, 'b, CustomData>> {
    // Display config
    let display_config = get_display_config();

    // CustomGameData CustomData
    let custom_data = CustomData {
        display_config: display_config.clone(),
    };

    // Pipeline
    // let mut color_mask = ColorMask::empty();
    // color_mask.insert(ColorMask::RED);
    // color_mask.insert(ColorMask::GREEN);
    // color_mask.insert(ColorMask::BLUE);
    let stage = Stage::with_backbuffer()
        .clear_target([0.2, 0.2, 0.2, 1.0], 10.0)
        .with_pass(DrawFlat2D::new().with_transparency(
            ColorMask::all(),
            ALPHA,
            // NOTE: I have no idea what this `DepthMode` does, as it isn't documented,
            //       but sprite ordering via their z positions only works with this `DepthMode` variant.
            Some(DepthMode::LessEqualWrite),
        ))
        .with_pass(DrawUi::new()); // NOTE: "It's recommended this be your last pass."
                                   // .with_pass(DrawDebugLines::<PosColorNorm>::new()); // TODO
    let pipeline = Pipeline::build().with_stage(stage);

    // Bundles
    let transform_bundle = TransformBundle::new();
    let render_bundle =
        RenderBundle::new(pipeline, Some(display_config.clone()))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&["transform_system"]);
    let input_bundle = InputBundle::<String, String>::new()
        .with_bindings_from_file(&resource("config/bindings.ron"))?;
    let ui_bundle = UiBundle::<String, String>::new();
    let audio_bundle = AudioBundle::new(|_: &mut AudioHandles| None); // I hate this
    let fps_bundle = FPSCounterBundle;

    // Create GameDataBuilder
    let mut game_data = CustomGameData::<CustomData>::new()
        .custom(custom_data)
        .dispatcher("startup")?
        .dispatcher("main_menu")?
        .dispatcher("ingame")?
        .dispatcher("paused")?
        .dispatcher("continue_or_new_game_menu")?
        .dispatcher("win_game_menu")?
        .dispatcher("bonus_select_menu")?
        .with_bundle("ingame", audio_bundle)? // initialize before input_bundle; https://github.com/amethyst/amethyst/issues/1779
        .with_core_bundle(transform_bundle)?
        .with_core_bundle(render_bundle)?
        .with_core_bundle(input_bundle)?
        .with_core_bundle(ui_bundle)?
        .with_core_bundle(fps_bundle)?
        .with_core(InputManagerSystem, "input_manager_system", &[
            "input_system",
        ])?
        .with_core(ScaleSpritesSystem, "scale_sprites_system", &[])?
        .with_core(TimerSystem::default(), "timer_system", &[])?
        .with("ingame", PlayerControlsSystem, "player_controls_system", &[
        ])?
        .with("ingame", GravitySystem, "gravity_system", &[])?
        .with(
            "ingame",
            LimitVelocitiesSystem,
            "limit_velocities_system",
            &["gravity_system", "player_controls_system"],
        )?
        .with(
            "ingame",
            MoveEntitiesSystem::<solid_tag::SolidTag>::default(),
            "move_entities_system",
            &[
                "gravity_system",
                "limit_velocities_system",
                "player_controls_system",
            ],
        )?
        .with("ingame", CameraSystem, "camera_system", &[
            "move_entities_system",
        ])?
        .with(
            "ingame",
            ConfineEntitiesSystem,
            "confine_entities_system",
            &["move_entities_system", "camera_system"],
        )?
        .with("ingame", ParallaxSystem, "parallax_system", &[
            "move_entities_system",
            "camera_system",
        ])?
        .with("ingame", CollisionSystem, "collision_system", &[
            "move_entities_system",
        ])?
        .with(
            "ingame",
            DecreaseVelocitiesSystem,
            "decrease_velocities_system",
            &[
                "gravity_system",
                "limit_velocities_system",
                "move_entities_system",
                "player_controls_system",
            ],
        )?
        .with("ingame", AnimationSystem, "animation_system", &[])?
        .with("ingame", PlayerAttackSystem, "player_attack_system", &[
            "player_controls_system",
            "decrease_velocities_system",
            "limit_velocities_system",
            "collision_system",
        ])?
        .with(
            "ingame",
            PlayerTakeDamageSystem,
            "player_take_damage_system",
            &["player_controls_system", "collision_system"],
        )?
        .with(
            "ingame",
            HealthDisplaySystem::default(),
            "health_display_system",
            &["player_take_damage_system"],
        )?
        .with("ingame", GoalSystem, "goal_system", &["collision_system"])?
        .with("ingame", BulletSystem, "bullet_system", &[
            "collision_system",
        ])?
        .with("ingame", EnemyAiSystem, "enemy_ai_system", &[
            "decrease_velocities_system",
            "limit_velocities_system",
            "player_attack_system",
            "collision_system",
        ])?
        .with("ingame", HeartsSystem::default(), "hearts_system", &[
            "move_entities_system",
            "player_attack_system",
            "enemy_ai_system",
        ])?
        .with(
            "ingame",
            SyncHeartsContainersWithHealthSystem,
            "sync_hearts_containers_with_health",
            &[
                "hearts_system",
                "collision_system",
                "player_attack_system",
                "player_take_damage_system",
                "bullet_system",
                "enemy_ai_system",
            ],
        )?
        .with("ingame", BulletCreatorSystem, "bullet_creator_system", &[
            "player_controls_system",
            "enemy_ai_system",
        ])?
        .with("ingame", HarmfulSystem, "harmful_system", &[
            "collision_system",
        ])?
        .with(
            "ingame",
            PlayerDashSystem::default(),
            "player_dash_system",
            &["move_entities_system"],
        )?
        .with("ingame", LoaderSystem, "loader_system", &[
            "move_entities_system",
        ])?;

    if in_development_mode() {
        game_data = game_data
            .with_core(DebugSystem::default(), "debug_system", &[])?
            .with("ingame", NoclipSystem::default(), "noclip_system", &[])?;
    }

    Ok(game_data)
}

#[cfg(feature = "debug")]
pub fn in_development_mode() -> bool {
    const DEV_VAR_NAME: &str = "STABMAN_DEV";
    env::vars()
        .any(|(key, val)| key == DEV_VAR_NAME && !val.is_empty() && val != "0")
}

#[cfg(not(feature = "debug"))]
pub fn in_development_mode() -> bool {
    false
}

fn get_display_config() -> DisplayConfig {
    let mut display_config =
        DisplayConfig::load(&resource("config/display.ron"));

    // Overwrite dimensions with resolution specified in `resolution.txt` file.
    match resolution_parser::get_resolution() {
        Ok(Some(resolution)) => {
            display_config.dimensions = Some(resolution);
            if display_config.max_dimensions.is_some() {
                display_config.max_dimensions = Some(resolution);
            }
            if display_config.min_dimensions.is_some() {
                display_config.min_dimensions = Some(resolution);
            }
        }
        Ok(None) => (),
        Err(err) => panic!(err),
    }

    display_config
}
