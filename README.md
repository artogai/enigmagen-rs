# Enigmagen-rs

Cracking the [Enigma machine](https://en.wikipedia.org/wiki/Enigma_machine) using a [genetic algorithm (GA)](https://en.wikipedia.org/wiki/Genetic_algorithm).

# Description

Implementation of GA-based search for Enigma machine settings: rotors, rotor settings, ring settings, and no plugboard. The solution [usually](#note-1) converges in under 10 minutes on my laptop (13th Gen Intel(R) Core(TM) i7-1355U). 

## Intro

I was originally inspired by watching the video ["Cracking Enigma in 2021 - Computerphile"](https://www.youtube.com/watch?v=RzWB5jL5RX0) which explores the question, "Can we easily break Enigma machine using modern computational power?". 

The Enigma machine is defined by its settings, and in the video, they describe a solution that leverages its known weakness: the more settings you guess, the more original letters you recover. This differs from modern encryption algorithms, where output changes drastically with small input [changes](https://en.wikipedia.org/wiki/Avalanche_effect).

The solution from the video tries to find best settings iteratively with respect to a [fitness function](https://en.wikipedia.org/wiki/Fitness_function).

1. find best rotors and rotor settings (other settings are zero)
2. find best ring settings
3. find best plugboard configuration

This seemed like an optimization problem, so I wanted to try a different approach - a genetic algorithm. 

## Solution

Enigma machine settings

- 3 rotors I - VI, chosen uniquely 
- 3 rotor settings A-Z, non-unique
- 3 ring settings A-Z, non-unique 
- plugboard, 0-10 pairs of letters without repetition ([not used here](#note-2))

So search space is: (6 * 5 * 4) * 26^3 * 26^3 = 37,069,893,120.

**Fitness function**

- build the machine from settings
- decode ciphertext
- calculate [index of coincidence](https://en.wikipedia.org/wiki/Index_of_coincidence) of the resulting text

I have found that it benefits greatly from caching since a lot of settings are carried over different generations, especially if algorithm gets stuck and can't improve solution for some time.

**Crossover operation**

Iterate over settings and randomly take each parameter either from parent 1 or parent 2, ensuring that rotors remain unique. The position of settings doesn't change, so the second ring setting always comes from the second ring settings of one of the parents.

**Mutation operation**

With ```mutation_probability```, flip none or some of the settings.

**Project structure**

```enigma.rs``` - wrapper around concrete Enigma implementation

```gen.rs``` - GA operations (generation, fitness, etc.) 

```main.rs``` - building and running simulation

I have used following packages
- Enigma machine: [enigma-simulator](https://docs.rs/enigma-simulator/latest/enigma_simulator/)
- GA library: [genevo](https://docs.rs/genevo/latest/genevo/)

---
#### Note 1

Due to the random nature of GA, it either finds the exact solution, a close solution where text is somewhat readable, or no solution. Restarting it in the last case usually fixes the problem.

When it finds the exact solution, the settings can be different from the initial ones (except for the rotors). As far as I understand, this happens because some rotor and ring settings combinations produce the same result.

If you want to try it yourself with your text, note that same limitations apply as in the original video - the text needs to be long enough for the fitness function to work properly.

---

---
#### Note 2

I have ignored plugboard settings due to two reasons: 

1. it blows up search space drastically 
2. finding right GA operations for them is harder

In the original video, it is shown that it is possible to find plugboard settings after the initial settings are found using [hill climbing](https://en.wikipedia.org/wiki/Hill_climbing). 

They also note that since rotors and rotor settings are searched first with ring settings set to zero, the algorithm doesn't find the exact solution. It will be interesting to see if finding all settings in one go, as done here, can improve results, but this is beyond the scope of this project.

---

# Run 
```
cargo run --release
```

```
cargo test
```