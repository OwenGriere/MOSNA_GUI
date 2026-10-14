import yaml
import argparse

def get_arguments():
    parser = argparse.ArgumentParser(description = "Get all arguments")
    parser.add_argument('--file', type = str, required=True, help = "config file")
    parser.add_argument("--working_dir", required=True, help="Working directory chosen in the GUI")
    parser.add_argument(
        "--no-figures",
        action="store_true",
        help="write the tables, and set the figures aside instead of drawing them",
    )
    args = parser.parse_args()
    return args.file, args.working_dir


def figures_wanted():
    """False when the step was started with `--no-figures`.

    Apart from `get_arguments`, whose two values every step unpacks: the box is
    read once, where the figures are drawn or set aside.
    """
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--no-figures", action="store_true")
    args, _ = parser.parse_known_args()
    return not args.no_figures

def get_config(config_path):
    with open(config_path, 'r') as f:
        config = yaml.safe_load(f)
    return config