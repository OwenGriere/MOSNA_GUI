"""Draw every figure set aside by the steps run without plotting.

The button "Plot all figures". Nothing is computed: each step started with
`--no-figures` wrote its tables as usual and moved the full description of
every figure into `.mosna-figures-deferred/`, one batch per run. They are drawn
here oldest first, so a file two runs describe ends up as the later one drew
it — and so is whatever a failed render left in the queue.

A batch is removed once drawn, so pressing the button twice draws nothing the
second time; one that fails is kept, as a failed queue always is, and the
others are drawn regardless.
"""

import argparse
import shutil
import sys
from pathlib import Path

from package.utils.emit_qt_progress import emit_qt_info
from package.utils.figure_queue import (
    deferred_batches,
    deferred_directory,
    draw_queue,
    pending,
    queue_directory,
)


def render_figures(working_dir) -> int:
    working_dir = Path(working_dir)

    # What a failed render kept in the queue is older than anything set aside
    # since, or it would have been moved with it: it is drawn first.
    batches = ([queue_directory(working_dir)] if pending(working_dir) else []) + deferred_batches(
        working_dir
    )
    if not batches:
        emit_qt_info(
            "[INFO] No figures are waiting to be drawn: every analysis run in this "
            "directory has already drawn its own."
        )
        return 0

    failed = []
    for index, batch in enumerate(batches, start=1):
        emit_qt_info(f"[INFO] Drawing figures set aside, batch {index} of {len(batches)}")
        if draw_queue(batch):
            shutil.rmtree(batch, ignore_errors=True)
        else:
            failed.append(batch)

    # And the directory itself, once nothing is left in it.
    try:
        deferred_directory(working_dir).rmdir()
    except OSError:
        pass

    if failed:
        print(
            f"{len(failed)} of {len(batches)} batches of figures could not be drawn; "
            "their specifications are kept:\n" + "\n".join(str(batch) for batch in failed),
            file=sys.stderr,
        )
        return 1
    emit_qt_info("[INFO] Every figure set aside is drawn")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--working_dir", required=True, help="Path to the working directory")
    args = parser.parse_args()
    return render_figures(args.working_dir)


if __name__ == "__main__":
    raise SystemExit(main())
