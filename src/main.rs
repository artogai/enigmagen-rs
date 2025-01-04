use genevo::{
    operator::prelude::{ElitistReinserter, MaximizeSelector},
    prelude::*,
};

pub mod enigma;
pub mod gen;

fn main() {
    let machine_settings = enigma::Settings {
        rotors: (2, 5, 3),
        ring_settings: (8, 5, 20),
        rotor_positions: (13, 3, 21),
        plugboard: vec![('A', 'B'), ('C', 'D')],
    };

    let sim_opts = gen::Options::default();

    let fitness_calc = gen::FitnessCalc {
        ciphertext: todo!(),
        max_value: sim_opts.fitness_scale,
    };

    let selector = MaximizeSelector::new(
        sim_opts.selection_ratio,
        sim_opts.num_individuals_per_parents,
    );

    let mutator = gen::SettingsMutator {
        mutation_rate: sim_opts.mutation_rate,
    };

    let reinserter = ElitistReinserter::new(fitness_calc, true, sim_opts.reinsertion_ratio);

    let generation_limit = GenerationLimit::new(sim_opts.generation_limit);

    let initial_population: Population<enigma::Settings> = build_population()
        .with_genome_builder(gen::SettingsBuilder)
        .of_size(sim_opts.population_size)
        .uniform_at_random();

    let mut sim = simulate(
        genetic_algorithm()
            .with_evaluation(fitness_calc)
            .with_selection(selector)
            .with_crossover(gen::SettingsCrossover)
            .with_mutation(mutator)
            .with_reinsertion(reinserter)
            .with_initial_population(initial_population)
            .build(),
    )
    .until(generation_limit)
    .build();
}
