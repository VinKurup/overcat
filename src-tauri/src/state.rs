use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Unix time in whole seconds — the basis for elapsed-time decay.
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Personality {
    Playful, // happiness falls fast — needs enrichment/attention
    Foodie,  // satiety falls fast — always hungry
    Lazy,    // energy falls fast, happiness falls slow — content to lounge
    Grumpy,  // happiness falls fast & harder to please
    Clingy,  // happiness falls fast when left alone
    Chaotic, // uniform rates for now (RNG-driven swings deferred)
}

impl Personality {
    fn from_seed(seed: i64) -> Self {
        match seed.rem_euclid(6) {
            0 => Personality::Playful,
            1 => Personality::Foodie,
            2 => Personality::Lazy,
            3 => Personality::Grumpy,
            4 => Personality::Clingy,
            _ => Personality::Chaotic,
        }
    }

    fn rate_multipliers(self) -> (f32, f32, f32) {
        match self {
            Personality::Playful => (1.0, 1.8, 1.0),
            Personality::Foodie => (1.8, 1.0, 1.0),
            Personality::Lazy => (1.0, 0.6, 1.6),
            Personality::Grumpy => (1.0, 1.6, 1.0),
            Personality::Clingy => (1.0, 1.7, 1.0),
            Personality::Chaotic => (1.0, 1.0, 1.0),
        }
    }
}

/// The cat's current state of being — *derived* from its stats, never stored.
/// The frontend animator maps this to which sprite to play.
#[derive(Serialize, Clone, Copy, Debug)]
pub enum Mood {
    Idle,
    Sad,
    Sick,
    Sleepy,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CatState {
    pub name: String,
    pub personality: Personality,
    pub satiety: u8,  
    pub happiness: u8, 
    pub energy: u8, 
    pub born_at: i64,  
    pub last_updated: i64,
}

const BASE_SATIETY_PER_HOUR: f32 = 8.0;
const BASE_HAPPINESS_PER_HOUR: f32 = 5.0;
const BASE_ENERGY_PER_HOUR: f32 = 6.0;

impl CatState {
    pub fn new() -> Self {
        let t = now();
        CatState {
            name: "Cat".to_string(),
            personality: Personality::from_seed(t),
            satiety: 80,
            happiness: 80,
            energy: 80,
            born_at: t,
            last_updated: t,
        }
    }

    pub fn apply_decay(&mut self, now: i64) {
        let elapsed = (now - self.last_updated).max(0);
        let hours = elapsed as f32 / 3600.0;
        let (m_sat, m_hap, m_ene) = self.personality.rate_multipliers();

        self.satiety = decayed(self.satiety, BASE_SATIETY_PER_HOUR * m_sat * hours);
        self.happiness = decayed(self.happiness, BASE_HAPPINESS_PER_HOUR * m_hap * hours);
        self.energy = decayed(self.energy, BASE_ENERGY_PER_HOUR * m_ene * hours);
        self.last_updated = now;
    }

    pub fn mood(&self) -> Mood {
        if self.satiety <= 10 || self.happiness <= 10 {
            Mood::Sick
        } else if self.energy <= 15 {
            Mood::Sleepy
        } else if self.satiety <= 30 || self.happiness <= 30 {
            Mood::Sad
        } else {
            Mood::Idle
        }
    }
}

fn decayed(current: u8, amount: f32) -> u8 {
    (current as f32 - amount).clamp(0.0, 100.0).round() as u8
}

fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("state.json"))
}

pub fn load(app: &AppHandle) -> CatState {
    let Ok(path) = state_path(app) else {
        return CatState::new();
    };
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|_| CatState::new()),
        Err(_) => CatState::new(),
    }
}

pub fn save(app: &AppHandle, state: &CatState) -> Result<(), String> {
    let path = state_path(app)?;
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic cat for tests: known stats, known birth time.
    fn fixture() -> CatState {
        CatState {
            name: "Test".into(),
            personality: Personality::Chaotic, // uniform multipliers
            satiety: 80,
            happiness: 80,
            energy: 80,
            born_at: 0,
            last_updated: 0,
        }
    }

    #[test]
    fn decay_reduces_stats_over_elapsed_time() {
        let mut cat = fixture();
        cat.apply_decay(3600);
        // Chaotic = base rates: satiety -8, happiness -5, energy -6.
        assert_eq!(cat.satiety, 72);
        assert_eq!(cat.happiness, 75);
        assert_eq!(cat.energy, 74);
        assert_eq!(cat.last_updated, 3600);
    }

    #[test]
    fn decay_floors_at_zero_never_negative() {
        let mut cat = fixture();
        cat.apply_decay(1_000_000);
        assert_eq!(cat.satiety, 0);
        assert_eq!(cat.happiness, 0);
        assert_eq!(cat.energy, 0);
    }

    #[test]
    fn personality_bends_decay_rates() {
        let mut foodie = fixture();
        foodie.personality = Personality::Foodie;
        foodie.apply_decay(3600);

        let mut chaotic = fixture();
        chaotic.apply_decay(3600);

        assert!(foodie.satiety < chaotic.satiety);
    }

    #[test]
    fn mood_reflects_critical_stats() {
        let mut cat = fixture();
        assert!(matches!(cat.mood(), Mood::Idle));

        cat.energy = 10;
        assert!(matches!(cat.mood(), Mood::Sleepy));

        cat.satiety = 5;
        assert!(matches!(cat.mood(), Mood::Sick));
    }

    #[test]
    fn serde_roundtrip_preserves_state() {
        let cat = fixture();
        let json = serde_json::to_string(&cat).unwrap();
        let restored: CatState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.satiety, cat.satiety);
        assert_eq!(restored.born_at, cat.born_at);
        assert!(matches!(restored.personality, Personality::Chaotic));
    }
}
