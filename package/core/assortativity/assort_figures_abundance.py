"""How much of each phenotype every sample holds.

Stacked, because the question the figure answers is what a sample is *made of*
— the bands have to add up to the whole for that to be readable at a glance.
"""

import numpy as np

from ...utils.colours import hex_of, colormap
from ...utils.figure_queue import Spec, queue


def _palette(n_colors):
    """The `tab20` family, extended the way the matplotlib original extended it.

    Twenty colours, then `tab20b` and `tab20c` as the cohort needs them, in the
    same order — so a phenotype keeps the colour it had.
    """
    palettes = [colormap("tab20")(np.linspace(0, 1, 20))]
    if n_colors > 20:
        palettes.append(colormap("tab20b")(np.linspace(0, 1, 20)))
    if n_colors > 40:
        palettes.append(colormap("tab20c")(np.linspace(0, 1, 20)))

    colours = np.vstack(palettes)[:n_colors]
    return [hex_of(colour) for colour in colours]


def assort_figures_abundance(net_stat, save_dir, working_dir):
    plot_df = net_stat.loc[:, net_stat.columns.str.startswith('% ')]
    plot_df = plot_df.div(plot_df.sum(axis=1), axis=0)

    if plot_df.empty:
        return

    # One row per phenotype, one column per sample: the renderer stacks the
    # rows, so the matrix is handed over in that orientation.
    values = plot_df.to_numpy(dtype=float).T
    # `% ` prefixes the column names; the legend wants the phenotype.
    phenotypes = [str(column)[2:] for column in plot_df.columns]
    samples = [str(index) for index in plot_df.index]

    spec = (
        Spec("abundance", "abundance", save_dir)
        .set_array("values", values)
        .set("samples", samples)
        .set("phenotypes", phenotypes)
        .set("colours", _palette(len(phenotypes)))
        .set("legend_title", "Cell type")
        .set("title", "Relative abundance of cell types per sample")
    )
    queue(spec, working_dir)
