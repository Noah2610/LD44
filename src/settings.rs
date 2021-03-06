use deathframe::geo::Vector;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod prelude {
    pub use super::Settings;
    pub use super::SettingsCamera;
    pub use super::SettingsEntityLoader;
    pub use super::SettingsHarmful;
    pub use super::SettingsItem;
    pub use super::SettingsItems;
    pub use super::SettingsLevelManager;
    pub use super::SettingsLevelManagerCampaign;
    pub use super::SettingsLoadingText;
    pub use super::SettingsNoclip;
    pub use super::SettingsPlayer;
    pub use super::SettingsPlayerQuickTurnaround;
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub player:                 SettingsPlayer,
    pub camera:                 SettingsCamera,
    pub loading_text:           SettingsLoadingText,
    pub level_manager:          SettingsLevelManager,
    pub enemies:                SettingsEnemies,
    pub items:                  SettingsItems,
    pub music_volume:           f32,
    pub music_volume_paused:    f32,
    pub death_floor:            f32,
    pub harmful:                SettingsHarmful,
    pub entity_loader:          SettingsEntityLoader,
    pub timers_print_to_stdout: bool,
    pub noclip:                 SettingsNoclip,
}

#[derive(Clone, Deserialize)]
pub struct SettingsCamera {
    pub size_mult:       Vector,
    pub inner_size_mult: Vector,
    pub base_speed:      Vector,
    pub deadzone:        Vector,
    pub loader_padding:  Vector,
}

#[derive(Clone, Deserialize)]
pub struct SettingsPlayer {
    pub size:                       Vector,
    pub acceleration:               Vector,
    pub jump_strength:              f32,
    pub wall_jump_strength:         Vector,
    pub decr_jump_strength:         f32,
    pub min_jump_velocity:          f32,
    pub max_velocity:               (Option<f32>, Option<f32>),
    pub decr_velocity:              Vector,
    pub gravity:                    Vector,
    pub jump_gravity:               Vector,
    pub slide_strength:             f32,
    pub quick_turnaround:           SettingsPlayerQuickTurnaround,
    pub air_quick_turnaround:       SettingsPlayerQuickTurnaround,
    pub decrease_x_velocity_in_air: bool,
    pub health:                     u32,
    pub max_health:                 u32,
    pub damage:                     u32,
}

#[derive(Clone)]
pub enum SettingsPlayerQuickTurnaround {
    No             = 0,
    ResetVelocity  = 1,
    InvertVelocity = 2,
}

#[derive(Clone, Deserialize)]
pub struct SettingsLoadingText {
    pub text:      String,
    pub font_file: String,
    pub font_size: f32,
}

#[derive(Clone, Deserialize)]
pub struct SettingsLevelManager {
    pub normal:  SettingsLevelManagerCampaign,
    pub bonus_a: SettingsLevelManagerCampaign,
    pub bonus_b: SettingsLevelManagerCampaign,
}

#[derive(Clone, Deserialize)]
pub struct SettingsLevelManagerCampaign {
    pub levels_dir:               String,
    pub level_names:              Vec<String>,
    pub song_names:               Vec<String>,
    pub tile_size:                Vector,
    pub savefile_path:            String,
    pub health_increase_on_death: u32,
    pub level_timer_ui:           SettingsTimerUi,
    pub global_timer_ui:          SettingsTimerUi,
}

#[derive(Clone, Deserialize)]
pub struct SettingsTimerUi {
    pub text_prefix: String,
    pub font_file:   String,
    pub font_size:   f32,
    pub font_color:  [f32; 4],
    pub offset:      Vector,
    pub highscore:   SettingsTimerUiHighscore,
}

#[derive(Clone, Deserialize)]
pub struct SettingsTimerUiHighscore {
    pub text_prefix: String,
    pub font_file:   String,
    pub font_size:   f32,
    pub font_color:  [f32; 4],
    pub offset:      Vector,
}

#[derive(Clone, Deserialize)]
pub struct SettingsEnemies {
    pub gravity:     Vector,
    pub normal:      SettingsEnemy,
    pub charger:     SettingsEnemy,
    pub flying:      SettingsEnemy,
    pub reaper:      SettingsEnemy,
    pub turret:      SettingsEnemy,
    pub turret_data: SettingsEnemyTurret,
}

#[derive(Clone, Deserialize)]
pub struct SettingsEnemy {
    pub health:                u32,
    pub damage:                u32,
    pub reward:                u32,
    pub knockback:             Vector,
    pub trigger_distance:      Vector,
    pub acceleration:          Vector,
    pub max_velocity:          (Option<f32>, Option<f32>),
    pub decr_velocity:         Vector,
    pub affected_by_knockback: bool,
}

#[derive(Clone, Deserialize)]
pub struct SettingsEnemyTurret {
    pub shot_interval_ms:   u64,
    pub bullet_velocity:    Vector,
    pub bullet_size:        Vector,
    pub bullet_lifetime_ms: u64,
}

#[derive(Clone, Deserialize)]
pub struct SettingsItems {
    pub extra_jump:     SettingsItem,
    pub wall_jump:      SettingsItem,
    pub knockback:      SettingsItem,
    pub bullet_shoot:   SettingsItem,
    pub dash:           SettingsItem,
    pub bullet_deflect: SettingsItem,
    pub thrust:         SettingsItem,
    pub speed_up:       SettingsItem,
    pub jump_up:        SettingsItem,
    pub damage_up:      SettingsItem,
    pub settings:       SettingsItemSettings,
}

#[derive(Clone, Deserialize)]
pub struct SettingsItem {
    pub cost: u32,
}

#[derive(Clone, Deserialize)]
pub struct SettingsItemSettings {
    pub knockback_strength:           Vector,
    pub bullet_shoot_damage:          u32,
    pub bullet_shoot_velocity:        Vector,
    pub bullet_shoot_size:            Vector,
    pub bullet_shoot_lifetime_ms:     u64,
    pub dash_duration_ms:             u64,
    pub dash_velocity:                Vector,
    pub dash_input_delay_ms:          u64,
    pub dash_double_tap:              bool,
    pub dash_only_in_air:             bool,
    pub bullet_deflect_damage:        u32,
    pub bullet_deflect_velocity_mult: Vector,
    pub bullet_deflect_lifetime_ms:   u64,
    pub thrust_strength:              Vector,
    pub speed_up_max_velocity_up:     f32,
    pub speed_up_acceleration_up:     f32,
    pub jump_up:                      f32,
    pub damage_up:                    u32,
}

#[derive(Clone, Deserialize)]
pub struct SettingsHarmful {
    pub knockback_strength: (f32, f32),
}

#[derive(Clone, Deserialize)]
pub struct SettingsEntityLoader {
    pub load_distance:                  (f32, f32),
    pub enemy_load_distance_difference: (f32, f32),
    pub bullet_load_distance:           (f32, f32),
}

#[derive(Clone, Deserialize)]
pub struct SettingsNoclip {
    pub acceleration: Vector,
    pub max_velocity: (Option<f32>, Option<f32>),
}

impl<'de> Deserialize<'de> for SettingsPlayerQuickTurnaround {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use SettingsPlayerQuickTurnaround as QTA;

        let value = i32::deserialize(deserializer)?;
        match value {
            0 => Ok(QTA::No),
            1 => Ok(QTA::ResetVelocity),
            2 => Ok(QTA::InvertVelocity),
            _ => {
                Err(D::Error::custom(format!("Value out of range: {}", value)))
            }
        }
    }
}

impl Serialize for SettingsPlayerQuickTurnaround {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.clone() as u8)
    }
}
