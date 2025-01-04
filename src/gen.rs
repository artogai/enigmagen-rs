use std::collections::HashMap;

use genevo::prelude::{FitnessFunction, Genotype};

use crate::enigma::{Machine, Settings};

impl Genotype for Settings {
    type Dna = u8;
}

#[derive(Clone, Debug)]
struct FitnessCalc {
    ciphertext: String,
    max_value: usize,
}

impl FitnessFunction<Settings, usize> for FitnessCalc {
    fn fitness_of(&self, s: &Settings) -> usize {
        let machine = Machine::new(s).expect("Wrong machine settings");
        let plaintext = machine.decrypt(&self.ciphertext);
        let metric = index_of_coincidence(&plaintext);
        (metric * (self.max_value as f64)).round() as usize
    }

    fn average(&self, fitness_values: &[usize]) -> usize {
        fitness_values.iter().sum::<usize>() / fitness_values.len()
    }

    fn highest_possible_fitness(&self) -> usize {
        self.max_value
    }

    fn lowest_possible_fitness(&self) -> usize {
        0
    }
}

fn index_of_coincidence(text: &str) -> f64 {
    let filtered_text = text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_uppercase();

    let n = filtered_text.len();
    if n <= 1 {
        return 0.0;
    }

    let mut hist = HashMap::<char, usize>::new();

    filtered_text
        .chars()
        .for_each(|c| *hist.entry(c).or_insert(0) += 1);

    let numerator = hist.values().map(|freq| freq * (freq - 1)).sum::<usize>();
    let denominator = n * (n - 1);

    numerator as f64 / denominator as f64
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::enigma;

    use super::*;

    const LONG_TEXT: &str = "To be, or not to be, that is the question: \
        Whether 'tis nobler in the mind to suffer \
        The slings and arrows of outrageous fortune, \
        Or to take arms against a sea of troubles \
        And by opposing end them.";

    #[test]
    fn test_ioc() {
        assert_relative_eq!(index_of_coincidence(""), 0.0);
        assert_relative_eq!(index_of_coincidence("A"), 0.0);
        assert_relative_eq!(index_of_coincidence("AB"), 0.0);
        assert_relative_eq!(index_of_coincidence("ABAA"), 0.5);
        assert_relative_eq!(index_of_coincidence(LONG_TEXT), 0.07034743722050224);
    }

    #[test]
    fn test_fitness() {
        let settings = enigma::Settings {
            rotors: (2, 5, 3),
            ring_settings: (8, 5, 20),
            ring_positions: (13, 3, 21),
            plugboard: vec![('A', 'B'), ('C', 'D')],
        };

        let machine = Machine::new(&settings).unwrap();
        let ciphertext = machine.encrypt(LONG_TEXT);

        let calc = FitnessCalc {
            ciphertext: ciphertext.to_owned(),
            max_value: 1000000,
        };

        let mut closer_settings = settings.clone();
        closer_settings.ring_positions = (13, 3, 22);

        let wrong_settings = enigma::Settings {
            rotors: (1, 2, 3),
            ring_settings: (1, 1, 1),
            ring_positions: (1, 1, 1),
            plugboard: Vec::new(),
        };

        assert_eq!(calc.fitness_of(&settings), 70347);
        assert_eq!(calc.fitness_of(&closer_settings), 39388);
        assert_eq!(calc.fitness_of(&wrong_settings), 36722);
    }
}
