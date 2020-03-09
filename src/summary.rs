use std::fmt;

pub trait Observer<'a, T: 'a> {
    fn observe(&mut self, value: &T) -> Result<(), failure::Error>;
    fn observe_many(
        &mut self,
        mut values: impl Iterator<Item = &'a T>,
    ) -> Result<(), failure::Error> {
        values.try_for_each(|value| self.observe(value))
    }
}

/// An implementation of online updates for central moments.
///
/// Functions prefixed with "population_" have no adjustment terms applied for
/// bias; they are computed as if the observed values are the entire
/// distribution. The other functions have adjustment terms applied for bias
/// where applicable; these functions assume that the observed values are
/// sampled from some distribution.
///
/// `momentp` is the pth order central moment scaled by n. It is the sum of
/// deviations from the mean taken to the pth power:
/// $ \sum_{i=1}^n (x_i - \bar{x})^p $.
#[derive(Debug, Default)]
struct CentralMomentsSummary {
    count: u64,
    mean: Option<f64>,
    moment2: Option<f64>,
    moment3: Option<f64>,
    moment4: Option<f64>,
}

impl CentralMomentsSummary {
    fn n(&self) -> f64 {
        self.count as f64
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn mean(&self) -> Option<f64> {
        self.mean
    }

    pub fn variance(&self) -> Option<f64> {
        Some(self.moment2? / (self.n() - 1.0))
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.variance().map(f64::sqrt)
    }

    pub fn skewness(&self) -> Option<f64> {
        Some(
            self.n() * (self.n() - 1.0).sqrt() * self.moment3?
                / (self.n() - 2.0)
                / self.moment2?.powf(1.5),
        )
    }

    pub fn kurtosis(&self) -> Option<f64> {
        Some(
            (self.n() + 1.0) * self.n() * (self.n() - 1.0) / (self.n() - 2.0) / (self.n() - 3.0)
                * self.moment4?
                / self.moment2?.powi(2)
                - 3.0 * (self.n() - 1.0).powi(2) / (self.n() - 2.0) / (self.n() - 3.0)
                + 3.0,
        )
    }

    pub fn population_variance(&self) -> Option<f64> {
        Some(self.moment2? / self.n())
    }

    pub fn population_standard_deviation(&self) -> Option<f64> {
        self.population_variance().map(f64::sqrt)
    }

    pub fn population_skewness(&self) -> Option<f64> {
        Some((self.n()).sqrt() * self.moment3? / self.moment2?.powf(1.5))
    }

    pub fn population_kurtosis(&self) -> Option<f64> {
        Some((self.n()) * self.moment4? / self.moment2?.powi(2))
    }
}

impl Observer<'_, f64> for CentralMomentsSummary {
    fn observe(&mut self, &value: &f64) -> Result<(), failure::Error> {
        self.count += 1;
        let delta = value - self.mean.unwrap_or_default();
        let delta_n = delta / self.count as f64;
        let delta2 = delta * delta;
        let delta_n2 = delta_n * delta_n;
        let mean = self.mean.get_or_insert(0.0);
        let moment2 = self.moment2.get_or_insert(0.0);
        let moment3 = self.moment3.get_or_insert(0.0);
        let moment4 = self.moment4.get_or_insert(0.0);
        *mean += delta_n;
        *moment2 += delta * (delta - delta_n);
        *moment3 += -3.0 * delta_n * *moment2 + delta * (delta2 - delta_n2);
        *moment4 += -4.0 * delta_n * *moment3 - 6.0 * delta_n2 * *moment2
            + delta * (delta * delta2 - delta_n * delta_n2);
        Ok(())
    }
}

