use rand::seq::IndexedRandom;

pub fn random_token() -> String {
    let length = 64;
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    let mut rng = rand::rng();
    let token: String = (0..length)
        .map(|_| *alphabet.choose(&mut rng).unwrap())
        .collect();
    token
}
