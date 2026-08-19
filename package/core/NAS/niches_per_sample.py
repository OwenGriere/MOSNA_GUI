import pandas as pd
from mosna import mosna
from package.utils.emit_qt_progress import emit_qt_info, emit_qt_progress
from package.core.NAS.merge_niche_pheno import merge_niche_pheno
from package.core.NAS.mosna_figures import mosna_figures

def niches_per_sample(method, net_dir, save_dir, data_info ,pheno_col, uniq_phenotype, stat_funcs, stat_names, id_level_1, id_level_2,
                     reducer_type, clusterer_type, n_neighbors, metric, n_clusters, resolution, min_dist, dim_clust,
                     min_cluster_size, k_cluster, normalize, order=1):

    emit_qt_info(f'[INFO] {method}, {net_dir}, {pheno_col}, {data_info}, {uniq_phenotype}, {stat_funcs}, {stat_names}, {id_level_1}, {id_level_2}')
    var_aggreg = mosna.compute_spatial_omic_features_single_network(
            method=method,
            net_dir=net_dir,
            attributes_col=pheno_col,
            data_info=data_info,
            use_attributes=uniq_phenotype,
            make_onehot=True,
            stat_funcs=stat_funcs,
            stat_names=stat_names,
            order=order,
            id_level_1=id_level_1,
            id_level_2=id_level_2,
            verbose=0,
    )

    # Voir aggregated_niches : les colonnes d'identifiants sont des métadonnées
    # de ligne, pas des variables à clusteriser.
    id_cols = [col for col in (id_level_1, id_level_2) if col in var_aggreg.columns]
    features_only = var_aggreg.drop(columns=id_cols)

    cluster_labels, _, _, _ = mosna.get_clusterer(
        data=features_only.values,
        data_dir=save_dir,
        reducer_type=reducer_type,
        clusterer_type=clusterer_type,
        n_neighbors=n_neighbors,
        metric=metric,
        n_clusters=n_clusters,
        resolution=resolution,
        min_dist=min_dist,
        dim_clust=dim_clust,
        min_cluster_size=min_cluster_size,
        k_cluster=k_cluster,
        verbose=0,
    )

    cell_types = merge_niche_pheno(net_dir, pheno_col, cluster_labels)

    if normalize == 'all':
        for normalization in ['total', 'niche', 'obs', 'clr', 'niche&obs']:
            counts = mosna.make_niches_composition(
                    var=cell_types,
                    niches=cluster_labels,
                    var_label=pheno_col,
                    normalize=normalization
            )
            save_dir_norm = save_dir / f'{normalization}'
            save_dir_norm.mkdir(exist_ok=True, parents=True)
            mosna_figures(cluster_labels, counts, save_dir_norm)
    else:
        counts = mosna.make_niches_composition(
                    var=cell_types,
                    niches=cluster_labels,
                    var_label=pheno_col,
                    normalize=normalize
        )
        mosna_figures(cluster_labels, counts, save_dir)