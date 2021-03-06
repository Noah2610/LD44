pub mod prelude {
    pub use super::Bullet;
    pub use super::BulletBuilder;
    pub use super::BulletOwner;
}

use std::time::{Duration, Instant};

use super::component_prelude::*;

#[derive(PartialEq)]
pub enum BulletOwner {
    Player,
    Enemy,
}

pub struct Bullet {
    pub owner:      BulletOwner,
    pub damage:     u32,
    pub created_at: Instant,
    pub lifetime:   Duration,
    pub knockback:  Option<Vector>,
    pub facing:     Option<Facing>,
}

impl Bullet {
    pub fn new() -> BulletBuilder {
        BulletBuilder::default()
    }
}

#[derive(Default)]
pub struct BulletBuilder {
    owner:     Option<BulletOwner>,
    damage:    Option<u32>,
    lifetime:  Option<Duration>,
    knockback: Option<Vector>,
    facing:    Option<Facing>,
}

impl BulletBuilder {
    pub fn owner(mut self, owner: BulletOwner) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn damage(mut self, damage: u32) -> Self {
        self.damage = Some(damage);
        self
    }

    pub fn lifetime(mut self, lifetime: Duration) -> Self {
        self.lifetime = Some(lifetime);
        self
    }

    pub fn knockback(mut self, knockback: Vector) -> Self {
        self.knockback = Some(knockback);
        self
    }

    pub fn facing(mut self, facing: Facing) -> Self {
        self.facing = Some(facing);
        self
    }

    pub fn build(self) -> Bullet {
        Bullet {
            owner:      self.owner.expect("Bullet needs an owner BulletOwner"),
            damage:     self.damage.expect("Bullet needs damage u32"),
            created_at: Instant::now(),
            lifetime:   self
                .lifetime
                .expect("Bullet needs a lifetime Duration"),
            knockback:  self.knockback,
            facing:     self.facing,
        }
    }
}

impl Component for Bullet {
    type Storage = VecStorage<Self>;
}
