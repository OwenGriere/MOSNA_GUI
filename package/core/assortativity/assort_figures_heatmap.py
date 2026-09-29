"""Every sample's z-scores, clustered on both axes.

Rows are phenotype pairs, columns are samples, and both are ordered by Ward
clustering — the tree that produced each order is drawn beside it.
"""

import numpy as np
from scipy.cluster.hierarchy import dendrogram, linkage
from scipy.spatial.distance import pdist

from ...utils.colours import resample
from ...utils.dendrogram import segments
from ...utils.figure_queue import Spec, queue
from matplotlib.colors import TwoSlopeNorm


def assort_figures_heatmap(net_stat, save_dir, working_dir, homo_pair=False):
    assort_cols = net_stat.columns[net_stat.columns.str.endswith(" Z")]
    assort_cols = [col for col in assort_cols if col != "assort Z"]

    if not homo_pair:
        # A phenotype's affinity for itself is not a pair; it is the largest
        # number in the matrix and takes the colour range with it.
        assort_cols = [
            col
            for col in assort_cols
            if col[:-2].split(" - ")[0] != col[:-2].split(" - ")[-1]
        ]

    heatmap_data = net_stat[assort_cols].T.astype(float)
    heatmap_data = heatmap_data.replace([np.inf, -np.inf], np.nan)
    heatmap_data = heatmap_data.dropna(axis=0, how="all")
    heatmap_data = heatmap_data.dropna(axis=1, how="all")

    if heatmap_data.empty:
        raise ValueError("The matrix holds no usable value.")

    col_linkage = linkage(
        pdist(heatmap_data.T.fillna(0), metric="euclidean"), method="ward"
    )
    row_linkage = linkage(
        pdist(heatmap_data.fillna(0), metric="euclidean"), method="ward"
    )

    col_leaf_order = dendrogram(col_linkage, no_plot=True)["leaves"]
    row_leaf_order = dendrogram(row_linkage, no_plot=True)["leaves"]

    heatmap_data = heatmap_data.iloc[row_leaf_order, col_leaf_order]

    values = heatmap_data.to_numpy(dtype=float)
    valid = values[~np.isnan(values)]
    if valid.size == 0:
        raise ValueError("The matrix holds no usable value.")

    # Symmetric about zero, so a positive and a negative z-score of the same
    # size read as equally strong.
    zlim = float(np.nanmax(np.abs(valid)))
    norm = TwoSlopeNorm(vmin=-zlim, vcenter=0, vmax=zlim)

    stem = (
        "Assortativity_heatmap_with_dendrogram"
        if homo_pair
        else "Assortativity_heatmap_with_dendrogram_without_auto_paired_pheno"
    )

    spec = (
        Spec("assortativity_heatmap", stem, save_dir)
        .set_array("z", values)
        # The ` Z` suffix is what makes a column an assortativity score, not
        # part of the pair's name.
        .set("y_labels", [str(index)[:-2] for index in heatmap_data.index])
        .set("x_labels", [str(column) for column in heatmap_data.columns])
        .set("colormap", resample("RdBu_r", norm, -zlim, zlim))
        .set("domain", [-zlim, zlim])
        .set_array("row_dendrogram", segments(row_linkage, row_leaf_order))
        .set_array("column_dendrogram", segments(col_linkage, col_leaf_order))
        .set("title", "Assortativity heatmap by images")
    )
    queue(spec, working_dir)
