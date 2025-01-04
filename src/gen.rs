use std::{collections::HashMap, sync::LazyLock};

use genevo::{
    genetic::{Children, Parents},
    operator::{CrossoverOp, GeneticOperator, MutationOp},
    prelude::{FitnessFunction, GenomeBuilder, Genotype},
    random::Rng,
};
use rand::seq::{IteratorRandom, SliceRandom};

use crate::enigma::{Machine, Settings};

#[derive(Debug)]
pub struct Options {
    pub fitness_scale: usize,
    pub population_size: usize,
    pub generation_limit: u64,
    pub num_individuals_per_parents: usize,
    pub selection_ratio: f64,
    pub mutation_rate: f64,
    pub reinsertion_ratio: f64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            fitness_scale: 1000000,
            population_size: 10000,
            generation_limit: 200,
            num_individuals_per_parents: 2,
            selection_ratio: 0.5,
            mutation_rate: 0.05,
            reinsertion_ratio: 0.7,
        }
    }
}

impl Genotype for Settings {
    type Dna = u8;
}

#[derive(Debug, Clone)]
pub struct FitnessCalc {
    pub ciphertext: String,
    pub max_value: usize,
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

static PLUGS: LazyLock<Vec<(char, char)>> = LazyLock::new(|| {
    let letters: Vec<char> = ('A'..='Z').collect();

    let mut plugs = Vec::new();

    for i in 0..letters.len() {
        for j in i + 1..letters.len() {
            plugs.push((letters[i], letters[j]));
        }
    }

    plugs
});

pub struct SettingsBuilder;

impl GenomeBuilder<Settings> for SettingsBuilder {
    fn build_genome<R>(&self, _: usize, rng: &mut R) -> Settings
    where
        R: Rng + Sized,
    {
        let gen_3_from_1_to_26 = |rng: &mut R| {
            std::iter::repeat_with(|| rng.gen_range(1..=26))
                .take(3)
                .collect::<Vec<u8>>()
        };

        let rotors = (1..=6).choose_multiple(rng, 3);
        let ring_settings = gen_3_from_1_to_26(rng);
        let rotor_positions = gen_3_from_1_to_26(rng);
        let plugs_cnt = rng.gen_range(0..=10);
        let plugs = PLUGS.choose_multiple(rng, plugs_cnt).cloned().collect();

        Settings {
            rotors: (rotors[0], rotors[1], rotors[2]),
            ring_settings: (ring_settings[0], ring_settings[1], ring_settings[2]),
            rotor_positions: (rotor_positions[0], rotor_positions[1], rotor_positions[2]),
            plugboard: plugs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SettingsCrossover;

impl GeneticOperator for SettingsCrossover {
    fn name() -> String {
        "Settings-Crossover".to_string()
    }
}

impl CrossoverOp<Settings> for SettingsCrossover {
    fn crossover<R>(&self, parents: Parents<Settings>, rng: &mut R) -> Children<Settings>
    where
        R: Rng + Sized,
    {
        let num_parents = parents.len();
        let mut offsprings: Vec<Settings> = Vec::with_capacity(num_parents);

        for _ in 0..num_parents {
            let i = rng.gen_range(0..num_parents);
            let mut j = rng.gen_range(0..num_parents);

            while i == j {
                j = rng.gen_range(0..num_parents);
            }

            offsprings.push(cross(&parents[i], &parents[j]));
        }

        offsprings
    }
}

fn cross(s1: &Settings, s2: &Settings) -> Settings {
    todo!()
}

#[derive(Debug, Clone, Copy)]
pub struct SettingsMutator {
    pub mutation_rate: f64,
}

impl GeneticOperator for SettingsMutator {
    fn name() -> String {
        "Settings-Mutator".to_string()
    }
}

impl MutationOp<Settings> for SettingsMutator {
    fn mutate<R>(&self, genome: Settings, rng: &mut R) -> Settings
    where
        R: Rng + Sized,
    {
        todo!()
    }
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
            rotor_positions: (13, 3, 21),
            plugboard: vec![('A', 'B'), ('C', 'D')],
        };

        let machine = Machine::new(&settings).unwrap();
        let ciphertext = machine.encrypt(LONG_TEXT);

        let calc = FitnessCalc {
            ciphertext: ciphertext.to_owned(),
            max_value: 1000000,
        };

        let mut closer_settings = settings.clone();
        closer_settings.rotor_positions = (13, 3, 22);

        let wrong_settings = enigma::Settings {
            rotors: (1, 2, 3),
            ring_settings: (1, 1, 1),
            rotor_positions: (1, 1, 1),
            plugboard: Vec::new(),
        };

        assert_eq!(calc.fitness_of(&settings), 70347);
        assert_eq!(calc.fitness_of(&closer_settings), 39388);
        assert_eq!(calc.fitness_of(&wrong_settings), 36722);
    }

    #[test]
    fn test_genome_builder() {
        fn assert_in_range(v: (u8, u8, u8), from: u8, until: u8) {
            assert!(v.0 >= from && v.0 < until);
            assert!(v.1 >= from && v.1 < until);
            assert!(v.2 >= from && v.2 < until);
        }

        let mut rng = rand::thread_rng();
        let b = SettingsBuilder {};

        for _ in 0..100 {
            let sett = b.build_genome(0, &mut rng);

            assert_in_range(sett.rotors, 1, 7);
            assert_in_range(sett.ring_settings, 1, 27);
            assert_in_range(sett.rotor_positions, 1, 27);
            assert!(sett.plugboard.len() <= 10);

            let mut rotors = vec![sett.rotors.0, sett.rotors.1, sett.rotors.2];
            rotors.dedup();
            assert_eq!(rotors.len(), 3);

            let mut plugs = sett.plugboard.clone();
            plugs.dedup();
            assert_eq!(plugs.len(), sett.plugboard.len());
        }
    }
}
