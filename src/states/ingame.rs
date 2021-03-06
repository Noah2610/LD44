use super::state_prelude::*;

pub struct Ingame {
    campaign:      CampaignType,
    level_manager: Option<LevelManager>,
    to_main_menu:  bool,
    new_game:      bool,
}

impl Ingame {
    pub fn builder() -> IngameBuilder {
        IngameBuilder::default()
    }

    fn handle_keys<'a, 'b>(
        &self,
        data: &StateData<CustomGameData<CustomData>>,
    ) -> Option<Trans<CustomGameData<'a, 'b, CustomData>, StateEvent>> {
        let input_manager = data.world.input_manager();

        if input_manager.is_down("pause") {
            let paused_state = Box::new(Paused::default());
            Some(Trans::Push(paused_state))
        } else {
            None
        }
    }

    fn level_manager(&self) -> &LevelManager {
        self.level_manager.as_ref().expect("LevelManager is None")
    }

    fn level_manager_mut(&mut self) -> &mut LevelManager {
        self.level_manager.as_mut().expect("LevelManager is None")
    }
}

impl<'a, 'b> State<CustomGameData<'a, 'b, CustomData>, StateEvent> for Ingame {
    fn on_start(&mut self, mut data: StateData<CustomGameData<CustomData>>) {
        // Initialize the LevelManager
        let settings = data.world.settings();
        let level_manager_settings = match self.campaign {
            CampaignType::Normal => settings.level_manager.normal,
            CampaignType::BonusA => settings.level_manager.bonus_a,
            CampaignType::BonusB => settings.level_manager.bonus_b,
        };
        self.level_manager = Some(LevelManager::new(
            &mut data,
            level_manager_settings,
            self.new_game,
        ));

        self.level_manager_mut().on_start(&mut data);
    }

    fn on_stop(&mut self, mut data: StateData<CustomGameData<CustomData>>) {
        self.level_manager().on_stop(&mut data);
    }

    fn on_pause(&mut self, mut data: StateData<CustomGameData<CustomData>>) {
        self.level_manager().on_pause(&mut data);
    }

    fn on_resume(&mut self, mut data: StateData<CustomGameData<CustomData>>) {
        self.level_manager().on_resume(&mut data);

        // Return to main menu, if `Paused` state set the resource to do so
        self.to_main_menu = data.world.read_resource::<ToMainMenu>().0;
    }

    fn handle_event(
        &mut self,
        _data: StateData<CustomGameData<CustomData>>,
        event: StateEvent,
    ) -> Trans<CustomGameData<'a, 'b, CustomData>, StateEvent> {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    Trans::Pop
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn update(
        &mut self,
        mut data: StateData<CustomGameData<CustomData>>,
    ) -> Trans<CustomGameData<'a, 'b, CustomData>, StateEvent> {
        // Return to main menu, if necessary
        if self.to_main_menu {
            return Trans::Pop;
        }

        data.data.update(&data.world, "ingame").unwrap();

        self.level_manager_mut().update(&mut data);
        if self.level_manager().has_won_game {
            // Switch to WinGameMenu
            return Trans::Switch(Box::new(WinGameMenu::new(
                self.campaign.clone(),
            )));
        }

        if let Some(trans) = self.handle_keys(&data) {
            return trans;
        }

        Trans::None
    }
}

#[derive(Default)]
pub struct IngameBuilder {
    campaign: CampaignType,
    new_game: bool,
}

impl IngameBuilder {
    pub fn campaign(mut self, campaign: CampaignType) -> Self {
        self.campaign = campaign;
        self
    }

    pub fn new_game(mut self, new_game: bool) -> Self {
        self.new_game = new_game;
        self
    }

    pub fn build(self) -> Ingame {
        Ingame {
            campaign:      self.campaign,
            level_manager: None,
            to_main_menu:  false,
            new_game:      self.new_game,
        }
    }
}
