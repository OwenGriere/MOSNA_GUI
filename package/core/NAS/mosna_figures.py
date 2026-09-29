"""What each niche is made of, and how many nodes fell into each.

Two figures per normalisation. The composition is a phenotype-by-niche matrix;
the histogram counts the nodes. Neither is drawn here — both are handed to the
renderer, which exports the PNG and the interactive chart from one description.
"""

import numpy as np

from ...utils.colours import cluster_palette, linear
from ...utils.figure_queue import Spec, queue


def mosna_figures(niches, counts, save_dir, working_dir, norm=None):
    suffix = f"_{norm}" if norm is not None else ""

    _composition(counts, save_dir, working_dir, suffix, norm)
    _histogram(niches, save_dir, working_dir)


def _composition(counts, save_dir, working_dir, suffix, norm):
    """The matrix, in the orientation `make_niches_composition` returns it:
    one row per phenotype, one column per niche."""
    values = counts.to_numpy(dtype=float)
    if values.size == 0:
        return

    spec = (
        Spec("niche_composition", f"Niches_Aggregated_Composition{suffix}", save_dir)
        .set_array("z", values)
        .set("y_labels", [str(index) for index in counts.index])
        .set("x_labels", [str(column) for column in counts.columns])
        # The Blues `plot_niches_composition` used. A sequential map, because a
        # composition is a proportion with a floor at zero and no natural
        # centre.
        .set("colormap", linear("Blues"))
        .set("domain", [float(np.nanmin(values)), float(np.nanmax(values))])
        .set("colorbar_title", norm or "proportion")
        .set("title", "Niches Aggregated Composition")
    )
    queue(spec, working_dir)


def _histogram(niches, save_dir, working_dir):
    """One bar per niche, in the niche's own colour.

    The colour is the point: it is the same one the niche has in the embedding
    and in the composition heatmap, so a tall bar here can be found again over
    there.
    """
    identifiers, counts = np.unique(np.asarray(niches), return_counts=True)
    if identifiers.size == 0:
        return

    categories = [str(identifier) for identifier in identifiers]
    spec = (
        Spec("histogram", "Niches_Histogram", save_dir)
        .set("categories", categories)
        .set_array("counts", counts.astype(float))
        .set("colours", cluster_palette(identifiers))
        .set("title", "Niches histogram")
    )
    queue(spec, working_dir)
