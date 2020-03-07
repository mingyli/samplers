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
}
