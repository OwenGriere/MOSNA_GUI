"""The same matrices with the diagonal blanked.

A phenotype's affinity for itself is the largest number in the matrix and
takes the whole colour range with it. Blanking it lets the pairs that are
actually being compared use the scale.
"""

from .mixing_matrix_figure import mixing_matrix_figures


def assort_figures_mixing_matrix_without_diag(net_stat, save_dir, working_dir, is_sample):
    mixing_matrix_figures(
        net_stat,
        save_dir,
        "assort_files_without_diag",
        is_sample,
        working_dir,
        drop_diagonal=True,
    )
