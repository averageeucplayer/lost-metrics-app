use chrono::Utc;
use rand::{rng, Rng};
use uuid::Uuid;

use crate::models::{Boss, Encounter, Player};

pub struct FakeEncounter {
    encounter: Encounter
}

impl FakeEncounter {
    pub fn new() -> Self {
        let participants = vec![
            Player {
                id: 1,
                name: "Berserker".into(),
                class_id: 102,
                class_name: "Berserker".into(),
                ..Default::default()
            },
            Player {
                id: 2,
                name: "Wildsoul".into(),
                class_id: 604,
                class_name: "Wildsoul".into(),
                ..Default::default()
            },
            Player {
                id: 3,
                name: "Aeromancer".into(),
                class_id: 603,
                class_name: "Aeromancer".into(),
                ..Default::default()
            },
            Player {
                id: 4,
                name: "Bard".into(),
                class_id: 204,
                class_name: "Bard".into(),
                ..Default::default()
            },
            Player {
                id: 5,
                name: "Slayer".into(),
                class_id: 112,
                class_name: "Slayer".into(),
                ..Default::default()
            },
            Player {
                id: 6,
                name: "Mage".into(),
                class_id: 201,
                class_name: "Mage".into(),
                ..Default::default()
            },
            Player {
                id: 7,
                name: "Deadeye".into(),
                class_id: 603,
                class_name: "Deadeye".into(),
                ..Default::default()
            },
            Player {
                id: 8,
                name: "Paladin".into(),
                class_id: 503,
                class_name: "Paladin".into(),
                ..Default::default()
            }
        ];

        let encounter = Encounter {
            id: Uuid::now_v7(),
            updated_on: Utc::now(),
            total_damage: 0.into(),
            participants,
            boss: Boss {
                id: 1,
                name: "Narok the Butcher".into()
            }
        };

        Self {
            encounter
        }
    }

    pub fn tick(&mut self) {

        let mut rng = rng();
        let index = rng.random_range(1..8);

        let min_damage = 1e6 as u64;
        let max_damage = 1e8 as u64;
        let damage = rng.random_range(min_damage..max_damage);
        
        let participant = &mut self.encounter.participants[index];
        participant.stats.total_damage += damage;

        self.encounter.total_damage += damage;

    }

    pub fn get(&self) -> &Encounter {
        &self.encounter
    }
}