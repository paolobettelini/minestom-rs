use rand::{thread_rng, Rng};

pub fn distribution(mu: f64, min: f64, max: f64) -> f64 {
    // compute e^{-10 * rand}
    let mut rng = thread_rng();
    let random_number: f64 = rng.gen_range(0.0..1.0);
    let result = (-10.0 * random_number).exp();

    // 50% to be on the left, 50% to be on the right
    if rng.random() {
        mu + result * (max - mu)
    } else {
        mu - result * (mu - min)
    }
}
