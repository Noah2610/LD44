use super::system_prelude::*;
use crate::settings::prelude::*;

pub struct PlayerControlsSystem;

impl<'a> System<'a> for PlayerControlsSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Settings>,
        Read<'a, Time>,
        Read<'a, InputManager>,
        Read<'a, CurrentLevelName>,
        Write<'a, BulletCreator>,
        Write<'a, Stats>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Collision>,
        ReadStorage<'a, Solid<SolidTag>>,
        ReadStorage<'a, Goal>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, HeartsContainer>,
        ReadStorage<'a, Noclip>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, DecreaseVelocity>,
        WriteStorage<'a, Gravity>,
        WriteStorage<'a, AnimationsContainer>,
        WriteStorage<'a, Flipped>,
    );

    fn run(
        &mut self,
        (
            entities,
            settings,
            time,
            input_manager,
            current_level_name,
            mut bullet_creator,
            mut stats,
            transforms,
            collisions,
            solids,
            goals,
            items,
            hearts_containers,
            noclips,
            mut players,
            mut velocities,
            mut decr_velocities,
            mut gravities,
            mut animations_containers,
            mut flippeds,
        ): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        let goal_next_level = (&goals)
            .join()
            .next()
            .map(|goal| goal.next_level)
            .unwrap_or(false);

        for (
            player,
            transform,
            velocity,
            decr_velocity,
            gravity_opt,
            player_collision,
            player_solid,
            animations_container,
            flipped,
            noclip_opt,
        ) in (
            &mut players,
            &transforms,
            &mut velocities,
            &mut decr_velocities,
            (&mut gravities).maybe(),
            &collisions,
            &solids,
            &mut animations_containers,
            &mut flippeds,
            noclips.maybe(),
        )
            .join()
        {
            let is_noclip = noclip_opt.is_some();

            let sides_touching = SidesTouching::new(
                &entities,
                player_collision,
                player_solid,
                &collisions,
                &solids,
            );

            if !is_noclip {
                handle_wall_cling(player, velocity, &sides_touching);

                handle_on_ground_and_in_air(
                    player,
                    velocity,
                    decr_velocity,
                    animations_container,
                    &sides_touching,
                );

                // Kill the player, if they fall below the death_floor
                if transform.translation().y < settings.death_floor {
                    player.health = 0;
                }
            }

            if player.in_control && !goal_next_level {
                if !is_noclip {
                    handle_move(
                        dt,
                        &input_manager,
                        player,
                        velocity,
                        decr_velocity,
                        animations_container,
                        flipped,
                        &sides_touching,
                    );

                    if let Some(gravity) = gravity_opt {
                        handle_jump(
                            &input_manager,
                            player,
                            velocity,
                            gravity,
                            &sides_touching,
                        );
                    }
                }

                handle_attack(
                    &input_manager,
                    player,
                    transform,
                    velocity,
                    animations_container,
                    flipped,
                    &mut bullet_creator,
                );

                handle_item_purchase(
                    &settings.items,
                    &entities,
                    &current_level_name,
                    &mut stats,
                    &input_manager,
                    player,
                    player_collision,
                    &items,
                    &collisions,
                    &hearts_containers,
                );
            } else if !player.in_control && !goal_next_level {
                // Start of a level
                // Play the level_start animation once, then regain control
                // TODO: Cleanup
                // animations_container.set("level_start");
                // if animations_container
                //     .current
                //     .as_ref()
                //     .map(|(_, anim)| anim.has_played())
                //     .unwrap_or(true)
                if true {
                    player.in_control = true;
                    animations_container.set("idle");
                }
            }
        }
    }
}

fn handle_wall_cling(
    player: &mut Player,
    velocity: &mut Velocity,
    sides_touching: &SidesTouching,
) {
    if sides_touching.is_touching_horizontally() {
        // Reset x velocity to 0 when colliding with a solid, horizontally.
        if (sides_touching.is_touching_left && velocity.x < 0.0)
            || (sides_touching.is_touching_right && velocity.x > 0.0)
        {
            velocity.x = 0.0;
        }
        // Clinging to wall, when not touching a solid vertically.
        if !sides_touching.is_touching_vertically() {
            // Keep y velocity at a constant velocity; slide down solid.
            let slide_strength = -player.slide_strength;
            if velocity.y < slide_strength {
                velocity.y = slide_strength;
            }
            // Reset ExtraJumps and Dashes
            player.reset_jumps_touching_vertically();
        }
    }
}

