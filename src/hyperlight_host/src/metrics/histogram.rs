use prometheus::{register_histogram_with_registry, Histogram as PHistogram};

use super::{
    get_histogram_opts, get_metrics_registry, GetHyperlightMetric, HyperlightMetric,
    HyperlightMetricOps,
};
use crate::{new_error, HyperlightError, Result};

/// A named histogram
#[derive(Debug)]
pub struct Histogram {
    histogram: PHistogram,
    /// The name of the histogram
    pub name: &'static str,
}

impl Histogram {
    /// Creates a new histogram and registers it with the metric registry
    pub fn new(name: &'static str, help: &str, buckets: Vec<f64>) -> Result<Self> {
        let registry = get_metrics_registry();
        let opts = get_histogram_opts(name, help, buckets);
        let histogram = register_histogram_with_registry!(opts, registry)?;
        Ok(Self { histogram, name })
    }
    /// Observes a value for a Histogram
    pub fn observe(&self, val: f64) {
        self.histogram.observe(val)
    }
    /// Gets the sum of values of an Histogram
    pub fn get_sample_sum(&self) -> f64 {
        self.histogram.get_sample_sum()
    }
    /// Gets the count of values of an Histogram
    pub fn get_sample_count(&self) -> u64 {
        self.histogram.get_sample_count()
    }
}

impl<S: HyperlightMetricOps> GetHyperlightMetric<Histogram> for S {
    fn metric(&self) -> Result<&Histogram> {
        let metric = self.get_metric()?;
        <&HyperlightMetric as TryInto<&Histogram>>::try_into(metric)
    }
}

impl<'a> TryFrom<&'a HyperlightMetric> for &'a Histogram {
    type Error = HyperlightError;
    fn try_from(metric: &'a HyperlightMetric) -> Result<Self> {
        match metric {
            HyperlightMetric::Histogram(histogram) => Ok(histogram),
            _ => Err(new_error!("metric is not a Histogram")),
        }
    }
}

impl From<Histogram> for HyperlightMetric {
    fn from(histogram: Histogram) -> Self {
        HyperlightMetric::Histogram(histogram)
    }
}

/// Observes a value for a Histogram
#[macro_export]
macro_rules! histogram_observe {
    ($metric:expr, $val:expr) => {{
        match $crate::metrics::GetHyperlightMetric::<$crate::metrics::Histogram>::metric($metric) {
            Ok(val) => {
                if let Err(e) = val.observe($val) {
                    log::error!("error calling observe with value {} on metric {} ", $val, e,)
                }
            }
            Err(e) => log::error!("error getting metric: {}", e),
        };
    }};
}

/// Gets the sum of values of an Histogram or logs an error if the metric is not found
/// Returns 0.0 if the metric is not found
#[macro_export]
macro_rules! histogram_sample_sum {
    ($metric:expr) => {{
        match $crate::metrics::GetHyperlightMetric::<$crate::metrics::Histogram>::metric($metric) {
            Ok(val) => match val.get_sample_sum() {
                Ok(val) => val,
                Err(e) => {
                    log::error!("error getting samples sum of metric {}", e,);
                    0.0
                }
            },

            Err(e) => {
                log::error!("error getting metric: {}", e);
                0.0
            }
        }
    }};
}

/// Gets the count of values of an Histogram or logs an error if the metric is not found
/// Returns 0 if the metric is not found
#[macro_export]
macro_rules! histogram_sample_count {
    ($metric:expr) => {{
        match $crate::metrics::GetHyperlightMetric::<$crate::metrics::Histogram>::metric($metric) {
            Ok(val) => match val.get_sample_count() {
                Ok(val) => val,
                Err(e) => {
                    log::error!("error getting samples count of metric {}", e,);
                    0
                }
            },

            Err(e) => {
                log::error!("error getting metric: {}", e);
                0
            }
        }
    }};
}
/// Observe the time it takes to execute an expression, record that time in microseconds in a Histogram and return the result of that expression
#[macro_export]
macro_rules! histogram_time_micros {
    ($metric:expr, $expr:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        histogram_observe!($metric, start.elapsed().as_micros() as f64);
        result
    }};
}