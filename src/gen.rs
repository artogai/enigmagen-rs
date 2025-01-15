use std::sync::Arc;

use genevo::{
    genetic::{Children, Parents},
    operator::{CrossoverOp, GeneticOperator, MutationOp},
    prelude::{FitnessFunction, GenomeBuilder, Genotype},
    random::Rng,
};
use rand::{distributions, prelude::Distribution, seq::IteratorRandom};

use crate::enigma::{
    Machine, Settings, MAX_RING_SETTINGS_NUM, MAX_ROTOR_NUM, MAX_ROTOR_POSITIONS_NUM,
};

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
            generation_limit: 100,
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
    pub ciphertext: Arc<String>,
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
    debug_assert!(
        text.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_whitespace()),
        "only A..Z and whitespace are supported"
    );

    let n = text.len();
    if n <= 1 {
        return 0.0;
    }

    let mut hist = [0; 26];

    text.chars()
        .filter(|c| c.is_ascii_uppercase())
        .for_each(|c| {
            let idx = c as usize - 'A' as usize;
            hist[idx] += 1;
        });

    let numerator = hist
        .into_iter()
        .filter(|&freq| freq > 0)
        .map(|freq| freq * (freq - 1))
        .sum::<usize>();

    let denominator = n * (n - 1);

    numerator as f64 / denominator as f64
}

pub struct SettingsBuilder;

impl GenomeBuilder<Settings> for SettingsBuilder {
    fn build_genome<R>(&self, _: usize, rng: &mut R) -> Settings
    where
        R: Rng + Sized,
    {
        Settings {
            rotors: gen_triple_unique(1, MAX_ROTOR_NUM, rng),
            ring_settings: gen_triple(1, MAX_RING_SETTINGS_NUM, rng),
            rotor_positions: gen_triple(1, MAX_ROTOR_POSITIONS_NUM, rng),
        }
    }
}

fn gen_triple_unique<R: Rng>(from: u8, to: u8, rng: &mut R) -> (u8, u8, u8) {
    let r = (from..=to).choose_multiple(rng, 3);
    (r[0], r[1], r[2])
}

fn gen_triple<R: Rng>(from: u8, to: u8, rng: &mut R) -> (u8, u8, u8) {
    let r = std::iter::repeat_with(|| rng.gen_range(from..=to))
        .take(3)
        .collect::<Vec<u8>>();

    (r[0], r[1], r[2])
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
        debug_assert_eq!(parents.len(), 2, "crossover should use 2 parents");

        vec![cross_settings(&parents[0], &parents[1], rng)]
    }
}

fn cross_settings<R: Rng>(sett1: &Settings, sett2: &Settings, rng: &mut R) -> Settings {
    let bernoulli = distributions::Bernoulli::new(0.5).unwrap();

    Settings {
        rotors: cross_rotors(sett1.rotors, sett2.rotors, bernoulli, rng),
        ring_settings: cross_positionally(sett1.ring_settings, sett2.ring_settings, bernoulli, rng),
        rotor_positions: cross_positionally(
            sett1.rotor_positions,
            sett2.rotor_positions,
            bernoulli,
            rng,
        ),
    }
}

fn cross_rotors<R: Rng>(
    rotors1: (u8, u8, u8),
    rotors2: (u8, u8, u8),
    bernoulli: distributions::Bernoulli,
    rng: &mut R,
) -> (u8, u8, u8) {
    loop {
        let r = cross_positionally(rotors1, rotors2, bernoulli, rng);

        if is_triple_unique(r) {
            return r;
        }
    }
}

fn is_triple_unique(t: (u8, u8, u8)) -> bool {
    t.0 != t.1 && t.1 != t.2 && t.2 != t.0
}

fn cross_positionally<R: Rng>(
    x: (u8, u8, u8),
    y: (u8, u8, u8),
    bernoulli: distributions::Bernoulli,
    rng: &mut R,
) -> (u8, u8, u8) {
    (
        if bernoulli.sample(rng) { x.0 } else { y.0 },
        if bernoulli.sample(rng) { x.1 } else { y.1 },
        if bernoulli.sample(rng) { x.2 } else { y.2 },
    )
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
    fn mutate<R>(&self, sett: Settings, rng: &mut R) -> Settings
    where
        R: Rng + Sized,
    {
        let num_mutations = ((9_f64 * self.mutation_rate) + rng.gen::<f64>()).floor() as usize;

        if num_mutations == 0 {
            return sett;
        }

        let mut mutated = sett.clone();

        for _ in 0..num_mutations {
            match rng.gen_range(0..=2) {
                0 => mutated.rotors = mutate_triple_unique(sett.rotors, 1, MAX_ROTOR_NUM, rng),
                1 => {
                    mutated.ring_settings =
                        mutate_triple(sett.ring_settings, 1, MAX_RING_SETTINGS_NUM, rng)
                }
                2 => {
                    mutated.rotor_positions =
                        mutate_triple(sett.rotor_positions, 1, MAX_ROTOR_POSITIONS_NUM, rng)
                }
                _ => panic!("out of settings range"),
            }
        }

        mutated
    }
}

