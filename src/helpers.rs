use crate::defaults::DimVec;
use std::{
    cmp::Ordering::{self, Equal, Greater, Less},
    fs::File,
    path::Path,
    str::FromStr,
};

pub fn add(a: &DimVec<f32>, b: &DimVec<f32>) -> DimVec<f32> {
    a.iter().zip(b).map(|(aa, bb)| aa + bb).collect()
}

pub fn add_to(a: &mut DimVec<f32>, b: &DimVec<f32>) {
    a.iter_mut().zip(b).for_each(|(aa, bb)| *aa += bb);
}

pub fn dot_product(a: &DimVec<f32>, b: &DimVec<f32>) -> f32 {
    a.iter()
        .zip(b)
        .fold(0.0, |start, (aa, &bb)| start + aa * bb)
}

pub trait ApproxEq {
    fn approx_eq(&self, other: &Self) -> bool;
}

pub trait ApproxCmp {
    fn approx_partial_cmp(&self, other: &Self) -> Option<Ordering>;
    fn approx_cmp(&self, other: &Self) -> Ordering;
}

impl ApproxEq for f32 {
    fn approx_eq(&self, other: &f32) -> bool {
        (self - other).abs() <= std::f32::EPSILON
    }
}

impl ApproxCmp for f32 {
    fn approx_partial_cmp(&self, other: &f32) -> Option<Ordering> {
        match (self < other, self > other, self.approx_eq(other)) {
            (false, false, false) => None,
            (false, true, false) => Some(Greater),
            (true, false, false) => Some(Less),
            (true, true, false) | (_, _, true) => Some(Equal),
        }
    }

    fn approx_cmp(&self, other: &f32) -> Ordering {
        self.approx_partial_cmp(other).expect(&format!(
            "No f32-comparison for {} and {} possible.",
            self, other
        ))
    }
}

impl ApproxEq for DimVec<f32> {
    fn approx_eq(&self, other: &DimVec<f32>) -> bool {
        self.iter()
            .zip(other)
            .fold(true, |acc, (aa, bb)| acc && aa.approx_eq(bb))
    }
}

pub fn open_file<P: AsRef<Path> + ?Sized>(path: &P) -> Result<File, String> {
    let path = path.as_ref();
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(_) => Err(format!("No such file {}", path.display())),
    }
}

pub fn open_new_file<P: AsRef<Path> + ?Sized>(path: &P) -> Result<File, String> {
    let path = path.as_ref();
    if path.exists() {
        return Err(format!(
            "Provided file {} does already exist. Please remove it.",
            path.display()
        ));
    }

    match File::create(path) {
        Ok(file) => Ok(file),
        Err(e) => Err(format!("{}", e)),
    }
}

/// Sets the logging-level of this repo.
///
/// max_log_level: None
/// => use default (Warn)
///
/// modules: in addition to default (`env!("CARGO_PKG_NAME")`)
///
/// Environment-variable RUST_LOG has precedence.
pub fn init_logging(max_log_level: &str, mut modules: Vec<&str>) -> Result<(), String> {
    let mut builder = env_logger::Builder::new();

    // maximum filter-level for all components: `warn`
    builder.filter(None, log::LevelFilter::Warn);

    // if quiet logging: doesn't log `info` for this repo
    let max_log_level = log::LevelFilter::from_str(&max_log_level.to_ascii_uppercase())
        .ok()
        .ok_or(format!(
            "The provided max-log-level {} is not supported.",
            max_log_level
        ))?;
    modules.push(env!("CARGO_PKG_NAME"));
    for module in modules {
        builder.filter(Some(module), max_log_level);
    }

    // overwrite default with environment-variables
    if let Ok(filters) = std::env::var("RUST_LOG") {
        builder.parse_filters(&filters);
    }
    if let Ok(write_style) = std::env::var("RUST_LOG_STYLE") {
        builder.parse_write_style(&write_style);
    }

    // init
    builder.init();

    // return
    Ok(())
}
