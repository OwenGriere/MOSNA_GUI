"""Queueing figures instead of drawing them.

The analyses used to draw their own figures with matplotlib and seaborn. They
no longer draw anything: each one writes a *specification* — the values, the
labels, the colours, the title, the file to write — and `mosna_xy` composes an
`xy` chart from it and exports the PNG and the interactive HTML.

This is the arrangement MOSNA_Enhanced uses, and the reason is the same: a
static image cannot answer "what is that outlier" or "which sample is that
column", and those are the two questions these figures get asked. The science
does not move — every value here is computed exactly where it was before.

# Why a file and not a pipe

A queued specification survives the process that wrote it, so a figure that
came out wrong can be redrawn from the exact input that produced it, without
re-running an analysis that took an hour. The queue is removed once the
renderer has succeeded.

# Why the queue is flat

`draw_per_sample` runs inside a `ProcessPoolExecutor`, so several processes
queue at once. Each entry is named after the process that wrote it and a
counter within that process, which is unique without any coordination: two
processes cannot share a pid at the same time. One renderer invocation then
covers everything, whoever wrote it.
"""

from __future__ import annotations

import itertools
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import numpy as np

#: Where specifications wait, under the working directory.
#:
#: Dot-prefixed, and never one of the directories the interface scans for
#: figures: a queue showing up in the gallery as a folder of nothing would be a
#: bug report.
QUEUE_DIRECTORY = ".mosna-figures"

#: The document inside one queued figure's folder.
DOCUMENT_NAME = "figure.json"

#: What is written for each figure. The PNG is what the gallery shows; the HTML
#: is the chart that can be zoomed and hovered.
DEFAULT_FORMATS = ("png", "html")

#: How a blob's declared type maps onto a numpy one. Little-endian explicitly,
#: because that is what `mosna_xy.spec` reads.
DTYPES = {"f64": "<f8", "u32": "<u4"}

#: Hands out a distinct number to every specification this process queues.
_SEQUENCE = itertools.count()


class Spec:
    """One figure, ready to be drawn.

    Built by chaining, so a figure reads as one expression and a half-built
    specification never escapes into a variable.
    """

    def __init__(self, kind: str, stem: str, save_dir: Path):
        self.kind = str(kind)
        self.stem = str(stem)
        self.save_dir = Path(save_dir)
        self.body: dict = {}
        self.blobs: dict[str, bytes] = {}

    def set(self, key: str, value) -> "Spec":
        """Add a field to the body, unless it is nothing."""
        if value is not None:
            self.body[key] = value
        return self

    def set_array(self, key: str, values, dtype: str = "f64") -> "Spec":
        """Add an array as a binary blob beside the document.

        A hundred thousand cells' coordinates are two hundred thousand doubles.
        Spelled out as JSON that is some four megabytes to write and to parse,
        against 1.6 to read with `numpy.fromfile`. Small arrays are welcome to
        go through `set` and stay readable.
        """
        array = np.ascontiguousarray(np.asarray(values, dtype=DTYPES[dtype]))
        self.body[key] = {
            "__blob__": f"{key}.bin",
            "dtype": dtype,
            "shape": list(array.shape),
        }
        self.blobs[f"{key}.bin"] = array.tobytes()
        return self

    def to_json(self) -> dict:
        """The document the renderer parses."""
        return {
            **self.body,
            "kind": self.kind,
            "stem": self.stem,
            "save_dir": str(self.save_dir),
        }


def queue_directory(working_dir) -> Path:
    """Where the specifications of a run wait."""
    return Path(working_dir) / QUEUE_DIRECTORY


def queue(spec: Spec, working_dir) -> Path:
    """Write one specification, and its blobs, into the queue.

    Returns the folder it was written to.
    """
    folder = queue_directory(working_dir) / (
        f"{os.getpid():07d}-{next(_SEQUENCE):05d}-{spec.kind}"
    )
    folder.mkdir(parents=True, exist_ok=True)

    for name, payload in spec.blobs.items():
        (folder / name).write_bytes(payload)

    (folder / DOCUMENT_NAME).write_text(
        json.dumps(spec.to_json(), indent=2), encoding="utf-8"
    )
    return folder


def renderer_interpreter() -> str:
    """The interpreter `mosna_xy` is installed in.

    Normally this one: setup.sh puts the renderer in the same environment as
    the analyses, which is why that environment is built on Python 3.11 — the
    version `xy` requires. `MOSNA_XY_PYTHON` overrides it, for an environment
    that cannot move off an older Python and keeps the renderer beside it.
    """
    return os.environ.get("MOSNA_XY_PYTHON") or sys.executable


def pending(working_dir) -> list[Path]:
    """The specifications waiting to be drawn."""
    directory = queue_directory(working_dir)
    if not directory.is_dir():
        return []
    return [
        folder
        for folder in sorted(directory.iterdir())
        if (folder / DOCUMENT_NAME).is_file()
    ]


def render(working_dir, formats=DEFAULT_FORMATS) -> int:
    """Draw everything queued, then remove the queue.

    An empty queue starts nothing: a run that found no samples ends up here
    with nothing to say.

    Returns the number of specifications that were handed to the renderer.

    # Why a figure that will not draw is a warning and not a failure

    The tables and the labels are written before this runs, and a run that
    computed for an hour must not be reported as failed because a chart could
    not be exported. So both ways of not drawing — no renderer installed, or a
    renderer that refused a figure — are reported and stepped over, and the
    queue is left where it is so the figures can be drawn afterwards without
    recomputing anything.
    """
    waiting = pending(working_dir)
    if not waiting:
        return 0

    directory = queue_directory(working_dir)
    command = [
        renderer_interpreter(),
        "-m",
        "mosna_xy",
        "render",
        str(directory),
        "--formats",
        ",".join(formats),
    ]

    try:
        completed = subprocess.run(command, check=False)
    except OSError as error:
        print(f"[WARN] the figures were not drawn: {error}", file=sys.stderr)
        return len(waiting)

    if completed.returncode == 0:
        shutil.rmtree(directory, ignore_errors=True)
    else:
        # The queue is deliberately kept: it is the exact input that failed,
        # and `python -m mosna_xy render` over it is how the failure is looked
        # into without recomputing anything.
        print(
            f"[WARN] some figures could not be drawn; their specifications are "
            f"kept in {directory}",
            file=sys.stderr,
        )
    return len(waiting)