fn mutate_triple_unique<R: Rng>(t: (u8, u8, u8), from: u8, to: u8, rng: &mut R) -> (u8, u8, u8) {
    let pos = rng.gen_range(0..3);

    loop {
        let next = change_triple(t, pos, from, to, rng);
        if is_triple_unique(next) {
            return next;
        }
    }
}

fn mutate_triple<R: Rng>(t: (u8, u8, u8), from: u8, to: u8, rng: &mut R) -> (u8, u8, u8) {
    let pos = rng.gen_range(0..3);

    change_triple(t, pos, from, to, rng)
}

fn change_triple<R: Rng>(t: (u8, u8, u8), pos: u8, from: u8, to: u8, rng: &mut R) -> (u8, u8, u8) {
    let v = rng.gen_range(from..=to);
    match pos {
        0 => (v, t.1, t.2),
        1 => (t.0, v, t.2),
        2 => (t.0, t.1, v),
        _ => panic!("out of triple range"),
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::enigma;

    use super::*;

    const LONG_TEXT: &str = "TO BE OR NOT TO BE THAT IS THE QUESTION WHETHER TIS NOBLER IN THE MIND TO SUFFER THE SLINGS AND ARROWS OF OUTRAGEOUS FORTUNE OR TO TAKE ARMS AGAINST A SEA OF TROUBLES AND BY OPPOSING END THEMTO DIETO SLEEP NO MORE AND BY A SLEEP TO SAY WE END THE HEARTACHE AND THE THOUSAND NATURAL SHOCKS THAT FLESH IS HEIR TOTIS A CONSUMMATION DEVOUTLY TO BE WISHD TO DIETO SLEEP TO SLEEP PERCHANCE TO DREAMAY THERES THE RUB FOR IN THAT SLEEP OF DEATH WHAT DREAMS MAY COME WHEN WE HAVE SHUFFLED OFF THIS MORTAL COIL MUST GIVE US PAUSE THERES THE RESPECT THAT MAKES CALAMITY OF SO LONG LIFE";

    #[test]
    fn test_ioc() {
        assert_relative_eq!(index_of_coincidence(""), 0.0);
        assert_relative_eq!(index_of_coincidence("A"), 0.0);
        assert_relative_eq!(index_of_coincidence("AB"), 0.0);
        assert_relative_eq!(index_of_coincidence("ABAA"), 0.5);
        assert_relative_eq!(index_of_coincidence(LONG_TEXT), 0.0455454816328268);
    }

    #[test]
    fn test_fitness() {
        let settings = enigma::Settings {
            rotors: (2, 5, 3),
            ring_settings: (8, 5, 20),
            rotor_positions: (13, 3, 21),
        };

        let machine = Machine::new(&settings).unwrap();
        let ciphertext = machine.encrypt(LONG_TEXT);

        let calc = FitnessCalc {
            ciphertext: Arc::new(ciphertext),
            max_value: 1000000,
        };

        let mut closer_settings = settings.clone();
        closer_settings.ring_settings = (8, 5, 1);

        let wrong_settings = enigma::Settings {
            rotors: (1, 2, 3),
            ring_settings: (1, 1, 1),
            rotor_positions: (1, 1, 1),
        };

        assert_eq!(calc.fitness_of(&settings), 45545);
        assert_eq!(calc.fitness_of(&closer_settings), 25278);
        assert_eq!(calc.fitness_of(&wrong_settings), 24561);
    }

    #[test]
    fn test_settings_builder() {
        let mut rng = rand::thread_rng();
        let b = SettingsBuilder {};

        for _ in 0..10000 {
            let sett = b.build_genome(0, &mut rng);
            assert!(is_settings_valid(&sett))
        }
    }

    #[test]
    fn test_settings_crossover() {
        let mut rng = rand::thread_rng();
        let b = SettingsBuilder {};
        let c = SettingsCrossover {};

        for _ in 0..10000 {
            let sett1 = b.build_genome(0, &mut rng);
            let sett2 = b.build_genome(0, &mut rng);

            let offsprings = c.crossover(vec![sett1.clone(), sett2.clone()], &mut rng);

            assert!(offsprings.iter().all(is_settings_valid))
        }
    }

    #[test]
    fn test_settings_mutator() {
        let mut rng = rand::thread_rng();
        let b = SettingsBuilder {};
        let m = SettingsMutator { mutation_rate: 0.1 };

        for _ in 0..10000 {
            let sett = b.build_genome(0, &mut rng);
            let mutated_sett = m.mutate(sett, &mut rng);

            assert!(is_settings_valid(&mutated_sett));
        }
    }

    fn is_settings_valid(sett: &Settings) -> bool {
        is_triple_unique(sett.rotors)
            && is_triple_in_range(sett.rotors, 1, MAX_ROTOR_NUM)
            && is_triple_in_range(sett.ring_settings, 1, MAX_RING_SETTINGS_NUM)
            && is_triple_in_range(sett.rotor_positions, 1, MAX_ROTOR_POSITIONS_NUM)
    }

    fn is_triple_in_range(t: (u8, u8, u8), from: u8, to: u8) -> bool {
        t.0 >= from && t.0 <= to && t.1 >= from && t.1 <= to && t.2 >= from && t.2 <= to
    }
}
