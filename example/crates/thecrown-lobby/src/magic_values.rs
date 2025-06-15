use rand::Rng;

// Scale values for various achievements and game mechanics
pub const MIN_SCALE: f64 = 0.1;
pub const AVG_SCALE: f64 = 1.0;
pub const MAX_SCALE: f64 = 15.0;
pub const SHRUNK_ACHIEVEMENT_SCALE: f64 = 0.2;
pub const TITAN_ACHIEVEMENT_SCALE: f64 = 13.0; // 12.2 for the same %

pub fn distribution(mu: f64, min: f64, max: f64) -> f64 {
    // compute e^{-10 * rand}
    let mut rng = rand::rng();
    let random_number: f64 = rng.random_range(0.0..1.0);
    let result = (-10.0 * random_number).exp();

    // 50% to be on the left, 50% to be on the right
    if rng.random_bool(0.5) {
        mu + result * (max - mu)
    } else {
        mu - result * (mu - min)
    }
}

pub fn step_height_scale(scale: f64) -> f64 {
    0.6 * scale
}

pub fn jump_strength_scale(scale: f64) -> f64 {
    // linear interop
    // 1->0.42
    // 15->1
    0.04143 * scale + 0.37857
}
