use rand_distr::{Distribution, Uniform};
use statrs::distribution::{Binomial, Exponential, Normal, Poisson};

pub fn gaussian(mean: f64, variance: f64) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let normal = Normal::new(mean, variance.sqrt())?;
    Ok(normal.sample_iter(rand::thread_rng()))
}

pub fn binomial(n: u64, p: f64) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let binomial = Binomial::new(p, n)?;
    Ok(binomial.sample_iter(rand::thread_rng()))
}

pub fn poisson(lambda: f64) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let poisson = Poisson::new(lambda)?;
    Ok(poisson.sample_iter(rand::thread_rng()))
}

pub fn exponential(lambda: f64) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let exponential = Exponential::new(lambda)?;
    Ok(exponential.sample_iter(rand::thread_rng()))
}

pub fn continuous_uniform(lower: f64, upper: f64) -> impl Iterator<Item = f64> {
    let uniform = Uniform::new(lower, upper);
    uniform.sample_iter(rand::thread_rng())
}

pub fn discrete_uniform(lower: i64, upper: i64) -> impl Iterator<Item = i64> {
    let uniform = Uniform::new_inclusive(lower, upper);
    uniform.sample_iter(rand::thread_rng())
}