fn handle_move(
    dt: f32,
    input_manager: &InputManager,
    player: &Player,
    velocity: &mut Velocity,
    decr_velocity: &mut DecreaseVelocity,
    animations_container: &mut AnimationsContainer,
    flipped: &mut Flipped,
    sides_touching: &SidesTouching,
) {
    const PLAYER_X_AXIS_ID_PREFIX: &str = "player_x";

    if let Some(x) = input_manager.axis_value_find(|(id, &value)| {
        id.starts_with(PLAYER_X_AXIS_ID_PREFIX) && value != 0.0
    }) {
        use crate::settings::SettingsPlayerQuickTurnaround as QTA;

        let x = x as f32;
        if x != 0.0 {
            let x_sign = x.signum();
            let on_ground = sides_touching.is_touching_bottom;

            // Turnaround stuff
            let turned_around = x_sign != velocity.x.signum();
            if turned_around {
                // Quick turnaround, when on ground
                let qta_setting = if on_ground {
                    &player.quick_turnaround
                // Quick turnaround, when in air
                } else {
                    &player.air_quick_turnaround
                };
                match &qta_setting {
                    QTA::ResetVelocity => velocity.x = 0.0,
                    QTA::InvertVelocity => velocity.x *= -1.0,
                    _ => (),
                }
            }

            // Move player
            let velocity_increase = (if on_ground {
                player.acceleration.0
            } else {
                player.air_acceleration.0
            } * dt)
                * x; // TODO: Maybe don't use the sign? Might work well with controller axis inputs.
                     // * x_sign; // TODO: Maybe don't use the sign? Might work well with controller axis inputs.

            // Increase velocity with a maximum
            velocity
                .increase_x_with_max(velocity_increase, player.max_velocity.0);

            // Don't decrease velocity when moving
            if x > 0.0
                && player
                    .max_velocity
                    .0
                    .map(|max| velocity.x <= max)
                    .unwrap_or(true)
            {
                decr_velocity.dont_decrease_x_when_pos();
            } else if x < 0.0
                && player
                    .max_velocity
                    .0
                    .map(|max| velocity.x >= -max)
                    .unwrap_or(true)
            {
                decr_velocity.dont_decrease_x_when_neg();
            }

            // Flip animation
            if !player.is_attacking {
                if flipped == &Flipped::Horizontal && x > 0.0 {
                    *flipped = Flipped::None;
                } else if flipped == &Flipped::None && x < 0.0 {
                    *flipped = Flipped::Horizontal;
                }
            }
        }
    }

    // Is standing on solid
    if sides_touching.is_touching_bottom {
        if velocity.x == 0.0 {
            animations_container.set("idle");
        } else {
            animations_container.set("walking");
        }
    }
}

fn handle_jump(
    input_manager: &InputManager,
    player: &mut Player,
    velocity: &mut Velocity,
    gravity: &mut Gravity,
    sides_touching: &SidesTouching,
) {
    let jump_btn_down = input_manager.is_down("player_jump");
    let can_wall_jump = player.items_data.wall_jump.can_wall_jump
        && jump_btn_down
        && sides_touching.is_touching_horizontally()
        && !sides_touching.is_touching_bottom;
    let can_jump = jump_btn_down
        && (sides_touching.is_touching_bottom || player.has_extra_jump())
        && !can_wall_jump;
    let mut jumped = false;
    if can_jump {
        if velocity.y < 0.0 {
            velocity.y = 0.0;
        }
        // Jump
        velocity.y += player.jump_strength;
        // Was an extra jump
        if !sides_touching.is_touching_bottom {
            player.items_data.extra_jump.used_extra_jumps += 1;
        }
        jumped = true;
    } else if can_wall_jump {
        if velocity.y < 0.0 {
            velocity.y = 0.0;
        }
        // Wall jump
        velocity.y += player.wall_jump_strength.1;
        if sides_touching.is_touching_left {
            velocity.x += player.wall_jump_strength.0;
        } else if sides_touching.is_touching_right {
            velocity.x -= player.wall_jump_strength.0;
        }
        jumped = true;
    }

    if jumped {
        // Set different gravity when jumping
        gravity.x = player.jump_gravity.0;
        gravity.y = player.jump_gravity.1;
    } else if input_manager.is_up("player_jump") {
        // Kill some of the upwards momentum, keeping at least a certain minimum velocity
        if velocity.y > player.decr_jump_strength {
            velocity.y = (velocity.y - player.decr_jump_strength)
                .max(player.min_jump_velocity);
        }
        // Set default gravity
        gravity.x = player.gravity.0;
        gravity.y = player.gravity.1;
    }
}

