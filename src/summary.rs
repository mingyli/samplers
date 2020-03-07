pub trait Summary<T> {
    fn observe(&mut self, value: T) -> Result<(), failure::Error>;
    fn observe_many(&mut self, mut values: impl Iterator<Item = T>) -> Result<(), failure::Error> {
        values.try_for_each(|value| self.observe(value))
    }
}

#[derive(Debug, Default)]
pub struct DistributionSummary {
    min: Option<f64>,
    max: Option<f64>,
    mean: Option<f64>,
    count: u64,
    sum_of_square_errors: Option<f64>,
}

impl Summary<f64> for DistributionSummary {
    fn observe(&mut self, value: f64) -> Result<(), failure::Error> {
        self.min = Some(self.min.map_or(value, |min| min.min(value)));
        self.max = Some(self.max.map_or(value, |max| max.max(value)));
        self.count += 1;
        let delta1 = value - self.mean.unwrap_or_default();
        self.mean = Some(self.mean.unwrap_or_default() + delta1 / (self.count as f64));
        let delta2 = value - self.mean.expect("Mean is some value.");
        self.sum_of_square_errors =
            Some(self.sum_of_square_errors.unwrap_or_default() + delta1 * delta2);
        Ok(())
    }
}

#[test]
fn test_distribution_summary() -> Result<(), failure::Error> {
    let mut summary = DistributionSummary::default();
    assert_eq!(summary.mean(), None);
    assert_eq!(summary.max(), None);
    assert_eq!(summary.variance(), None);
    assert_eq!(summary.sample_variance(), None);
    assert_eq!(summary.count(), 0);
    summary.observe(8.25)?;
    assert_eq!(summary.min(), Some(8.25));
    assert_eq!(summary.max(), Some(8.25));
    assert_eq!(summary.mean(), Some(8.25));
    assert_eq!(summary.variance(), Some(0.0));
    summary.observe(-1.5)?;
    assert_eq!(summary.min(), Some(-1.5));
    assert_eq!(summary.max(), Some(8.25));
    assert_eq!(summary.mean(), Some(3.375));
    assert_eq!(summary.variance(), Some(23.765625));
    assert_eq!(summary.sample_variance(), Some(47.53125));

    let mut summary = DistributionSummary::default();
    summary.observe_many([-1.25, 6.25, 16.0].iter().cloned())?;
    assert_eq!(summary.mean(), Some(7.0));
    assert_eq!(summary.variance(), Some(49.875));
    assert_eq!(summary.sample_variance(), Some(74.8125));

    Ok(())
}

impl DistributionSummary {
    pub fn mean(&self) -> Option<f64> {
        self.mean
    }

    pub fn variance(&self) -> Option<f64> {
        Some(self.sum_of_square_errors? / self.count as f64)
    }

    pub fn sample_variance(&self) -> Option<f64> {
        Some(self.sum_of_square_errors? / (self.count as f64 - 1.0))
    }

    pub fn min(&self) -> Option<f64> {
        self.min
    }

    pub fn max(&self) -> Option<f64> {
        self.max
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

use std::fmt;
impl fmt::Display for DistributionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Count: {}\nMinimum: {}\nMaximum: {}\nMean: {}\nVariance: {}",
            self.count(),
            self.min().unwrap(),
            self.max().unwrap(),
            self.mean().unwrap(),
            self.variance().unwrap()
        )
    }
}

pub fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (_count, mean) = values.fold((0, 0.0), |(count, mean), v| {
        (count + 1, mean + (v - mean) / (count as f64 + 1.0))
    });
    mean
}

pub fn mean_result(
    mut values: impl Iterator<Item = Result<f64, failure::Error>>,
) -> Result<f64, failure::Error> {
    let (_count, mean) = values
        .try_fold::<_, _, Result<(u64, f64), failure::Error>>((0, 0.0), |(count, mean), r| {
            Ok((count + 1, mean + (r? - mean) / (count as f64 + 1.0)))
        })?;
    Ok(mean)
}

pub fn variance(values: impl Iterator<Item = f64>) -> (f64, f64) {
    // An implementation of Welford's algorithm.
    let (count, _mean, sum_square_difference_from_mean) = values.fold(
        (0, 0.0, 0.0),
        |(mut count, mut mean, mut sum_square_difference_from_mean), v| {
            count += 1;
            let delta1 = v - mean;
            mean += delta1 / count as f64;
            let delta2 = v - mean;
            sum_square_difference_from_mean += delta1 * delta2;
            (count, mean, sum_square_difference_from_mean)
        },
    );
    let population_variance = sum_square_difference_from_mean / count as f64;
    let sample_variance = sum_square_difference_from_mean / (count as f64 - 1.0);
    (population_variance, sample_variance)
}

pub fn variance_result(
    mut values: impl Iterator<Item = Result<f64, failure::Error>>,
) -> Result<(f64, f64), failure::Error> {
    // An implementation of Welford's algorithm.
    let (count, _mean, sum_square_difference_from_mean) =
        values.try_fold::<_, _, Result<(u64, f64, f64), failure::Error>>(
            (0, 0.0, 0.0),
            |(mut count, mut mean, mut sum_square_difference_from_mean), r| {
                let v = r?;
                count += 1;
                let delta1 = v - mean;
                mean += delta1 / count as f64;
                let delta2 = v - mean;
                sum_square_difference_from_mean += delta1 * delta2;
                Ok((count, mean, sum_square_difference_from_mean))
            },
        )?;
    let population_variance = sum_square_difference_from_mean / count as f64;
    let sample_variance = sum_square_difference_from_mean / (count as f64 - 1.0);
    Ok((population_variance, sample_variance))
}

#[test]
fn test_mean() {
    let values = [];
    assert_eq!(mean(values.iter().cloned()), (0.0));
    let values = [(-1.2), (3.8)];
    assert_eq!(mean(values.iter().cloned()), (1.3));
}

#[test]
fn test_variance() {
    let values = [4.2, -0.8];
    assert_eq!(variance(values.iter().cloned()), (6.25, 12.5));
    let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    assert_eq!(variance(values.iter().cloned()), (105.0 / 36.0, 3.5));
}
