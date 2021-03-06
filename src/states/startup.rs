use amethyst::audio::AudioSink;

use super::state_prelude::*;

pub struct Startup {
    loading_text_entity: Option<Entity>,
}

impl Startup {
    // TODO
    fn is_finished_loading(&self) -> bool {
        true
    }

    fn initialize_loading_text(
        &mut self,
        data: &mut StateData<CustomGameData<CustomData>>,
    ) {
        let world = &mut data.world;

        let settings = world.settings();

        let screen_size = data
            .data
            .custom
            .clone()
            .unwrap()
            .display_config
            .dimensions
            .unwrap_or((1200, 800));

        let font = world.read_resource::<AssetLoader>().load(
            resource(settings.loading_text.font_file),
            TtfFormat,
            Default::default(),
            (),
            &world.read_resource(),
        );

        let transform = new_ui_transform(
            "loading",
            AmethystAnchor::Middle,
            (0.0, 0.0, 0.0, screen_size.0 as f32, screen_size.1 as f32, 0),
        );

        self.loading_text_entity = Some(
            world
                .create_entity()
                .with(transform)
                .with(UiText::new(
                    font,
                    settings.loading_text.text,
                    [1.0, 1.0, 1.0, 1.0],
                    settings.loading_text.font_size,
                ))
                .build(),
        );

        // Update manually once, so the "Loading" text is displayed
        data.data.update(&data.world, "startup").unwrap();
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b, CustomData>, StateEvent> for Startup {
    fn on_start(&mut self, mut data: StateData<CustomGameData<CustomData>>) {
        // Resources
        data.world.add_resource(load_settings());
        let settings = data.world.settings();
        let mut sprite_sheet_handles = SpriteSheetHandles::default();
        sprite_sheet_handles
            .load(resource("spritesheets/player_hearts.png"), &mut data.world);
        sprite_sheet_handles
            .load(resource("spritesheets/player_bullets.png"), &mut data.world);
        {
            let mut sink = data.world.write_resource::<AudioSink>();
            sink.set_volume(settings.music_volume);
        }
        let audio_handles = AudioHandles::default();
        data.world.add_resource(sprite_sheet_handles);
        data.world.add_resource(audio_handles);
        data.world.add_resource(TextureHandles::default());
        data.world.add_resource(InputManager::default());
        data.world.add_resource(BulletCreator::default());
        data.world.add_resource(UpdateHealthDisplay::default());
        data.world.add_resource(ToMainMenu::default());
        data.world.add_resource(Timers::default());
        data.world.add_resource(Stats::default());
        data.world.add_resource(CurrentLevelName::default());
        data.world.add_resource(LoadingLevel::default());

        // TODO
        // data.world
        //     .add_resource(DebugLines::new().with_capacity(200));
        // data.world
        //     .add_resource(DebugLinesParams { line_width: 1.0 });

        self.initialize_loading_text(&mut data);
    }

    fn update(
        &mut self,
        data: StateData<CustomGameData<CustomData>>,
    ) -> Trans<CustomGameData<'a, 'b, CustomData>, StateEvent> {
        data.data.update(&data.world, "startup").unwrap();

        if self.is_finished_loading() {
            let ingame = Box::new(MainMenu::default());
            // Remove loading text
            if let Some(entity) = self.loading_text_entity {
                data.world
                    .delete_entity(entity)
                    .expect("Should delete loading text entity");
            }
            return Trans::Switch(ingame);
        }

        Trans::None
    }
}

impl Default for Startup {
    fn default() -> Self {
        Self {
            loading_text_entity: None,
        }
    }
}

fn load_settings() -> Settings {
    let settings_raw = read_file(resource("config/settings.ron"))
        .expect("Couldn't read settings.ron file");
    ron::Value::from_str(&settings_raw)
        .unwrap()
        .into_rust()
        .unwrap()
}
