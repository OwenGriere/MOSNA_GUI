"""The tree drawn beside a clustered heatmap.

The renderer receives the tree as line segments in two coordinates it can place
without knowing anything about clustering: a position along the axis the tree
belongs to, and a merge height normalised to `[0, 1]`. The clustering itself is
`scipy.cluster.hierarchy`, exactly as before — only the drawing moved.
"""

from __future__ import annotations

import numpy as np


def segments(linkage_matrix, order) -> np.ndarray:
    """The tree, in the drawing order its heatmap was given.

    `order` is the leaf order the rows or columns were drawn in, so a leaf's
    position is where it actually sits on the axis. Each row of the result is
    `[position0, height0, position1, height1]`.

    Returns an `(n, 4)` array, empty when there is nothing to draw.
    """
    linkage_matrix = np.asarray(linkage_matrix, dtype=float)
    if linkage_matrix.size == 0:
        return np.empty((0, 4), dtype=float)

    n_leaves = linkage_matrix.shape[0] + 1
    if n_leaves < 2:
        return np.empty((0, 4), dtype=float)

    # Where each leaf was drawn. A leaf missing from the order — which cannot
    # happen, and would be a silent mislabelling if it did — keeps its own
    # index rather than collapsing onto zero.
    position = np.arange(n_leaves, dtype=float)
    for drawn, leaf in enumerate(order):
        if 0 <= leaf < n_leaves:
            position[leaf] = float(drawn)

    # The tallest merge is the root; every height is read against it, so the
    # tree fills the band it is drawn in whatever the distances happen to be.
    tallest = float(linkage_matrix[:, 2].max())
    scale = (lambda d: d / tallest) if tallest > 0 else (lambda d: 0.0)

    node_position = list(position)
    node_height = [0.0] * n_leaves

    lines = []
    for left, right, distance, _ in linkage_matrix:
        left, right = int(left), int(right)
        x_left, h_left = node_position[left], node_height[left]
        x_right, h_right = node_position[right], node_height[right]
        height = scale(float(distance))

        lines.append([x_left, h_left, x_left, height])
        lines.append([x_left, height, x_right, height])
        lines.append([x_right, height, x_right, h_right])

        node_position.append((x_left + x_right) / 2.0)
        node_height.append(height)

    return np.asarray(lines, dtype=float)
