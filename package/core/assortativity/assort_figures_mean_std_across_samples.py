"""The mean assortativity across samples, with its uncertainty.

Colour is the mean, the size of the square is the standard error. Both on one
grid, which is the whole reason this is one figure and not two: a strong mean
measured once and a weak mean measured forty times must not look alike.
"""

import re

import numpy as np
import pandas as pd

from ...utils.colours import resample, symmetric_log
from ...utils.figure_queue import Spec, queue

#: The largest and smallest square, as `xy` sizes a marker. The original drew
#: cells of 0.85 and 0.15 of a grid step; these are the same ratio in pixels.
SIZE_MAX = 26.0
SIZE_MIN = 5.0

#: Splits `CD8_T - Treg Z` into its two phenotypes.
PAIR = re.compile(r'^(.+?) - (.+?) Z$')


def assort_figures_mean_std_across_samples(net_stat, save_dir, working_dir,
                                           homo_pair=False):
    assort_cols = net_stat.columns[net_stat.columns.str.endswith(" Z")]
    assort_cols = [col for col in assort_cols if col != "assort Z"]

    net_stat = net_stat.copy()
    if not homo_pair:
        for col in assort_cols:
            pheno1, pheno2 = col.split(' - ', maxsplit=1)
            if pheno1 == pheno2[:-2]:
                net_stat[col] = np.nan

    assort = net_stat[assort_cols].copy()
    assort = assort.replace([np.inf, -np.inf], np.nan)
    assort = assort.dropna(axis=1, how='all')
    assort = assort.loc[:, assort.std() > 0]

    mean_assort = assort.mean(axis=0)
    sem_assort = assort.sem(axis=0)

    celltypes = sorted({
        celltype
        for col in mean_assort.index
        for match in [PAIR.match(col)]
        if match
        for celltype in (match.group(1).strip(), match.group(2).strip())
    })

    matrix_mean = pd.DataFrame(np.nan, index=celltypes, columns=celltypes)
    matrix_sem = pd.DataFrame(np.nan, index=celltypes, columns=celltypes)

    for col in mean_assort.index:
        match = PAIR.match(col)
        if not match:
            continue
        ct1, ct2 = match.group(1).strip(), match.group(2).strip()
        if ct1 not in celltypes or ct2 not in celltypes:
            continue
        # The measure is symmetric: a pair is one number, written into both
        # halves so the matrix reads the same from either axis.
        matrix_mean.loc[ct1, ct2] = mean_assort[col]
        matrix_mean.loc[ct2, ct1] = mean_assort[col]
        matrix_sem.loc[ct1, ct2] = sem_assort[col]
        matrix_sem.loc[ct2, ct1] = sem_assort[col]

    if matrix_mean.notna().sum().sum() == 0:
        raise ValueError("matrix_mean holds no value at all.")

    means = matrix_mean.to_numpy(dtype=float)
    sems = matrix_sem.to_numpy(dtype=float)

    # Symmetric log, because the z-scores span orders of magnitude and a linear
    # scale leaves everything but the strongest pair the same colour.
    zlim = float(np.nanmax(np.abs(means)))
    norm = symmetric_log(zlim)

    stem = (
        "Assortativity_heatmap_across_patient"
        if homo_pair
        else "Assortativity_heatmap_across_patient_without_auto_paired_pheno"
    )

    spec = (
        Spec("assortativity_mean_std", stem, save_dir)
        .set_array("z", means)
        .set_array("sizes", _sizes(sems))
        .set("labels", celltypes)
        .set("colormap", resample("RdBu_r", norm, -zlim, zlim))
        .set("domain", [-zlim, zlim])
        .set("title", "Mean assortativity + SEM across samples")
    )
    queue(spec, working_dir)


def _sizes(sems):
    """Square size from the standard error: the more certain, the larger.

    Inverted on purpose, and the same way the original inverted it — a small
    error is a measurement worth looking at, so it takes up more of its cell.
    """
    finite = sems[np.isfinite(sems)]
    if finite.size == 0:
        return np.full(sems.shape, SIZE_MAX)

    low, high = float(finite.min()), float(finite.max())
    if high == low:
        return np.where(np.isfinite(sems), SIZE_MAX, 0.0)

    fraction = (sems - low) / (high - low)
    return np.where(
        np.isfinite(sems),
        SIZE_MAX - fraction * (SIZE_MAX - SIZE_MIN),
        0.0,
    )
