"""One sample's spatial network: built with tysserand, written, and queued.

# What this no longer does

It no longer draws. The network was a matplotlib figure of thirty inches a
side, and the questions it got asked — which cell is that, what phenotype is
this cluster — a PNG cannot answer. The interface now draws the network itself
from the files written here, at any zoom and with the attributes still
attached, and the niche networks are handed to the `xy` renderer as an
interactive chart.

Everything else is unchanged: `tysserand` builds the edges, and the nodes and
edges tables are written exactly where they were.
"""

import gc

import numpy as np
import pandas as pd

from tysserand import tysserand as ty

from ...utils.figure_queue import Spec, queue
from ...utils.find_sample_from_file import find_sample_from_file
from ...utils.read_extension import get_opener


def draw_per_sample(node_file,
                    X_position,
                    Y_position,
                    pheno_col,
                    clusters_cmap,
                    method,
                    min_neighbors,
                    saving_folder,
                    temp_folder,
                    patient_colmun,
                    sample_column,
                    extension,
                    edges_file=None,
                    working_dir=None):

    opener = get_opener(extension)
    node = opener(node_file)

    patient, sample = find_sample_from_file(node_file, patient_colmun, sample_column)
    # L'identifiant est repris tel qu'il apparaît dans le nom de fichier. Le
    # `int()` qui était appliqué ici transformait `nodes_patient-01.csv` en
    # `nodes_patient-1.parquet` : les fichiers écrits dans temp/net_dir_mosna ne
    # portaient plus le même identifiant que les fichiers d'entrée, et un
    # identifiant sans chiffre (`P01a`, un code-barres) levait une ValueError.

    clustering = node[pheno_col]
    coords = node[[X_position, Y_position]].to_numpy()

    if edges_file is None:
        pairs = ty.build_delaunay(coords)
        pairs = ty.link_solitaries(coords, pairs, method=method,
                                   min_neighbors=min_neighbors, verbose=0)
    else:
        pairs = pd.read_parquet(edges_file).values

    edge = pd.DataFrame(data=pairs, columns=['source', 'target'])

    if sample_column is None:
        stem = f"net_{patient}"
        title = f"Tysserand network {patient_colmun} {patient}"
        nodes_name = f"nodes_{patient_colmun}-{patient}.parquet"
        edges_name = f"edges_{patient_colmun}-{patient}.parquet"
    else:
        stem = f"net_{patient}-{sample}"
        title = (f"Tysserand network {patient_colmun} {patient} "
                 f"and {sample_column} {sample}")
        nodes_name = f"nodes_{patient_colmun}-{patient}_{sample_column}-{sample}.parquet"
        edges_name = f"edges_{patient_colmun}-{patient}_{sample_column}-{sample}.parquet"

    if temp_folder is not None and temp_folder != 'None':
        node.to_parquet(temp_folder / nodes_name)
        edge.to_parquet(temp_folder / edges_name)

    # A figure only when one is asked for. Step 1 asks for none: the interface
    # draws that network live from the files just written, which is the same
    # data at any zoom. The niche replot does ask, because its colouring is the
    # result of the analysis and belongs in the gallery beside the rest of it.
    if working_dir is not None and saving_folder is not None and saving_folder != 'None':
        queue(
            _network_spec(stem, saving_folder, coords, pairs, clustering,
                          clusters_cmap, pheno_col, title),
            working_dir,
        )

    del pairs, coords, clustering, clusters_cmap, node, opener, edge
    gc.collect()


def _network_spec(stem, saving_folder, coords, pairs, clustering,
                  clusters_cmap, pheno_col, title):
    """The network as the renderer takes it.

    The phenotype of each cell is handed over as an index into a list of names
    and a list of colours, rather than as a column of strings: the renderer
    draws one series per phenotype, and an index is what tells it which cell
    belongs to which without repeating a name a hundred thousand times.
    """
    labels = pd.Series(clustering).astype(object)
    # By frequency, which is the order `generate_cmap` assigned the colours in,
    # so a phenotype keeps the colour it has in every other figure.
    phenotypes = [p for p in labels.value_counts().index]
    positions = {phenotype: index for index, phenotype in enumerate(phenotypes)}

    index = labels.map(positions).fillna(len(phenotypes)).to_numpy(dtype=np.uint32)
    colours = [str(clusters_cmap.get(phenotype, "#888888")) for phenotype in phenotypes]

    return (
        Spec("network", stem, saving_folder)
        .set_array("coords", np.asarray(coords, dtype=float))
        .set_array("edges", np.asarray(pairs, dtype=np.uint32), dtype="u32")
        .set_array("phenotype_index", index, dtype="u32")
        .set("phenotypes", [str(phenotype) for phenotype in phenotypes])
        .set("colours", colours)
        .set("legend_title", str(pheno_col))
        .set("title", title)
    )
