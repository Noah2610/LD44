mod items_data;

use super::component_prelude::*;
use super::Enemy;

pub use items_data::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub acceleration:               Vector,
    pub air_acceleration:           Vector,
    pub jump_strength:              f32,
    pub wall_jump_strength:         Vector,
    pub decr_jump_strength:         f32,
    pub min_jump_velocity:          f32,
    pub max_velocity:               (Option<f32>, Option<f32>),
    pub gravity:                    Vector,
    pub jump_gravity:               Vector,
    pub slide_strength:             f32,
    pub quick_turnaround:           SettingsPlayerQuickTurnaround,
    pub air_quick_turnaround:       SettingsPlayerQuickTurnaround,
    pub decrease_x_velocity_in_air: bool,
    pub health:                     u32,
    pub max_health:                 u32,
    pub damage:                     u32,
    pub is_attacking:               bool,
    pub in_control:                 bool,
    pub items_data:                 ItemsData,
}

impl Player {
    pub fn new(settings: SettingsPlayer) -> Self {
        Self {
            acceleration:               settings.acceleration,
            air_acceleration:           settings.acceleration, // TODO
            jump_strength:              settings.jump_strength,
            wall_jump_strength:         settings.wall_jump_strength,
            decr_jump_strength:         settings.decr_jump_strength,
            min_jump_velocity:          settings.min_jump_velocity,
            max_velocity:               settings.max_velocity,
            gravity:                    settings.gravity,
            jump_gravity:               settings.jump_gravity,
            slide_strength:             settings.slide_strength,
            quick_turnaround:           settings.quick_turnaround,
            air_quick_turnaround:       settings.air_quick_turnaround,
            decrease_x_velocity_in_air: settings.decrease_x_velocity_in_air,
            health:                     settings.health,
            max_health:                 settings.max_health,
            damage:                     settings.damage,
            is_attacking:               false,
            in_control:                 false,
            items_data:                 ItemsData::default(),
        }
    }

    pub fn deal_damage_to(&self, enemy: &mut Enemy) {
        enemy.take_damage(self.damage);
    }

    pub fn take_damage(&mut self, damage: u32) {
        if (self.health as i32) - (damage as i32) >= 0 {
            self.health -= damage;
        } else {
            self.health = 0;
        }
    }

    pub fn add_health(&mut self, health: u32) {
        self.health = (self.max_health).min(self.health + health);
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }

    pub fn has_extra_jump(&self) -> bool {
        self.items_data.extra_jump.used_extra_jumps
            < self.items_data.extra_jump.extra_jumps
    }

    pub fn has_dash(&self) -> bool {
        self.items_data.dash.used_dashes < self.items_data.dash.dashes
    }

    // Reset ExtraJumps and Dashes
    pub fn reset_jumps(&mut self) {
        if self.items_data.extra_jump.used_extra_jumps != 0 {
            self.items_data.extra_jump.used_extra_jumps = 0;
        }
        if self.items_data.dash.used_dashes != 0
            && !self.items_data.dash.is_dashing
        {
            self.items_data.dash.used_dashes = 0;
        }
    }

    // Reset ExtraJumps and Dashes when sliding on wall
    pub fn reset_jumps_touching_vertically(&mut self) {
        if self.items_data.wall_jump.can_wall_jump {
            self.reset_jumps();
        }
    }
}

impl Component for Player {
    type Storage = HashMapStorage<Self>;
}

impl Health for Player {
    fn health(&self) -> u32 {
        self.health
    }

    fn health_mut(&mut self) -> &mut u32 {
        &mut self.health
    }

    fn take_damage(&mut self, damage: u32) {
        self.take_damage(damage);
    }
}
