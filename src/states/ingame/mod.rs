mod level_loader;
mod level_manager;

use super::state_prelude::*;
use level_manager::prelude::*;

pub struct Ingame {
    campaign:      CampaignType,
    level_manager: Option<LevelManager>,
    to_main_menu:  bool,
}

impl Ingame {
    pub fn new(campaign: CampaignType) -> Self {
        Self {
            campaign:      campaign,
            level_manager: None,
            to_main_menu:  false,
        }
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
            CampaignType::Bonus => settings.level_manager.bonus,
        };
        self.level_manager =
            Some(LevelManager::new(&mut data, level_manager_settings));

        // Initialize global timer
        // NOTE: This needs to happen before the level loads
        if self.level_manager().is_first_level() {
            let mut timers = data.world.write_resource::<Timers>();
            let timer = climer::Timer::default();
            timers.global = Some(timer);
        }

        self.level_manager_mut().load_current_level(&mut data);
        // Force update `HealthDisplay`
        data.world.write_resource::<UpdateHealthDisplay>().0 = true;

        // Now start the global timer
        data.world
            .write_resource::<Timers>()
            .global
            .as_mut()
            .map(|timer| timer.start().unwrap());
    }

    fn on_stop(&mut self, data: StateData<CustomGameData<CustomData>>) {
        // Delete _ALL_ entities before
        data.world.delete_all();

        // Stop global timer
        let mut timers = data.world.write_resource::<Timers>();
        timers.global = None;
    }

    fn on_resume(&mut self, data: StateData<CustomGameData<CustomData>>) {
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

        if let Some(trans) = self.handle_keys(&data) {
            return trans;
        }

        self.level_manager_mut().update(&mut data);

        Trans::None
    }
}
