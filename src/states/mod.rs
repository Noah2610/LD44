mod ingame;
mod main_menu;
mod paused;
mod startup;

pub mod prelude {
    pub use super::ingame::Ingame;
    pub use super::main_menu::MainMenu;
    pub use super::paused::Paused;
    pub use super::startup::Startup;
}

mod state_prelude {
    pub use amethyst::assets::{AssetStorage, Loader};
    pub use amethyst::ecs::{Entities, Entity, World};
    pub use amethyst::input::is_close_requested;
    pub use amethyst::prelude::*;
    pub use amethyst::renderer::{
        Camera as AmethystCamera,
        DisplayConfig,
        PngFormat,
        Projection,
        SpriteRender,
        SpriteSheet,
        SpriteSheetFormat,
        SpriteSheetHandle,
        Texture,
        TextureMetadata,
        Transparent,
        VirtualKeyCode,
    };
    pub use amethyst::shrev::{EventChannel, ReaderId};
    pub use amethyst::ui::{
        Anchor as AmethystAnchor,
        TtfFormat,
        UiCreator,
        UiEvent,
        UiEventType,
        UiText,
        UiTransform,
    };
    pub use amethyst::{State, StateData, StateEvent, Trans};

    pub use climer::Timer;
    pub use deathframe::custom_game_data::prelude::*;
    pub use deathframe::handlers::prelude::*;
    pub use deathframe::input_manager::InputManager;

    pub use super::helpers::*;
    pub use super::prelude::*;
    pub use crate::bullet_creator::prelude::*;
    pub use crate::components::prelude::*;
    pub use crate::resource_helpers::*;
    pub use crate::settings::prelude::*;
    pub use crate::world_helpers::*;
    pub use crate::CustomData;
}

pub use prelude::*;

pub mod helpers;