#[test]
fn test_central_moments_summary() -> Result<(), failure::Error> {
    // Test the same behavior as Google Sheets and scipy.stats.kurtosis.
    fn approx_eq(a: f64, b: f64) -> bool {
        const THRESHOLD: f64 = 0.001;
        (a - b).abs() < THRESHOLD
    }

    let mut summary = CentralMomentsSummary::default();
    summary.observe_many([-1.25, 6.25, 16.0, -6.25, 1.25, 8.0].iter())?;
    assert_eq!(summary.mean(), Some(4.0));
    assert_eq!(summary.variance(), Some(61.05));
    assert_eq!(summary.population_variance(), Some(50.875));
    assert!(approx_eq(
        summary.standard_deviation().unwrap(),
        7.813449942
    ));
    assert!(approx_eq(
        summary.population_standard_deviation().unwrap(),
        7.132671309
    ));
    assert!(approx_eq(summary.skewness().unwrap(), 0.3528219643));
    assert!(approx_eq(
        summary.population_skewness().unwrap(),
        0.2576647315
    ));
    println!("{:?}", summary.kurtosis());
    println!("{:?}", summary);
    assert!(approx_eq(summary.kurtosis().unwrap(), 2.92392));
    println!("{:?}", summary.population_kurtosis());
    assert!(approx_eq(summary.population_kurtosis().unwrap(), 2.11677));

    Ok(())
}

#[derive(Debug, Default)]
pub struct DistributionSummary {
    min: Option<f64>,
    max: Option<f64>,
    central_moments_summary: CentralMomentsSummary,
}

impl DistributionSummary {
    pub fn min(&self) -> Option<f64> {
        self.min
    }

    pub fn max(&self) -> Option<f64> {
        self.max
    }

    pub fn count(&self) -> u64 {
        self.central_moments_summary.count()
    }

    pub fn mean(&self) -> Option<f64> {
        self.central_moments_summary.mean()
    }

    pub fn variance(&self) -> Option<f64> {
        self.central_moments_summary.variance()
    }

    pub fn standard_deviation(&self) -> Option<f64> {
        self.central_moments_summary.standard_deviation()
    }

    pub fn skewness(&self) -> Option<f64> {
        self.central_moments_summary.skewness()
    }

    pub fn kurtosis(&self) -> Option<f64> {
        self.central_moments_summary.kurtosis()
    }

    pub fn population_variance(&self) -> Option<f64> {
        self.central_moments_summary.population_variance()
    }

    pub fn population_standard_deviation(&self) -> Option<f64> {
        self.central_moments_summary.population_standard_deviation()
    }

    pub fn population_skewness(&self) -> Option<f64> {
        self.central_moments_summary.population_skewness()
    }

    pub fn population_kurtosis(&self) -> Option<f64> {
        self.central_moments_summary.population_kurtosis()
    }
}

impl Observer<'_, f64> for DistributionSummary {
    fn observe(&mut self, &value: &f64) -> Result<(), failure::Error> {
        self.min = Some(self.min.map_or(value, |min| min.min(value)));
        self.max = Some(self.max.map_or(value, |max| max.max(value)));
        self.central_moments_summary.observe(&value)?;
        Ok(())
    }
}

impl fmt::Display for DistributionSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Count: {}\nMinimum: {}\nMaximum: {}\nMean: {}\nVariance: {}\nStandard deviation: \
             {}\nSkewness: {}\nKurtosis: {}\nPopulation variance: {}\nPopulation standard \
             deviation: {}\nPopulation skewness: {}\nPopulation kurtosis: {}",
            self.count(),
            self.min().unwrap_or(std::f64::NAN),
            self.max().unwrap_or(std::f64::NAN),
            self.mean().unwrap_or(std::f64::NAN),
            self.variance().unwrap_or(std::f64::NAN),
            self.standard_deviation().unwrap_or(std::f64::NAN),
            self.skewness().unwrap_or(std::f64::NAN),
            self.kurtosis().unwrap_or(std::f64::NAN),
            self.population_variance().unwrap_or(std::f64::NAN),
            self.population_standard_deviation()
                .unwrap_or(std::f64::NAN),
            self.population_skewness().unwrap_or(std::f64::NAN),
            self.population_kurtosis().unwrap_or(std::f64::NAN),
        )
    }
}

#[test]
fn test_distribution_summary() -> Result<(), failure::Error> {
    let mut summary = DistributionSummary::default();
    assert_eq!(summary.mean(), None);
    assert_eq!(summary.max(), None);
    summary.observe(&8.25)?;
    assert_eq!(summary.min(), Some(8.25));
    assert_eq!(summary.max(), Some(8.25));
    summary.observe(&-1.5)?;
    assert_eq!(summary.min(), Some(-1.5));
    assert_eq!(summary.max(), Some(8.25));
    Ok(())
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
