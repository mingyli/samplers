use crate::summary::Observer;
use crate::SamplersError;

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

impl Observer<'_, f64> for Histogram {
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
