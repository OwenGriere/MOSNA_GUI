import pandas as pd

# Le parseur C par défaut de `read_csv` n'arrondit pas correctement : sur des
# coordonnées écrites avec toutes leurs décimales, il renvoie une valeur à 1 ULP
# de la valeur exacte pour environ 15 % des cellules. `round_trip` utilise le
# strtod de la libc et rend exactement la valeur que `float()` donnerait, ce qui
# rend la lecture reproductible et indépendante du moteur pandas.
FLOAT_PRECISION = "round_trip"


def get_opener(extension):
    if extension == "csv":
        def opener(path):
            return pd.read_csv(path, float_precision=FLOAT_PRECISION)

    elif extension == "parquet":
        def opener(path):
            return pd.read_parquet(path)

    elif extension == "tsv":
        def opener(path):
            return pd.read_csv(path, sep='\t', float_precision=FLOAT_PRECISION)

    return opener
