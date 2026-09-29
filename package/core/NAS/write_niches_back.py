"""Écrire la niche de chaque cellule dans le fichier de nœuds d'où elle vient.

Sans cela, la colonne `niches` n'existe nulle part après une analyse agrégée :
`merge_niche_pheno` est ce qui l'écrivait, et le chemin agrégé ne l'appelle
plus depuis qu'il obtient ses types cellulaires de `mosna.aggregate_cell_types`.
Le redessin des réseaux colorés par niche réclame pourtant cette colonne, et
échouait donc sur un `KeyError: 'niches'`.

# Pourquoi l'ordre est la seule chose difficile ici

Les étiquettes sont dans l'ordre des lignes de `var_aggreg`. Les cellules sont
dans l'ordre des fichiers. `mosna.aggregate_cell_types` fait déjà tenir les
deux ensemble, et d'une seule façon : les paires (patient, échantillon) sont
parcourues dans l'ordre où elles apparaissent dans `var_aggreg`, et chaque
paire apporte ses cellules dans l'ordre du fichier. La même règle est reprise
ici, pour que la niche écrite sur une cellule soit celle que la composition a
comptée pour elle.

Cette hypothèse est vérifiée avant d'écrire quoi que ce soit, parce qu'une
étiquette posée sur la mauvaise cellule ne se voit pas : la figure reste belle
et le résultat est faux.
"""

import numpy as np
import pandas as pd

from package.utils.find_sample import find_sample
from package.utils.find_sample_from_file import find_sample_from_file

#: Le nom de la colonne écrite, celui que le redessin va chercher.
NICHE_COLUMN = "niches"


def write_niches_back(net_dir, var_aggreg_samples_info, cluster_labels,
                      id_level_1, id_level_2, extension="parquet"):
    """Écrit `NICHE_COLUMN` dans chaque fichier de nœuds de `net_dir`.

    Lève `ValueError` si l'appariement décrit ci-dessus ne tient pas, plutôt
    que d'écrire des étiquettes dont rien ne garantirait qu'elles désignent la
    bonne cellule.
    """
    labels = np.asarray(cluster_labels)
    info = var_aggreg_samples_info.reset_index(drop=True)

    if len(info) != len(labels):
        raise ValueError(
            f"{len(labels)} étiquettes pour {len(info)} lignes agrégées : "
            "les niches ne peuvent pas être réécrites sur les cellules"
        )

    # Où chaque paire (patient, échantillon) commence et finit dans var_aggreg.
    blocks = _contiguous_blocks(info, id_level_1, id_level_2)

    # Le fichier de chaque paire. Les identifiants viennent du nom de fichier,
    # donc ce sont des chaînes ; ceux de var_aggreg peuvent être des entiers.
    by_pair = {}
    for path in find_sample(net_dir, extension, id_level_1, id_level_2):
        patient, sample = find_sample_from_file(path, id_level_1, id_level_2)
        by_pair[(str(patient), str(sample))] = path

    for (patient, sample), (start, stop) in blocks.items():
        path = by_pair.get((str(patient), str(sample)))
        if path is None:
            raise ValueError(
                f"aucun fichier de nœuds pour {id_level_1}-{patient} "
                f"{id_level_2}-{sample}"
            )

        nodes = pd.read_parquet(path)
        if len(nodes) != stop - start:
            raise ValueError(
                f"{path.name} contient {len(nodes)} cellules, mais "
                f"{stop - start} lignes agrégées lui sont attribuées"
            )

        nodes[NICHE_COLUMN] = labels[start:stop]
        nodes.to_parquet(path, index=False)

    return len(blocks)


def _contiguous_blocks(info, id_level_1, id_level_2):
    """`{(patient, échantillon): (début, fin)}`, dans l'ordre d'apparition.

    Refuse une paire qui reparaît après avoir été interrompue par une autre :
    l'appariement de `mosna.aggregate_cell_types` suppose des blocs d'un seul
    tenant, et une table qui n'en est pas faite le ferait échouer en silence.
    """
    keys = list(zip(info[id_level_1], info[id_level_2]))

    blocks = {}
    start = 0
    for position in range(1, len(keys) + 1):
        if position == len(keys) or keys[position] != keys[start]:
            pair = keys[start]
            if pair in blocks:
                raise ValueError(
                    f"les lignes de {pair} ne sont pas d'un seul tenant dans "
                    "la table agrégée : l'ordre des étiquettes est inconnu"
                )
            blocks[pair] = (start, position)
            start = position
    return blocks
