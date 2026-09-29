"""One sample's phenotype-by-phenotype mixing matrix.

Shared by the two figures that differ only in whether the diagonal is shown:
a phenotype's affinity for itself dominates the colour range, so the second
figure blanks it and lets the off-diagonal pairs use the whole scale.
"""

import numpy as np

from mosna import mosna

from ...utils.colours import resample, two_slope_on_zero
from ...utils.figure_queue import Spec, queue


def mixing_matrix_figures(net_stat, save_dir, sub_directory, is_sample,
                          working_dir, drop_diagonal):
    saving_folder = save_dir / sub_directory
    saving_folder.mkdir(parents=True, exist_ok=True)

    for identifier in net_stat.index:
        assort_df_Z = net_stat.loc[[identifier]]
        assort_df_Z = assort_df_Z[net_stat.columns[net_stat.columns.str.endswith(" Z")]]
        assort_Z = assort_df_Z['assort Z']

        if is_sample is not None:
            patient = assort_df_Z.index[0].split('-')[1].split('_')[0]
            sample = assort_df_Z.index[0].split('-')[2]
            stem = f"heatmap_zscore_{patient}-{sample}"
        else:
            patient = assort_df_Z.index[0].split('-')[1]
            stem = f"heatmap_zscore_{patient}"

        assort_df_Z = assort_df_Z.reset_index().drop(columns=['id', 'assort Z'])
        mat_Z = mosna.series_to_mixmat(assort_df_Z.iloc[0])

        mat_Z = mat_Z.astype(float)
        mat_Z = mat_Z.replace([np.inf, -np.inf], np.nan)
        mat_Z = mat_Z.dropna(axis=0, how="all")
        mat_Z = mat_Z.dropna(axis=1, how="all")

        if mat_Z.empty:
            continue

        # Copied, because a DataFrame backed by a single block hands out a
        # read-only view and writing the diagonal through it raises.
        values = mat_Z.to_numpy(dtype=float, copy=True)
        if drop_diagonal:
            np.fill_diagonal(values, np.nan)

        norm = two_slope_on_zero(values)

        spec = (
            Spec("mixing_matrix", stem, saving_folder)
            .set_array("z", values)
            .set("x_labels", [str(column) for column in mat_Z.columns])
            .set("y_labels", [str(index) for index in mat_Z.index])
            .set("colormap", resample("RdBu_r", norm, norm.vmin, norm.vmax))
            .set("domain", [norm.vmin, norm.vmax])
            .set(
                "title",
                "Z-score heatmap with a general assortativity: "
                f"{assort_Z.values[0]}",
            )
        )
        queue(spec, working_dir)
