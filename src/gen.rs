use std::{cmp, collections::HashMap, sync::LazyLock};

use genevo::{
    genetic::{Children, Parents},
    operator::{CrossoverOp, GeneticOperator, MutationOp},
    prelude::{FitnessFunction, GenomeBuilder, Genotype},
    random::Rng,
};
use rand::{
    distributions,
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
};

use crate::enigma::{
    Machine, Settings, MAX_PLUGS, MAX_RING_SETTINGS_NUM, MAX_ROTOR_NUM, MAX_ROTOR_POSITIONS_NUM,
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
        Settings {
            rotors: gen_rotors(rng),
            ring_settings: gen_ring_settings(rng),
            rotor_positions: gen_rotor_positions(rng),
            plugboard: gen_plugboard(rng),
        }
    }
}

fn gen_rotors<R: Rng>(rng: &mut R) -> (u8, u8, u8) {
    gen_triple_unique(1, MAX_ROTOR_NUM, rng)
}

fn gen_ring_settings<R: Rng>(rng: &mut R) -> (u8, u8, u8) {
    gen_triple(1, MAX_RING_SETTINGS_NUM, rng)
}

fn gen_rotor_positions<R: Rng>(rng: &mut R) -> (u8, u8, u8) {
    gen_triple(1, MAX_ROTOR_POSITIONS_NUM, rng)
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

fn gen_plugboard<R: Rng>(rng: &mut R) -> Vec<(char, char)> {
    let plugs_cnt = rng.gen_range(0..=MAX_PLUGS);
    let mut plugs = Vec::new();

    for _ in 0..plugs_cnt {
        let mut next: (char, char);

        loop {
            next = PLUGS.choose(rng).unwrap().clone();

            if can_add_plug(&plugs, next) {
                break;
            }
        }

        plugs.push(next.clone());
    }

    plugs
}

fn can_add_plug(plugs: &[(char, char)], next: (char, char)) -> bool {
    // on small numbers vec should be faster that building a set
    plugs
        .iter()
        .all(|(p0, p1)| p0 != &next.0 && p1 != &next.0 && p0 != &next.1 && p1 != &next.1)
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
        let bernoulli = distributions::Bernoulli::new(0.5).unwrap();

        for _ in 0..num_parents {
            let i = rng.gen_range(0..num_parents);
            let mut j = rng.gen_range(0..num_parents);

            while i == j {
                j = rng.gen_range(0..num_parents);
            }

            offsprings.push(cross_settings(&parents[i], &parents[j], bernoulli, rng));
        }

        offsprings
    }
}

fn cross_settings<R: Rng>(
    sett1: &Settings,
    sett2: &Settings,
    bernoulli: distributions::Bernoulli,
    rng: &mut R,
) -> Settings {
    Settings {
        rotors: cross_rotors(sett1.rotors, sett2.rotors, bernoulli, rng),
        ring_settings: cross_positionally(sett1.ring_settings, sett2.ring_settings, bernoulli, rng),
        rotor_positions: cross_positionally(
            sett1.rotor_positions,
            sett2.rotor_positions,
            bernoulli,
            rng,
        ),
        plugboard: cross_plugboards(&sett1.plugboard, &sett2.plugboard, bernoulli, rng),
    }
}

fn cross_plugboards<R: Rng>(
    plugs1: &[(char, char)],
    plugs2: &[(char, char)],
    bernoulli: distributions::Bernoulli,
    rng: &mut R,
) -> Vec<(char, char)> {
    let mut plugs = Vec::new();

    for i in 0..cmp::max(plugs1.len(), plugs2.len()) {
        let c = bernoulli.sample(rng);

        if c && i < plugs1.len() && can_add_plug(&plugs, plugs1[i]) {
            plugs.push(plugs1[i]);
        }

        if !c && i < plugs2.len() && can_add_plug(&plugs, plugs2[i]) {
            plugs.push(plugs2[i]);
        }
    }

    plugs
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
    t.0 != t.1 && t.1 != t.2
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
        let mut mutated = sett.clone();

        match rng.gen_range(0..=3) {
            0 => mutated.rotors = mutate_rotors(sett.rotors, rng),
            1 => mutated.ring_settings = mutate_ring_settings(sett.ring_settings, rng),
            2 => mutated.rotor_positions = mutate_rotor_positions(sett.rotor_positions, rng),
            3 => mutated.plugboard = mutate_plugboard(&sett.plugboard, rng),
            _ => panic!("out of settings range"),
        }

        mutated
    }
}

fn mutate_rotors<R: Rng>(rotors: (u8, u8, u8), rng: &mut R) -> (u8, u8, u8) {
    mutate_triple_unique(rotors, 1, MAX_ROTOR_NUM, rng)
}

fn mutate_ring_settings<R: Rng>(sett: (u8, u8, u8), rng: &mut R) -> (u8, u8, u8) {
    mutate_triple(sett, 1, MAX_RING_SETTINGS_NUM, rng)
}

fn mutate_rotor_positions<R: Rng>(pos: (u8, u8, u8), rng: &mut R) -> (u8, u8, u8) {
    mutate_triple(pos, 1, MAX_ROTOR_POSITIONS_NUM, rng)
}

fn mutate_plugboard<R: Rng>(plugs: &[(char, char)], rng: &mut R) -> Vec<(char, char)> {
    let mut mutated = plugs.to_vec();

    if !plugs.is_empty() {
        let idx = rng.gen_range(0..plugs.len());
        let mut next: (char, char);

        loop {
            next = PLUGS.choose(rng).unwrap().clone();
            let (left, right) = mutated.split_at(idx);

            if can_add_plug(left, next) && can_add_plug(&right[1..], next) {
                break;
            }
        }

        mutated[idx] = next;
    }

    mutated
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

    fn is_settings_valid(sett: &Settings) -> bool {
        is_triple_unique(sett.rotors)
            && is_triple_in_range(sett.rotors, 1, MAX_ROTOR_NUM)
            && is_triple_in_range(sett.ring_settings, 1, MAX_RING_SETTINGS_NUM)
            && is_triple_in_range(sett.rotor_positions, 1, MAX_ROTOR_POSITIONS_NUM)
            && is_plugboard_valid(&sett.plugboard)
    }

    fn is_plugboard_valid(p: &[(char, char)]) -> bool {
        let plugs = p
            .iter()
            .flat_map(|(p1, p2)| vec![p1, p2])
            .collect::<Vec<_>>();

        let mut letters_uniq = plugs.clone();
        letters_uniq.dedup();

        p.len() <= MAX_PLUGS && plugs.len() == letters_uniq.len()
    }

    fn is_triple_in_range(t: (u8, u8, u8), from: u8, to: u8) -> bool {
        t.0 >= from && t.0 <= to && t.1 >= from && t.1 <= to && t.2 >= from && t.2 <= to
    }
}
