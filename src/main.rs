use std::sync::Arc;

use genevo::operator::prelude::{ElitistReinserter, MaximizeSelector};
use genevo::{population::*, prelude::*, types::fmt::Display};

pub mod enigma;
pub mod gen;

const PLAINTEXT: &str = "TO BE OR NOT TO BE THAT IS THE QUESTION WHETHER TIS NOBLER IN THE MIND TO SUFFER THE SLINGS AND ARROWS OF OUTRAGEOUS FORTUNE OR TO TAKE ARMS AGAINST A SEA OF TROUBLES AND BY OPPOSING END THEMTO DIETO SLEEP NO MORE AND BY A SLEEP TO SAY WE END THE HEARTACHE AND THE THOUSAND NATURAL SHOCKS THAT FLESH IS HEIR TOTIS A CONSUMMATION DEVOUTLY TO BE WISHD TO DIETO SLEEP TO SLEEP PERCHANCE TO DREAMAY THERES THE RUB FOR IN THAT SLEEP OF DEATH WHAT DREAMS MAY COME WHEN WE HAVE SHUFFLED OFF THIS MORTAL COIL MUST GIVE US PAUSE THERES THE RESPECT THAT MAKES CALAMITY OF SO LONG LIFE";

fn main() {
    let machine_settings = enigma::Settings {
        rotors: (2, 5, 3),
        ring_settings: (8, 5, 20),
        rotor_positions: (13, 3, 21),
    };

    let machine = enigma::Machine::new(&machine_settings).unwrap();

    let ciphertext = machine.encrypt(PLAINTEXT);

    let sim_opts = gen::Options::default();

    let fitness_calc = gen::FitnessCalc {
        ciphertext: Arc::new(ciphertext.clone()),
        max_value: sim_opts.fitness_scale,
    };

    let selector = MaximizeSelector::new(
        sim_opts.selection_ratio,
        sim_opts.num_individuals_per_parents,
    );

    let mutator = gen::SettingsMutator {
        mutation_rate: sim_opts.mutation_rate,
    };

    let reinserter = ElitistReinserter::new(fitness_calc.clone(), true, sim_opts.reinsertion_ratio);

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

    loop {
        let result = sim.step();

        match result {
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
            Ok(SimResult::Final(step, processing_time, duration, _)) => {
                let best_solution = step.result.best_solution;
                println!(
                    "Final result after {}: generation: {}, \
                     best solution with fitness {} found in generation {}, processing_time: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt(),
                );
                let settings = best_solution.solution.genome;
                println!("settings: {:?}", settings);

                let machine = enigma::Machine::new(&settings).unwrap();
                println!("plaintext: {:?}", machine.decrypt(&ciphertext));
                break;
            }
            Err(error) => {
                println!("{}", error);
                break;
            }
        }
    }
}
