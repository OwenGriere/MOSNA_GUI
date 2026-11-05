//! Typed view of the `Tysserand` section.

use crate::error::Result;
use crate::model::raw::RawConfig;
use crate::section;
use crate::value::{get_int, get_opt_str, get_str};

/// Method used to draw the edges of a spatial network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgesMethod {
    Delaunay,
    Knn,
}

impl EdgesMethod {
    pub fn parse(s: &str) -> Self {
        match s {
            "knn" => EdgesMethod::Knn,
            // `delaunay` is the default in the shipped configuration, and the
            // Python `link_solitaries` falls back to it for any other value.
            _ => EdgesMethod::Delaunay,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            EdgesMethod::Delaunay => "delaunay",
            EdgesMethod::Knn => "knn",
        }
    }
}

/// Key of the per-node trimming ratio (`trim_dist_ratio` in tysserand).
pub const TRIM_DISTANCE_RATIO: &str = "Trim distance ratio";
/// Key of the reference edge rank (`n_edges` in tysserand).
pub const TRIM_NUMBER_OF_EDGES: &str = "Trim number of edges";
/// Key of the length below which no edge is trimmed (`min_dist` in tysserand).
pub const TRIM_MIN_DISTANCE: &str = "Trim min distance";

/// The three Delaunay trimming keys, in the order the interface shows them.
pub const TRIM_KEYS: [&str; 3] = [TRIM_DISTANCE_RATIO, TRIM_NUMBER_OF_EDGES, TRIM_MIN_DISTANCE];

/// tysserand 0.6.0's `build_delaunay` defaults, which an absent key falls back
/// to: `trim_dist_ratio=2`, `n_edges=3`, `min_dist=0`.
pub const DEFAULT_TRIM_DISTANCE_RATIO: f64 = 2.0;
pub const DEFAULT_TRIM_NUMBER_OF_EDGES: usize = 3;
pub const DEFAULT_TRIM_MIN_DISTANCE: f64 = 0.0;

/// Parameters driving step 1, the spatial network reconstruction.
#[derive(Debug, Clone)]
pub struct TysserandConfig {
    pub nodes_directory: String,
    pub patient_column: String,
    /// `None` when the dataset has a single identifier level.
    pub sample_column: Option<String>,
    pub extension: String,
    pub x_column: String,
    pub y_column: String,
    pub phenotype_column: String,
    pub edges_method: EdgesMethod,
    pub min_neighbors: usize,
    pub cpu: usize,
}

impl TysserandConfig {
    /// Extract and type-check the section.
    pub fn from_raw(config: &RawConfig) -> Result<Self> {
        let s = config.section(section::TYSSERAND)?;
        let name = section::TYSSERAND;
        Ok(Self {
            nodes_directory: get_str(s, name, "Nodes directory")?,
            patient_column: get_str(s, name, "Patient column name")?,
            sample_column: get_opt_str(s, "Sample column name"),
            extension: get_str(s, name, "Extension")?,
            x_column: get_str(s, name, "X coordinates column")?,
            y_column: get_str(s, name, "Y coordinates column")?,
            phenotype_column: get_str(s, name, "Phenotype column")?,
            edges_method: EdgesMethod::parse(&get_str(s, name, "Edges method")?),
            min_neighbors: get_int(s, name, "Min neighbors")?.max(0) as usize,
            cpu: get_int(s, name, "CPU")?.max(1) as usize,
        })
    }
}
