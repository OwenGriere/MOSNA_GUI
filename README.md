<h1 align="center">A graphical interface for <a href="https://github.com/PancaldiLab/MOSNA">MOSNA</a> and <a href="https://github.com/PancaldiLab/tysserand">Tysserand</a> — spatial network construction and analysis tools developed by <b>PancaldiLAB</b></h1>

---

The interface is a Rust program. The analyses are not: Tysserand builds the
networks and MOSNA computes the assortativity and the niches, in Python,
unchanged — the interface starts them as sub-processes, so they can still be
run from a terminal without it. What the analyses no longer do is *draw*: each
one describes its figures, and the `mosna_xy` renderer turns every description
into a PNG and an interactive chart from the same data.

---

## Table of contents

- [Quick start](#quick-start)
  - [Linux / macOS](#linux--macos)
  - [Windows](#windows)
- [Launching the app](#launching-the-app)
- [Requirements](#requirements)
- [Tool overview](#tool-overview)
  - [Architecture](#architecture)
  - [Workflow](#workflow)
  - [Input data format](#input-data-format)
  - [Step 0 — Project setup](#step-0--project-setup)
  - [Step 1 — Tysserand spatial networks](#step-1--tysserand-spatial-networks)
  - [Step 2 — Assortativity](#step-2--assortativity)
  - [Step 3 — Niche Analysis](#step-3--niche-analysis)
- [Figures](#figures)
- [CLI usage](#cli-usage)
- [Configuration reference](#configuration-reference)
- [Developing](#developing)

---

## Quick start

### Linux / macOS

> **Requirements:** [Miniconda](https://docs.conda.io/en/latest/miniconda.html)
> or Anaconda, and the [Rust toolchain](https://rustup.rs).

Careful: accept all of conda's channel terms first — create a throw-away
environment and cancel before it proceeds — or the installer stops half way
through to ask a licensing question and leaves the environment incomplete.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # once
git clone <repo-url> MOSNA_GUI
cd MOSNA_GUI
bash setup.sh
```

Options:

| Flag | Effect |
|---|---|
| `--no-shortcut` | Skip desktop launcher creation |
| `--no-rust` | Build the conda environment only, do not compile the interface |
| `--dev` | Install `mosna-package` in editable mode |

The script will:
1. Create a `mosna-GUI` conda environment (Python 3.11)
2. Install all scientific dependencies via conda-forge
3. Install `mosna-package` and the `mosna_xy` renderer into the environment
4. Compile the interface with `cargo build --release`
5. Generate `MosnaGUI.sh` and a Desktop shortcut

### Windows

> **Requirement:** the [Rust toolchain](https://rustup.rs). Miniconda is
> downloaded automatically if absent.

1. Download or clone this repository
2. Double-click `setup_windows.bat`

---

## Launching the app

| Platform | Command |
|---|---|
| Linux / macOS | `bash MosnaGUI.sh` or double-click the Desktop shortcut |
| Windows | Double-click `MosnaGUI.bat` or the Desktop shortcut |

The launcher activates the conda environment before starting the interface,
which is what puts the analyses' interpreter first on `PATH`. Two environment
variables override what the interface finds on its own:

| Variable | Effect |
|---|---|
| `MOSNA_PYTHON` | The interpreter the analyses are run with |
| `MOSNA_GUI_ROOT` | The directory holding `package/` |
| `MOSNA_XY_PYTHON` | The interpreter the figures are drawn with, when it is not the one running the analyses |

---

## Requirements

| Dependency | Version |
|---|---|
| Rust | 1.82 or newer |
| Python | 3.11 — `xy` requires it, and the analyses and the renderer share one interpreter |
| scanpy | ≥ 1.9 |
| scipy | 1.13 |
| pandas | ≥ 2.0 |
| xy | 0.0.6 (pinned: it is alpha, and a patch release changes what the figures look like) |
| mosna | from `mosna-package/` |

Full list: [`requirements.txt`](requirements.txt)

---

## Tool overview

### Architecture

```
MOSNA_GUI/
├── Cargo.toml             ← the Rust workspace
├── crates/
│   ├── mosna-gui/             ← the interface: panels, viewer, manual, theme
│   ├── mosna-config/          ← configuration.yaml, read and written byte for byte
│   ├── mosna-io/              ← parquet / csv / tsv, and finding samples on disk
│   └── mosna-paths/           ← where the config, the sources and Python live
├── package/
│   ├── tysserand_network.py   ← Step 1 runner
│   ├── assortativity.py       ← Step 2 runner
│   ├── niche_analysis.py      ← Step 3 runner
│   ├── core/                  ← analysis cores, and the figures they describe
│   └── utils/                 ← shared utilities, the figure queue, the colours
├── python/
│   └── mosna_xy/          ← the renderer: one builder per kind of figure
├── CONFIG/
│   └── configuration.yaml.example   ← config template
├── mosna-package/         ← MOSNA Python package (unchanged)
├── setup.sh               ← Linux/macOS installer
└── setup_windows.bat      ← Windows installer
```

### Workflow

```
[Step 0]  Set working directory + data paths
    ↓
[Step 1]  Tysserand — Build spatial networks from cell coordinates
    ↓
[Step 2]  Assortativity — Compute mixing statistics per network
    ↓
[Step 3]  Niche Analysis — Cluster niches from aggregated neighborhoods
```

### Input data format

Your input files must be CSV or Parquet tables following this structure:

| CellID | patient | sample | X_position | Y_position | Phenotype |
|--------|---------|--------|------------|------------|-----------|
| c001   | pt-01   | s1     | 120.3      | 84.7       | CD8_T     |
| …      | …       | …      | …          | …          | …         |

`patient` and `sample` are not required, and you can add other attributes such
as marker intensities.

File naming convention expected by the tool:

```
nodes_patient-01_sample-s1.parquet
nodes_patient-02_sample-s1.parquet
```

---

### Step 0 — Project setup

| Parameter | Description |
|---|---|
| **Nodes directory** | Folder containing your spatial cell tables |
| **Network directory** | Folder with pre-built edges/nodes (default: auto-generated by Step 1) |
| **Patient column name** | Column used as the first grouping level (e.g. `patient`) |
| **Sample column name** | Optional second grouping level (e.g. `sample`) |
| **Extension** | File format: `csv`, `parquet`, or `tsv` |

---

### Step 1 — Tysserand spatial networks

| Parameter | Description |
|---|---|
| **X / Y coordinates column** | Spatial position columns |
| **Phenotype column** | Cell type or cluster column |
| **Edges method** | `delaunay` (triangulation) or `knn` (k-nearest neighbours) |
| **Min neighbors** | Minimum neighbours for KNN |
| **CPU** | Cores for parallel processing |

Step 1 writes no figure. Its only one was a picture of the network, and the
Viewer's **Network** tab draws the network itself — from the same files, at any
zoom, with every attribute still readable at the pointer.

---

### Step 2 — Assortativity

| Parameter | Description |
|---|---|
| **Phenotype column** | Column defining cell types |
| **Index** | Cell index column (`index` = DataFrame index) |
| **Number of shuffle** | Randomizations to build the null distribution |
| **Randomization diagnostic** | Estimate time cost for your shuffle count |

---

### Step 3 — Niche Analysis (NAS)

| Parameter | Description |
|---|---|
| **Saving directory** | Output subfolder name, under `Niche_Analysis/Aggregation` or `Niche_Analysis/Per_sample` |
| **Processing method** | `Aggregated nodes` or `Per sample` |
| **Niches method** | `NAS` (SCAN-IT is work in progress) |
| **Phenotype column** | Cell type column |
| **Column to aggregate** | Columns used in the aggregated network |
| **X / Y coordinates for niches** | Spatial columns to rebuild coloured networks |
| **CPU** | Cores for parallel processing |

**Clustering parameters:**

| Parameter | Description |
|---|---|
| `order` | Neighborhood order (1 = direct neighbors) |
| `stat_funcs` | Aggregation functions: `np.mean`, `np.std`, … |
| `clusterer_type` | `gmm`, `leiden`, `hdbscan`, `spectral`, `ecg` |
| `metric` | Distance metric: `euclidean`, `manhattan`, `cosine` |
| `normalize` | Feature normalization: `total`, `niche`, `obs`, `clr`, `all` |
| `reducer_type` | Dimensionality reduction: `umap`, or `none` |
| `n_neighbors` | Neighbors for graph/UMAP construction |
| `min_dist` | UMAP tightness (smaller = more compact) |
| `dim_clust` | Reduced dimensions used for clustering |
| `n_clusters` | Number of clusters (gmm, spectral) |
| `resolution` | Leiden resolution (higher = more clusters) |
| `min_cluster_size` | HDBSCAN minimum cluster size |

---

## Figures

The analyses no longer call matplotlib. Each one writes a *specification* per
figure — the values, the labels, the colours, the file to write — into
`<working dir>/.mosna-figures/`, and then runs `python -m mosna_xy render` over
the lot. Every figure comes out twice, as a `.png` and as a `.html` chart that
can be panned, zoomed and read values off.

The queue survives the process that wrote it, so a figure that came out wrong
can be redrawn from the exact input that produced it, without re-running an
analysis that took an hour:

```bash
python -m mosna_xy render /path/to/working_dir/.mosna-figures --formats png,html
```

A failed render keeps the queue and says so; a successful one removes it.

---

## CLI usage

The analyses are ordinary Python modules and can be run without the interface.

```bash
conda activate mosna-GUI
cd /path/to/MOSNA_GUI

python -m package.tysserand_network \
    --file CONFIG/configuration.yaml \
    --working_dir /path/to/output/

python -m package.assortativity \
    --file CONFIG/configuration.yaml \
    --working_dir /path/to/output/

python -m package.niche_analysis \
    --file CONFIG/configuration.yaml \
    --working_dir /path/to/output/
```

---

## Configuration reference

Copy [`CONFIG/configuration.yaml.example`](CONFIG/configuration.yaml.example)
to `CONFIG/configuration.yaml` and fill in your paths. The
`configuration.yaml` file is git-ignored (it contains absolute paths specific
to your machine).

The interface reads and writes that file byte for byte the way PyYAML wrote it,
so a configuration is interchangeable between the interface and the command
line.

---

## Developing

```bash
cargo test --workspace       # the interface, its model and its manual
cargo build --release        # what setup.sh runs
cargo fmt --all              # formatting
cargo clippy --workspace     # lints
```

The interface is split so that everything except the drawing is testable:
`model/` holds the logic — which widget a key gets, how its value reads back,
how a log line is classified, which figures belong to which patient — `panels/`
draws it, and `docs/` is the manual, as data the interface renders itself. The
manual is checked against the code: a parameter the interface offers and the
manual does not explain fails the test suite, and so does a dependency added
without being credited.
