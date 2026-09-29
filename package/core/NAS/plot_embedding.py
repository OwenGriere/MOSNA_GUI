"""The two-dimensional projection, coloured by niche.

Each niche's identifier is written at its centroid, which is what lets a blob
here be matched to a column of the composition heatmap.
"""

import numpy as np
import pandas as pd

from ...utils.colours import cluster_palette
from ...utils.figure_queue import Spec, queue


def plot_embedding(embedding_path, cluster_labels, save_dir, cluster_params,
                   working_dir):
    if not embedding_path.exists():
        return None

    embedding = np.load(embedding_path)
    if embedding.ndim != 2 or embedding.shape[0] == 0 or embedding.shape[1] < 2:
        return None

    labels = np.asarray(cluster_labels)
    # By frequency, as `mosna.plot_clusters` ordered them: the palette's first
    # and most saturated colours go to the niches with the most nodes.
    identifiers = pd.Series(labels).value_counts().index.tolist()
    index = np.full(len(labels), len(identifiers), dtype=np.uint32)
    centroids = []
    for position, identifier in enumerate(identifiers):
        selected = labels == identifier
        index[selected] = position
        centroids.append(
            [
                float(embedding[selected, 0].mean()),
                float(embedding[selected, 1].mean()),
            ]
        )

    # The file name carries the parameters the projection was made with, so two
    # runs in one directory do not overwrite each other — the same convention
    # `mosna.plot_clusters` followed.
    parameters = "".join(f"_{key}-{value}" for key, value in cluster_params.items())

    spec = (
        Spec("embedding", f"cluster_labels{parameters}", save_dir)
        .set_array("points", embedding[:, :2].astype(float))
        .set_array("clusters", index, dtype="u32")
        .set("cluster_ids", [str(identifier) for identifier in identifiers])
        .set("colours", cluster_palette(identifiers))
        .set_array("centroids", np.asarray(centroids, dtype=float))
        .set("legend_title", "Niche")
        .set("title", "Niches on the 2D projection")
    )
    queue(spec, working_dir)
    return None
