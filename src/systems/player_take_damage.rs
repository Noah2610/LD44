use super::system_prelude::*;

pub struct PlayerTakeDamageSystem;

impl<'a> System<'a> for PlayerTakeDamageSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Collision>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Flipped>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Velocity>,
    );

    fn run(
        &mut self,
        (entities, collisions, enemies, flippeds, mut players, mut velocities): Self::SystemData,
    ) {
        for (player, player_collision, player_velocity, player_flipped) in
            (&mut players, &collisions, &mut velocities, &flippeds).join()
        {
            for (enemy_entity, enemy) in (&entities, &enemies).join() {
                let enemy_id = enemy_entity.id();

                if let Some(collision::Data {
                    side,
                    state: collision::State::Enter,
                    ..
                }) = player_collision.collision_with(enemy_id)
                {
                    // Take damage
                    enemy.deal_damage_to(player);
                    // Knockback
                    let knockback = match side {
                        Side::Left => (enemy.knockback.0, enemy.knockback.1),
                        Side::Right => {
                            (enemy.knockback.0 * -1.0, enemy.knockback.1)
                        }
                        Side::Top | Side::Inner => (
                            x_knockback_for_vertical_side(
                                enemy.knockback.0,
                                player_flipped,
                            ),
                            enemy.knockback.1,
                        ),
                        Side::Bottom => (
                            x_knockback_for_vertical_side(
                                enemy.knockback.0,
                                player_flipped,
                            ),
                            enemy.knockback.1 * -1.0,
                        ),
                    };
                    player_velocity.x += knockback.0;
                    player_velocity.y += knockback.1;
                }
            }
        }
    }
}

fn x_knockback_for_vertical_side(knockback: f32, flipped: &Flipped) -> f32 {
    if let Flipped::None = flipped {
        knockback * -1.0
    } else {
        knockback
    }
}
