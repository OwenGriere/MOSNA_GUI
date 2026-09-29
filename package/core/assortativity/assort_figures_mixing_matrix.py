"""The mixing matrix of every sample, diagonal included."""

from .mixing_matrix_figure import mixing_matrix_figures


def assort_figures_mixing_matrix(net_stat, save_dir, working_dir, is_sample=None):
    mixing_matrix_figures(
        net_stat,
        save_dir,
        "assort_files",
        is_sample,
        working_dir,
        drop_diagonal=False,
    )
