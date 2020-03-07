use rand::Rng;
use rand_distr::{Distribution, Normal, StandardNormal};

pub fn gaussian(mean: f64, variance: f64) -> Result<impl Iterator<Item = f64>, failure::Error> {
    let normal = Normal::new(mean, variance.sqrt())?;
    Ok(normal.sample_iter(rand::thread_rng()))
}

pub fn standard_gaussian() -> impl Iterator<Item = f64> {
    rand::thread_rng().sample_iter(StandardNormal)
}
