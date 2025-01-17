use std::sync::Arc;

use anyhow::anyhow;
use chrono::Duration;
use gen::index_of_coincidence_norm;
use genevo::operator::prelude::{ElitistReinserter, MaximizeSelector};
use genevo::prelude::*;
use genevo::types::fmt::Display;
use moka::sync::Cache;

pub mod enigma;
pub mod gen;

fn main() {
    let plaintext = "TO BE OR NOT TO BE THAT IS THE QUESTION WHETHER TIS NOBLER IN THE MIND TO SUFFER THE SLINGS AND ARROWS OF OUTRAGEOUS FORTUNE OR TO TAKE ARMS AGAINST A SEA OF TROUBLES AND BY OPPOSING END THEM TO DIE TO SLEEP NO MORE AND BY A SLEEP TO SAY WE END THE HEARTACHE AND THE THOUSAND NATURAL SHOCKS THAT FLESH IS HEIR TO TIS A CONSUMMATION DEVOUTLY TO BE WISHD TO DIE TO SLEEP TO SLEEP PERCHANCE TO DREAM AY THERES THE RUB FOR IN THAT SLEEP OF DEATH WHAT DREAMS MAY COME WHEN WE HAVE SHUFFLED OFF THIS MORTAL COIL MUST GIVE US PAUSE THERES THE RESPECT THAT MAKES CALAMITY OF SO LONG LIFE";

    let settings = enigma::Settings {
        rotors: (2, 5, 3),
        ring_settings: (8, 5, 20),
        rotor_positions: (13, 3, 21),
    };

    let sim_opts = gen::Options {
        fitness_scale: 1_000_000,
        population_size: 1_500_000,
        generation_limit: 300,
        time_limit: Duration::minutes(15),
        selection_ratio: 0.5,
        mutation_rate: 0.05,
        reinsertion_ratio: 0.7,
        cache_size: 3_000_000,
    };

    let target_fitness = Some(index_of_coincidence_norm(
        &plaintext,
        sim_opts.fitness_scale,
    ));

    let machine = enigma::Machine::new(&settings).unwrap();
    let ciphertext = machine.encrypt(plaintext);

    println!("Plaintext: {}", plaintext);
    println!("Ciphertext: {}", ciphertext);

    let found_settings = run_simulation(&ciphertext, sim_opts, target_fitness).unwrap();
    let found_machine = enigma::Machine::new(&found_settings).unwrap();
    let found_plaintext = found_machine.decrypt(&ciphertext);

    println!("Decrypted plaintext: {}", found_plaintext);
}

fn run_simulation(
    ciphertext: &str,
    opts: gen::Options,
    target_fitness: Option<usize>,
) -> anyhow::Result<enigma::Settings> {
    let fitness_calc = gen::FitnessCalc {
        ciphertext: Arc::new(ciphertext.to_string()),
        max_value: opts.fitness_scale,
        cache: Cache::new(opts.cache_size as u64),
    };

    let selector = MaximizeSelector::new(opts.selection_ratio, 2);

    let mutator = gen::SettingsMutator {
        mutation_rate: opts.mutation_rate,
    };

    let reinserter = ElitistReinserter::new(fitness_calc.clone(), true, opts.reinsertion_ratio);

    let initial_population = build_population()
        .with_genome_builder(gen::SettingsBuilder)
        .of_size(opts.population_size)
        .uniform_at_random();

    let termination = or(
        or(
            GenerationLimit::new(opts.generation_limit),
            TimeLimit::new(opts.time_limit),
        ),
        FitnessLimit::new(target_fitness.unwrap_or(opts.fitness_scale)),
    );

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
    .until(termination)
    .build();

    loop {
        match sim.step() {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                println!(
                    "step: generation: {}, average_fitness: {}, \
                     best fitness: {}, duration: {}, processing_time: {}",
                    step.iteration,
                    evaluated_population.average_fitness(),
                    best_solution.solution.fitness,
                    step.duration.fmt(),
                    step.processing_time.fmt(),
                );
                let settings = best_solution.solution.genome;
                println!("settings: {:?}", settings);
            }
            Ok(SimResult::Final(step, processing_time, duration, reason)) => {
                let best_solution = step.result.best_solution;
                println!(
                    "Final result after {}: generation: {}, \
                     best solution with fitness {} found in generation {}, processing_time: {}, reason: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt(),
                    reason,
                );
                let settings = best_solution.solution.genome;
                println!("settings: {:?}", settings);
                return Ok(settings);
            }
            Err(err) => {
                return Err(anyhow!(err));
            }
        }
    }
}
