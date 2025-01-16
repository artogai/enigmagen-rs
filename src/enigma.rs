use enigma_simulator::{EnigmaBuilder, EnigmaMachine};

pub const MAX_ROTOR_NUM: u8 = 6;
pub const MAX_RING_SETTINGS_NUM: u8 = 26;
pub const MAX_ROTOR_POSITIONS_NUM: u8 = 26;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Settings {
    pub rotors: (u8, u8, u8),
    pub ring_settings: (u8, u8, u8),
    pub rotor_positions: (u8, u8, u8),
}

pub struct Machine {
    internal: EnigmaMachine,
}

impl Machine {
    pub fn new(s: &Settings) -> anyhow::Result<Self> {
        Ok(Self {
            internal: EnigmaMachine::new()
                .reflector("B")
                .rotors(s.rotors.0, s.rotors.1, s.rotors.2)
                .ring_positions(
                    s.rotor_positions.0,
                    s.rotor_positions.1,
                    s.rotor_positions.2,
                )
                .ring_settings(s.ring_settings.0, s.ring_settings.1, s.ring_settings.2)?,
        })
    }

    pub fn decrypt(&self, text: &str) -> String {
        self.internal.decrypt(text)
    }

    pub fn encrypt(&self, text: &str) -> String {
        self.internal.encrypt(text)
    }
}