/// Handle some specifics when player is standing on solid ground vs when in air.
fn handle_on_ground_and_in_air(
    player: &mut Player,
    velocity: &mut Velocity,
    decr_velocity: &mut DecreaseVelocity,
    animations_container: &mut AnimationsContainer,
    sides_touching: &SidesTouching,
) {
    // Reset y velocity to 0 when standing on solid ground
    // or when hitting a solid ceiling.
    if (sides_touching.is_touching_bottom && velocity.y < 0.0)
        || (sides_touching.is_touching_top && velocity.y > 0.0)
    {
        velocity.y = 0.0;
    }
    // Player is mid-air
    if !sides_touching.is_touching_bottom {
        // Switch to mid-air animation
        animations_container.set("mid_air");
        // Don't decrease velocity when in air.
        if !player.decrease_x_velocity_in_air {
            decr_velocity.dont_decrease_x();
        }
    }
    if sides_touching.is_touching_bottom {
        // Reset ExtraJumps and Dashes
        player.reset_jumps();
    }
}

/// Returns `true` if the player started an attack
fn handle_attack<'a>(
    input_manager: &InputManager,
    player: &mut Player,
    player_transform: &Transform,
    player_velocity: &mut Velocity,
    animations_container: &mut AnimationsContainer,
    flipped: &mut Flipped,
    bullet_creator: &mut BulletCreator,
) {
    let is_attacking = if !player.is_attacking {
        if input_manager.is_down("player_attack") {
            true
        } else if input_manager.is_down("player_attack_left") {
            *flipped = Flipped::Horizontal;
            true
        } else if input_manager.is_down("player_attack_right") {
            *flipped = Flipped::None;
            true
        } else {
            false
        }
    } else {
        false
    };

    if is_attacking {
        player.is_attacking = true;
        // Play attack animation
        animations_container.play("attack");
        // Thrust
        if player.items_data.thrust.can_thrust {
            let thrust = (
                player.items_data.thrust.strength.0
                    * match flipped {
                        Flipped::None => 1.0,
                        Flipped::Horizontal => -1.0,
                        _ => 1.0,
                    },
                player.items_data.thrust.strength.1,
            );
            player_velocity.x += thrust.0;
            player_velocity.y += thrust.1;
        }
    }

    let should_shoot_bullet =
        is_attacking && player.items_data.bullet_shoot.can_shoot;

    if should_shoot_bullet {
        bullet_creator.push(BulletComponents {
            bullet:    Bullet::new()
                .owner(BulletOwner::Player)
                .damage(player.items_data.bullet_shoot.damage)
                .lifetime(player.items_data.bullet_shoot.lifetime)
                .knockback(player.items_data.knockback.velocity)
                .facing(match flipped {
                    Flipped::None => Facing::Right,
                    Flipped::Horizontal => Facing::Left,
                    _ => Facing::Right,
                })
                .build(),
            transform: {
                let pos = player_transform.translation();
                let mut transform = Transform::default();
                transform.set_xyz(pos.x, pos.y, pos.z);
                transform
            },
            velocity:  Velocity::new(
                player.items_data.bullet_shoot.velocity.0
                    * match flipped {
                        Flipped::None => 1.0,
                        Flipped::Horizontal => -1.0,
                        _ => 1.0,
                    },
                player.items_data.bullet_shoot.velocity.1,
            ),
            size:      Size::from(player.items_data.bullet_shoot.size),
        });
    }
}

fn handle_item_purchase(
    settings: &SettingsItems,
    entities: &Entities,
    current_level_name: &CurrentLevelName,
    stats: &mut Stats,
    input_manager: &InputManager,
    player: &mut Player,
    player_collision: &Collision,
    items: &ReadStorage<Item>,
    collisions: &ReadStorage<Collision>,
    hearts_containers: &ReadStorage<HeartsContainer>,
) {
    for (item_entity, item, _, hearts_container_opt) in
        (entities, items, collisions, hearts_containers.maybe()).join()
    {
        let item_id = item_entity.id();
        if let Some(collision::Data {
            state: collision::State::Steady,
            ..
        }) = player_collision.collision_with(item_id)
        {
            if item.cost < player.health
                && input_manager.is_down("player_buy_item")
            {
                // Pickup item
                item.apply(player, settings);
                entities.delete(item_entity).unwrap();
                player.take_damage(item.cost);
                // Remove hearts
                if let Some(hearts_container) = hearts_container_opt {
                    for id in hearts_container.heart_ids.iter() {
                        entities.delete(entities.entity(*id)).unwrap();
                    }
                }
                // Increase stats items bought
                if let Some(level) = current_level_name.0.as_ref() {
                    stats.level_mut(level).items_bought.increase();
                }
            }
        }
    }
}
