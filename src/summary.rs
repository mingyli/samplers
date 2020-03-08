use crate::SamplersError;

pub trait Measure<'a, T: 'a> {
    fn observe(&mut self, value: &T) -> Result<(), failure::Error>;
    fn observe_many(
        &mut self,
        mut values: impl Iterator<Item = &'a T>,
    ) -> Result<(), failure::Error> {
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

impl Measure<'_, f64> for DistributionSummary {
    fn observe(&mut self, &value: &f64) -> Result<(), failure::Error> {
        self.min = Some(self.min.map_or(value, |min| min.min(value)));
        self.max = Some(self.max.map_or(value, |max| max.max(value)));
        self.count += 1;
        let delta1 = value - self.mean.unwrap_or_default();
        self.mean = Some(self.mean.unwrap_or_default() + delta1 / (self.count as f64));
        let delta2 = value - self.mean.unwrap_or_default();
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
    summary.observe(&8.25)?;
    assert_eq!(summary.min(), Some(8.25));
    assert_eq!(summary.max(), Some(8.25));
    assert_eq!(summary.mean(), Some(8.25));
    assert_eq!(summary.variance(), Some(0.0));
    summary.observe(&-1.5)?;
    assert_eq!(summary.min(), Some(-1.5));
    assert_eq!(summary.max(), Some(8.25));
    assert_eq!(summary.mean(), Some(3.375));
    assert_eq!(summary.variance(), Some(23.765625));
    assert_eq!(summary.sample_variance(), Some(47.53125));

    let mut summary = DistributionSummary::default();
    summary.observe_many([-1.25, 6.25, 16.0].iter())?;
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
            "Count: {}\nMinimum: {}\nMaximum: {}\nMean: {}\nVariance: {}\nSample variance: {}",
            self.count(),
            self.min().unwrap_or(std::f64::NAN),
            self.max().unwrap_or(std::f64::NAN),
            self.mean().unwrap_or(std::f64::NAN),
            self.variance().unwrap_or(std::f64::NAN),
            self.sample_variance().unwrap_or(std::f64::NAN),
        )
    }
}

// A histogram with boundaries [-5.0, 0.0, 5.0] means its
// buckets are (-inf, -5.0), [-5.0, 0.0), [0.0, 5.0), [5.0, inf).
#[derive(Debug)]
pub struct Histogram {
    boundaries: Vec<f64>,
    counts: Vec<u64>,
}

impl Histogram {
    pub fn new(boundaries: Vec<f64>) -> Histogram {
        // TODO: validate boundaries
        Histogram {
            counts: vec![0; boundaries.len() + 1],
            boundaries,
        }
    }

    pub fn collect(&self) -> Vec<Bucket> {
        use itertools::Itertools;

        std::iter::once(std::f64::NEG_INFINITY)
            .chain(self.boundaries.iter().cloned())
            .chain(std::iter::once(std::f64::INFINITY))
            .tuple_windows::<(f64, f64)>()
            .zip(self.counts.iter())
            .map(|((lower, upper), &count)| Bucket {
                lower,
                upper,
                count,
            })
            .collect()
    }
}

impl Measure<'_, f64> for Histogram {
    fn observe(&mut self, &value: &f64) -> Result<(), failure::Error> {
        let mut it = self
            .boundaries
            .iter()
            .enumerate()
            .filter(|(_index, &boundary)| value < boundary);
        if let Some((index, _boundary)) = it.next() {
            if let Some(count) = self.counts.get_mut(index) {
                *count += 1;
            } else {
                return Err(SamplersError::CouldNotObserveValue { value }.into());
            }
        } else if let Some(last) = self.counts.last_mut() {
            *last += 1;
        } else {
            return Err(SamplersError::CouldNotObserveValue { value }.into());
        }
        Ok(())
    }
}

#[test]
fn test_histogram() -> Result<(), failure::Error> {
    let mut histogram = Histogram::new(vec![-5.0, 0.0, 5.0]);
    assert_eq!(histogram.counts, vec![0, 0, 0, 0]);
    histogram.observe(&1.0)?;
    assert_eq!(histogram.counts, vec![0, 0, 1, 0]);
    histogram.observe(&1.0)?;
    assert_eq!(histogram.counts, vec![0, 0, 2, 0]);
    histogram.observe(&-1.0)?;
    assert_eq!(histogram.counts, vec![0, 1, 2, 0]);
    histogram.observe(&-6.0)?;
    assert_eq!(histogram.counts, vec![1, 1, 2, 0]);
    histogram.observe_many([-20.0, 120.0, 2.0].iter())?;
    assert_eq!(histogram.counts, vec![2, 1, 3, 1]);

    let mut histogram = Histogram::new(vec![0.0]);
    assert_eq!(histogram.counts, vec![0, 0]);
    histogram.observe_many([-20.0, 120.0, 2.0].iter())?;
    assert_eq!(histogram.counts, vec![1, 2]);

    Ok(())
}

#[derive(Debug, Default)]
pub struct Bucket {
    lower: f64,
    upper: f64,
    count: u64,
}

impl Bucket {
    pub fn lower(&self) -> f64 {
        self.lower
    }

    pub fn upper(&self) -> f64 {
        self.upper
    }

    pub fn count(&self) -> u64 {
        self.count
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
