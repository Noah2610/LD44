use std::time::Instant;

use deathframe::geo::Vector;

use super::system_prelude::*;

pub struct PlayerAttackSystem;

impl<'a> System<'a> for PlayerAttackSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Collision>,
        ReadStorage<'a, NoAttack>,
        ReadStorage<'a, Invincible>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, PlayerAttack>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, AnimationsContainer>,
        WriteStorage<'a, Flipped>,
        WriteStorage<'a, Hidden>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Bullet>,
    );

    fn run(
        &mut self,
        (
            entities,
            sizes,
            collisions,
            no_attacks,
            invincibles,
            mut players,
            mut player_attacks,
            mut transforms,
            mut velocities,
            mut animations_containers,
            mut flippeds,
            mut hiddens,
            mut enemies,
            mut bullets,
        ): Self::SystemData,
    ) {
        // Get some player data
        let player_data_opt: Option<(_, _, Vector, _)> =
            (&mut players, &transforms, &sizes, &flippeds)
                .join()
                .find_map(|(player, transform, size, flipped)| {
                    Some((
                        player.is_attacking,
                        Vector::from(transform),
                        size.into(),
                        flipped.clone(),
                    ))
                });

        // Deactivate PlayerAttack if neccessary
        for (player, animations_container) in
            (&mut players, &animations_containers).join()
        {
            if animations_container.play_once.is_none() {
                player.is_attacking = false;
            }
        }

        // PlayerAttack logic
        if let Some((is_attacking, player_pos, player_size, player_flipped)) =
            player_data_opt
        {
            let mut attack_id_opt = None;

            for (
                attack_entity,
                attack,
                attack_transform,
                attack_animations_container,
                attack_flipped,
            ) in (
                &entities,
                &mut player_attacks,
                &mut transforms,
                &mut animations_containers,
                &mut flippeds,
            )
                .join()
            {
                attack_id_opt = Some(attack_entity.id());

                if is_attacking {
                    // Move PlayerAttack's transform
                    let mut pos = player_pos.clone();
                    match player_flipped {
                        Flipped::None => {
                            pos.0 += player_size.0;
                        }
                        Flipped::Horizontal => {
                            pos.0 -= player_size.0;
                        }
                        _ => (),
                    }
                    if *attack_flipped != player_flipped {
                        *attack_flipped = player_flipped.clone();
                    }
                    attack_transform.set_x(pos.0);
                    attack_transform.set_y(pos.1);
                    attack.active = true;
                    attack_animations_container.play("attack_default");
                    hiddens.remove(attack_entity);
                } else {
                    // Hacky: move PlayerAttack way off screen, so collision data is unset
                    attack_transform.set_x(-1000.0);
                    attack_transform.set_y(-1000.0);
                    attack.active = false;
                    attack_animations_container.play_once = None;
                    hiddens.insert(attack_entity, Hidden).unwrap();
                }
            }

            // Actual attacking logic
            if attack_id_opt.is_some() {
                let now = Instant::now();

                for (player, _) in (&mut players, !&no_attacks).join() {
                    for (attack, attack_collision, player_flipped) in
                        (&player_attacks, &collisions, &flippeds).join()
                    {
                        if attack.active {
                            for (enemy_entity, enemy, enemy_velocity, _) in (
                                &entities,
                                &mut enemies,
                                &mut velocities,
                                !&invincibles,
                            )
                                .join()
                            {
                                let enemy_id = enemy_entity.id();
                                // Attack enemy
                                if let Some(collision::Data {
                                    state: collision::State::Enter,
                                    ..
                                }) =
                                    attack_collision.collision_with(enemy_id)
                                {
                                    player.deal_damage_to(enemy);
                                    // Knockback
                                    if enemy.affected_by_knockback
                                        && player
                                            .items_data
                                            .knockback
                                            .has_knockback
                                    {
                                        enemy_velocity.x = player
                                            .items_data
                                            .knockback
                                            .velocity
                                            .0
                                            * match player_flipped {
                                                Flipped::None => 1.0,
                                                Flipped::Horizontal => -1.0,
                                                _ => 1.0,
                                            };
                                        enemy_velocity.y = player
                                            .items_data
                                            .knockback
                                            .velocity
                                            .1;
                                    }
                                }
                            }

                            // BulletDeflect
                            if player.items_data.bullet_deflect.can_deflect {
                                for (bullet_entity, bullet, bullet_velocity) in
                                    (&entities, &mut bullets, &mut velocities)
                                        .join()
                                {
                                    let bullet_id = bullet_entity.id();
                                    if let (
                                        &BulletOwner::Enemy,
                                        Some(collision::Data {
                                            state: collision::State::Enter,
                                            ..
                                        }),
                                    ) = (
                                        &bullet.owner,
                                        attack_collision
                                            .collision_with(bullet_id),
                                    ) {
                                        // Deflect bullet
                                        let bullet_data =
                                            &player.items_data.bullet_deflect;
                                        bullet.owner = BulletOwner::Player;
                                        bullet.damage = bullet_data.damage;
                                        bullet_velocity.x = bullet_velocity.x
                                            * bullet_data.velocity_mult.0;
                                        bullet_velocity.y = bullet_velocity.y
                                            * bullet_data.velocity_mult.1;
                                        bullet.lifetime = bullet_data.lifetime;
                                        bullet.created_at = now;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
