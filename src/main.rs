pub mod enigma;
pub mod gen;

fn main() {
    let settings = enigma::Settings {
        rotors: (2, 5, 3),
        ring_settings: (8, 5, 20),
        rotor_positions: (13, 3, 21),
        plugboard: vec![('A', 'B'), ('C', 'D')],
    };

    let machine = enigma::Machine::new(&settings).unwrap();

    let cipher = machine.encrypt("test test test");
    let plain = machine.decrypt(&cipher);

    println!("{}", cipher);
    println!("{}", plain);
}
