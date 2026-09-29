"""Colours, in the form the renderer takes them.

`xy` maps a value onto a colour by interpolating linearly between the stops it
is given, over the domain it is given. Every normalisation these figures use —
the diverging one centred on zero, the symmetric-log one — is therefore
expressed *as stops*: resampling the colour map through the normalisation
reproduces it to within a quantisation step, and leaves the renderer with real
data values on its axes and its colour bar rather than a pre-normalised
`[0, 1]` nobody can read.

The maps themselves still come from matplotlib, so the figures keep exactly the
colours they had. Only `matplotlib.colors` and `matplotlib.cm` are imported —
never `pyplot`, which would load a drawing backend for a lookup table.
"""

from __future__ import annotations

import matplotlib
import numpy as np
from matplotlib import cm
from matplotlib.colors import Normalize, SymLogNorm, TwoSlopeNorm

#: How many stops a resampled map carries. Enough that the quantisation is
#: below what a screen can show.
STOPS = 256

#: The colour of a cell with no value, and the one `cmap.set_bad` used.
MISSING = "#888888"


def hex_of(rgba) -> str:
    """One matplotlib colour as the renderer spells it."""
    r, g, b = (int(round(channel * 255)) for channel in rgba[:3])
    return f"#{r:02x}{g:02x}{b:02x}"


def linear(name: str, stops: int = STOPS) -> list[str]:
    """A named map, evenly sampled: for a scale that is already linear."""
    stops = max(stops, 2)
    table = colormap(name)
    return [hex_of(table(index / (stops - 1))) for index in range(stops)]


def resample(name: str, norm: Normalize, vmin: float, vmax: float,
             stops: int = STOPS) -> list[str]:
    """A named map as seen *through* `norm`, over `[vmin, vmax]`.

    Stop `i` is the colour the normalisation gives to the value the renderer
    will place there, so reading the stops linearly reproduces the curve.
    """
    stops = max(stops, 2)
    table = colormap(name)
    span = vmax - vmin
    colours = []
    for index in range(stops):
        value = vmin + span * index / (stops - 1)
        fraction = float(np.clip(np.nan_to_num(norm(value), nan=0.0), 0.0, 1.0))
        colours.append(hex_of(table(fraction)))
    return colours


def colormap(name: str):
    """One named map, however the installed matplotlib offers them.

    `matplotlib.cm.get_cmap` was removed in 3.9 and `matplotlib.colormaps` did
    not exist before 3.5; the environment is pinned to neither, so both are
    tried rather than the figures failing on a colour lookup.
    """
    registry = getattr(matplotlib, "colormaps", None)
    if registry is not None:
        return registry[name]
    return cm.get_cmap(name)


def two_slope_on_zero(values) -> TwoSlopeNorm:
    """A diverging normalisation over `values`, centred on zero.

    Reproduces the guards the figures applied before building the norm: a range
    that does not straddle zero is widened, because `TwoSlopeNorm` requires
    `vmin < vcenter < vmax` and raises otherwise — which is what used to happen
    on a cohort whose z-scores were all positive.
    """
    finite = np.asarray(values, dtype=float).ravel()
    finite = finite[np.isfinite(finite)]

    if finite.size == 0:
        vmin, vmax = -1e-6, 1e-6
    else:
        vmin, vmax = float(finite.min()), float(finite.max())

    if vmin >= 0:
        vmin = -1e-6
    if vmax <= 0:
        vmax = 1e-6
    if vmin == vmax:
        vmin -= 1e-6
        vmax += 1e-6

    return TwoSlopeNorm(vmin=vmin, vcenter=0.0, vmax=vmax)


def symmetric_log(zlim: float) -> SymLogNorm:
    """The symmetric-log normalisation the mean-assortativity figure uses."""
    linthresh = max(0.1, zlim * 0.05)
    return SymLogNorm(linthresh=linthresh, linscale=1, vmin=-zlim, vmax=zlim, base=10)


def cluster_palette(labels) -> list[str]:
    """One colour per label, in the order the labels are given.

    `mosna.make_cluster_cmap` is what every other figure and the interactive
    network are coloured by, so a niche is the same colour everywhere. It
    returns fewer colours than there are niches on a large cohort and is cycled,
    exactly as `generate_cmap` cycles it.
    """
    from mosna import mosna

    palette = mosna.make_cluster_cmap(list(labels))
    return [palette[index % len(palette)] for index in range(len(labels))]
