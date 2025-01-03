use enigma_simulator::{EnigmaBuilder, EnigmaMachine};

#[derive(PartialEq, Debug, Clone)]
pub struct Settings {
    pub rotors: (u8, u8, u8),
    pub ring_settings: (u8, u8, u8),
    pub ring_positions: (u8, u8, u8),
    pub plugboard: Vec<(char, char)>,
}

pub struct Machine {
    internal: EnigmaMachine,
}

impl Machine {
    pub fn new(s: &Settings) -> anyhow::Result<Self> {
        let plugboard = s
            .plugboard
            .iter()
            .map(|(l, r)| format!("{}{}", l, r))
            .collect::<Vec<_>>()
            .join(" ");

        Ok(Self {
            internal: EnigmaMachine::new()
                .reflector("B")
                .rotors(s.rotors.0, s.rotors.1, s.rotors.2)
                .ring_positions(s.ring_positions.0, s.ring_positions.1, s.ring_positions.2)
                .ring_settings(s.ring_settings.0, s.ring_settings.1, s.ring_settings.2)
                .plugboard(&plugboard)?,
        })
    }

    pub fn decrypt(&self, text: &str) -> String {
        self.internal.decrypt(text)
    }

    pub fn encrypt(&self, text: &str) -> String {
        self.internal.encrypt(text)
    }
}
