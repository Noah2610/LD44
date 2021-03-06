mod charger;
mod tracer;
mod turret;

use deathframe::geo::Vector;

use super::system_prelude::*;
use crate::settings::prelude::*;

struct PlayerData<'a> {
    pub player: &'a mut Player,
    pub pos:    Vector,
}

pub struct EnemyAiSystem;

impl<'a> System<'a> for EnemyAiSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Settings>,
        Read<'a, Time>,
        Read<'a, CurrentLevelName>,
        Write<'a, World>,
        Write<'a, BulletCreator>,
        Write<'a, Stats>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Collision>,
        ReadStorage<'a, Solid<SolidTag>>,
        ReadStorage<'a, Invincible>,
        ReadStorage<'a, Loadable>,
        ReadStorage<'a, Loaded>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, EnemyAi>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Flipped>,
        WriteStorage<'a, DecreaseVelocity>,
        // WriteStorage<'a, MaxVelocity>,
        WriteStorage<'a, AnimationsContainer>,
    );

    fn run(
        &mut self,
        (
            entities,
            settings,
            time,
            current_level_name,
            mut world,
            mut bullet_creator,
            mut stats,
            transforms,
            collisions,
            solids,
            invincibles,
            loadables,
            loadeds,
            mut enemies,
            mut enemy_ais,
            mut players,
            mut velocities,
            mut flippeds,
            mut decrease_velocities,
            // mut max_velocities,
            mut animations_containers,
        ): Self::SystemData,
    ) {
        if let Some(player_data) = (&mut players, &transforms).join().find_map(
            |(player, transform)| {
                if player.in_control {
                    Some(PlayerData {
                        player,
                        pos: transform.into(),
                    })
                } else {
                    None
                }
            },
        ) {
            let dt = time.delta_seconds();
            // Call `world.maintain()` later if any enemies were deleted.
            let mut call_world_maintain = false;

            for (
                enemy_entity,
                enemy,
                enemy_ai,
                enemy_transform,
                mut enemy_velocity_opt,
                enemy_flipped,
                mut enemy_decr_vel_opt,
                // enemy_max_vel,
                enemy_animations_container,
                mut enemy_collision_opt,
                mut enemy_solid_opt,
                enemy_invincible_opt,
                enemy_loadable_opt,
                enemy_loaded_opt,
            ) in (
                &entities,
                &mut enemies,
                &mut enemy_ais,
                &transforms,
                (&mut velocities).maybe(),
                &mut flippeds,
                (&mut decrease_velocities).maybe(),
                // &mut max_velocities,
                &mut animations_containers,
                (&collisions).maybe(),
                (&solids).maybe(),
                invincibles.maybe(),
                loadables.maybe(),
                loadeds.maybe(),
            )
                .join()
            {
                if let (Some(_), Some(_)) | (None, None) =
                    (enemy_loadable_opt, enemy_loaded_opt)
                {
                    let sides_touching_opt =
                        if let (Some(enemy_collision), Some(enemy_solid)) =
                            (enemy_collision_opt, enemy_solid_opt)
                        {
                            Some(SidesTouching::new(
                                &entities,
                                enemy_collision,
                                enemy_solid,
                                &collisions,
                                &solids,
                            ))
                        } else {
                            None
                        };

                    // Run AI specific code
                    match enemy_ai {
                        EnemyAi::Tracer => tracer::run(
                            dt,
                            &player_data,
                            enemy,
                            enemy_transform,
                            enemy_velocity_opt
                                .as_mut()
                                .expect("Enemy Tracer AI needs Velocity"),
                            enemy_decr_vel_opt.as_mut().expect(
                                "Enemy Tracer AI needs DecreaseVelocity",
                            ),
                        ),
                        EnemyAi::Charger(data) => charger::run(
                            dt,
                            &player_data,
                            enemy,
                            data,
                            enemy_transform,
                            enemy_velocity_opt
                                .as_mut()
                                .expect("Enemy Charger AI needs Veclocity"),
                            enemy_decr_vel_opt.as_mut().expect(
                                "Enemy Charger AI needs DecreaseVelocity",
                            ),
                            enemy_collision_opt
                                .as_mut()
                                .expect("Enemy Charger AI needs Collision"),
                            enemy_solid_opt
                                .as_mut()
                                .expect("Enemy Charger AI needs Solid"),
                            &entities,
                            &solids,
                        ),
                        EnemyAi::Turret(data) => turret::run(
                            dt,
                            &player_data,
                            enemy,
                            data,
                            enemy_transform,
                            enemy_animations_container,
                            &mut bullet_creator,
                        ),
                    }

                    // Reset velocity when enemy is touching a solid
                    if let Some(enemy_velocity) = enemy_velocity_opt.as_mut() {
                        if let Some(sides_touching) =
                            sides_touching_opt.as_ref()
                        {
                            if (sides_touching.is_touching_left
                                && enemy_velocity.x < 0.0)
                                || (sides_touching.is_touching_right
                                    && enemy_velocity.x > 0.0)
                            {
                                enemy_velocity.x = 0.0;
                            }
                            if (sides_touching.is_touching_bottom
                                && enemy_velocity.y < 0.0)
                                || (sides_touching.is_touching_top
                                    && enemy_velocity.y > 0.0)
                            {
                                enemy_velocity.y = 0.0;
                            }
                        }

                        // Flip sprite
                        if enemy_velocity.x > 0.0 {
                            if enemy_flipped == &mut Flipped::Horizontal {
                                *enemy_flipped = Flipped::None;
                            }
                            enemy_animations_container.set_if_has("walking");
                        } else if enemy_velocity.x < 0.0 {
                            if enemy_flipped == &mut Flipped::None {
                                *enemy_flipped = Flipped::Horizontal;
                            }
                            enemy_animations_container.set_if_has("walking");
                        } else {
                            enemy_animations_container.set_if_has("idle");
                        }
                    }

                    // Kill the enemies when they fall below the death_floor
                    if enemy_transform.translation().y < settings.death_floor {
                        enemy.health = 0;
                    }

                    // Handle enemy death
                    if enemy_invincible_opt.is_none() && enemy.is_dead() {
                        call_world_maintain = true;
                        player_data.player.add_health(enemy.reward);
                        entities.delete(enemy_entity).unwrap();
                        // Increase stats kill count
                        if let Some(level) = current_level_name.0.as_ref() {
                            stats
                                .level_mut(level)
                                .kills
                                .increase_for(&enemy.enemy_type);
                        }
                    }
                }
            }

            if call_world_maintain {
                world.maintain();
            }
        }
    }
}
