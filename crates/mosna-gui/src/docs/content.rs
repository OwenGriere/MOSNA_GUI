//! The manual's text, in English and French.
//!
//! Adapted from the Python interface's HTML manual, with an installation chapter the
//! Python version does not have — the Python was always run from a checkout,
//! this is installed.
//!
//! Keeping the text here rather than in a data file means the compiler checks
//! its structure, and the test suite can compare it against what the interface
//! actually offers.

use super::model::{
    Block, CalloutKind, Chapter, Citation as C, ParameterRow as Row, Section, Text as T,
};

/// Assemble every chapter.
pub fn chapters() -> Vec<Chapter> {
    vec![
        installation(),
        getting_started(),
        workflow(),
        parameters(),
        results(),
        credits(),
    ]
}

// ---------------------------------------------------------------------------
// Installation
// ---------------------------------------------------------------------------

fn installation() -> Chapter {
    Chapter {
        id: "installation",
        title: T::new("Installation", "Installation"),
        sections: vec![
            Section {
                id: "install-requirements",
                title: T::new("Requirements", "Prérequis"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "MOSNA GUI is two halves. The interface is a Rust program — this \
                         window. The analyses are the Python ones: Tysserand builds the \
                         networks, MOSNA computes the assortativity and the niches, and the \
                         interface starts them as sub-processes, so they can still be run \
                         from a terminal without it.",
                        "MOSNA GUI est fait de deux moitiés. L'interface est un programme Rust \
                         — cette fenêtre. Les analyses sont les analyses Python : Tysserand \
                         construit les réseaux, MOSNA calcule l'assortativité et les niches, \
                         et l'interface les lance comme sous-processus — elles restent donc \
                         lançables depuis un terminal sans elle.",
                    )),
                    Block::Paragraph(T::new(
                        "So the machine needs both: conda, to build the environment the \
                         analyses run in, and the Rust toolchain, to build the interface. \
                         setup.sh does both in one pass.",
                        "La machine a donc besoin des deux : conda, pour construire \
                         l'environnement dans lequel tournent les analyses, et la chaîne \
                         d'outils Rust, pour construire l'interface. setup.sh fait les deux en \
                         une seule passe.",
                    )),
                    Block::Code {
                        caption: T::new(
                            "Install the Rust toolchain, once",
                            "Installer la chaîne d'outils Rust, une seule fois",
                        ),
                        lines: vec![
                            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
                        ],
                    },
                    Block::Callout {
                        kind: CalloutKind::Note,
                        text: T::new(
                            "Open a new terminal after installing Rust, so that the new \
                             commands are on your PATH.",
                            "Ouvrez un nouveau terminal après avoir installé Rust, pour que \
                             les nouvelles commandes soient dans votre PATH.",
                        ),
                    },
                    Block::Paragraph(T::new(
                        "Conda comes from Miniconda or Anaconda. Accept its channel terms \
                         before running setup.sh: an installer that stops half way to ask a \
                         licensing question leaves the environment incomplete.",
                        "Conda provient de Miniconda ou d'Anaconda. Acceptez ses conditions de \
                         canaux avant de lancer setup.sh : un installeur qui s'arrête à \
                         mi-chemin pour poser une question de licence laisse l'environnement \
                         incomplet.",
                    )),
                    Block::Callout {
                        kind: CalloutKind::Note,
                        text: T::new(
                            "The environment is Python 3.11. That is not a preference: the \
                             figures are drawn by xy, which requires 3.11 or newer, and the \
                             analyses and the renderer share one interpreter.",
                            "L'environnement est en Python 3.11. Ce n'est pas une préférence : \
                             les figures sont dessinées par xy, qui exige 3.11 ou plus récent, \
                             et les analyses et le moteur de rendu partagent un même \
                             interpréteur.",
                        ),
                    },
                ],
            },
            Section {
                id: "install-linux",
                title: T::new(
                    "Installing on Linux and macOS",
                    "Installer sous Linux et macOS",
                ),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "From the MOSNA_GUI directory, one command does everything. The first \
                         run takes several minutes: it resolves a scientific Python \
                         environment and compiles the interface.",
                        "Depuis le dossier MOSNA_GUI, une seule commande fait tout. Le premier \
                         passage prend plusieurs minutes : il résout un environnement Python \
                         scientifique et compile l'interface.",
                    )),
                    Block::Code {
                        caption: T::new("Install", "Installer"),
                        lines: vec!["cd MOSNA_GUI", "bash setup.sh"],
                    },
                    Block::List(vec![
                        T::new(
                            "a conda environment named mosna-GUI, holding scanpy, tysserand, \
                             the mosna package and the xy renderer",
                            "un environnement conda nommé mosna-GUI, contenant scanpy, \
                             tysserand, le paquet mosna et le moteur de rendu xy",
                        ),
                        T::new(
                            "target/release/mosna-gui — this interface",
                            "target/release/mosna-gui — cette interface",
                        ),
                        T::new(
                            "MosnaGUI.sh and a launcher on your desktop",
                            "MosnaGUI.sh et un lanceur sur votre bureau",
                        ),
                    ]),
                    Block::Code {
                        caption: T::new("Other options", "Autres options"),
                        lines: vec![
                            "bash setup.sh --no-shortcut    # skip the desktop launcher",
                            "bash setup.sh --no-rust        # environment only, no rebuild",
                        ],
                    },
                    Block::Callout {
                        kind: CalloutKind::Tip,
                        text: T::new(
                            "If your desktop asks whether to trust the launcher, allow it \
                             once. setup.sh already marks the file executable, which is what \
                             most desktops need.",
                            "Si votre bureau demande s'il faut faire confiance au lanceur, \
                             autorisez-le une fois. setup.sh marque déjà le fichier comme \
                             exécutable, ce qu'attendent la plupart des bureaux.",
                        ),
                    },
                ],
            },
            Section {
                id: "install-windows",
                title: T::new("Installing on Windows", "Installer sous Windows"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "Double-click INSTALLATION.exe in the MOSNA GUI folder. A window asks \
                         where to put the folder and whether to add a desktop shortcut, then \
                         installs whatever is missing — the Microsoft C++ build tools, Rust, \
                         Miniconda — builds the same mosna-GUI conda environment as setup.sh, \
                         compiles the interface and creates its shortcuts, in the Start Menu \
                         and, if asked for, on the desktop.",
                        "Double-cliquez sur INSTALLATION.exe dans le dossier de MOSNA GUI. Une \
                         fenêtre demande où placer le dossier et s'il faut un raccourci sur le \
                         bureau, puis installe ce qui manque — outils C++ de Microsoft, Rust, \
                         Miniconda —, construit le même environnement conda mosna-GUI que \
                         setup.sh, compile l'interface et crée ses raccourcis, dans le menu \
                         Démarrer et, si vous l'avez demandé, sur le bureau.",
                    )),
                    Block::Code {
                        caption: T::new("Install, then remove", "Installer, puis désinstaller"),
                        lines: vec!["INSTALLATION.exe", "UNINSTALL.exe"],
                    },
                    Block::Paragraph(T::new(
                        "UNINSTALL.exe, in the same folder, removes the shortcuts, the \
                         launcher, the mosna-GUI environment and the build and, if you tick \
                         them, the tools the installation added and the folder itself. Its \
                         log, like the installer's, is kept in %TEMP%.",
                        "UNINSTALL.exe, dans le même dossier, supprime les raccourcis, le \
                         lanceur, l'environnement mosna-GUI et la compilation et, si vous les \
                         cochez, les outils que l'installation a ajoutés et le dossier \
                         lui-même. Son journal, comme celui de l'installation, est gardé dans \
                         %TEMP%.",
                    )),
                ],
            },
            Section {
                id: "install-after",
                title: T::new("After installing", "Après l'installation"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "Start MOSNA GUI from the desktop icon, or from a terminal through \
                         the launcher setup.sh wrote — it activates the conda environment \
                         first, which is what lets the interface find the interpreter the \
                         analyses need.",
                        "Démarrez MOSNA GUI depuis l'icône du bureau, ou depuis un terminal \
                         via le lanceur écrit par setup.sh — il active d'abord l'environnement \
                         conda, ce qui permet à l'interface de trouver l'interpréteur dont ont \
                         besoin les analyses.",
                    )),
                    Block::Code {
                        caption: T::new("From a terminal", "Depuis un terminal"),
                        lines: vec!["bash MosnaGUI.sh"],
                    },
                    Block::Paragraph(T::new(
                        "Removing it is removing the environment and the build. Your \
                         configuration and your results are outside both, and are left alone.",
                        "Le désinstaller, c'est supprimer l'environnement et la construction. \
                         Votre configuration et vos résultats sont en dehors des deux, et ne \
                         sont pas touchés.",
                    )),
                    Block::Code {
                        caption: T::new("Remove it again", "Le désinstaller"),
                        lines: vec![
                            "conda env remove -n mosna-GUI",
                            "cargo clean",
                            "rm -f MosnaGUI.sh ~/Desktop/MosnaGUI.desktop",
                        ],
                    },
                    Block::Callout {
                        kind: CalloutKind::Note,
                        text: T::new(
                            "Two environment variables override what the interface finds on \
                             its own: MOSNA_PYTHON names the interpreter to run the analyses \
                             with, and MOSNA_GUI_ROOT the directory holding the package/ \
                             folder. Neither is needed when the launcher is used.",
                            "Deux variables d'environnement l'emportent sur ce que l'interface \
                             trouve seule : MOSNA_PYTHON désigne l'interpréteur avec lequel \
                             lancer les analyses, et MOSNA_GUI_ROOT le dossier contenant le \
                             dossier package/. Aucune n'est nécessaire si l'on passe par le \
                             lanceur.",
                        ),
                    },
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Getting started
// ---------------------------------------------------------------------------

fn getting_started() -> Chapter {
    Chapter {
        id: "getting-started",
        title: T::new("Getting started", "Premiers pas"),
        sections: vec![
            Section {
                id: "purpose",
                title: T::new("What this interface is for", "À quoi sert cette interface"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "This interface analyses spatial omics data using networks. It drives \
                         Tysserand and MOSNA step by step, so you can run a whole spatial \
                         analysis without writing code.",
                        "Cette interface analyse des données omiques spatiales à l'aide de \
                         réseaux. Elle pilote Tysserand et MOSNA étape par étape, ce qui permet \
                         de mener une analyse spatiale complète sans écrire de code.",
                    )),
                    Block::List(vec![
                        T::new(
                            "Step 1 — Tysserand reconstructs a spatial network per sample.",
                            "Étape 1 — Tysserand reconstruit un réseau spatial par échantillon.",
                        ),
                        T::new(
                            "Step 2 — Assortativity measures which cell types sit next to which.",
                            "Étape 2 — L'assortativité mesure quels types cellulaires voisinent \
                             avec quels autres.",
                        ),
                        T::new(
                            "Step 3 — Niche Analysis groups neighbourhoods into spatial niches.",
                            "Étape 3 — L'analyse de niches regroupe les voisinages en niches \
                             spatiales.",
                        ),
                    ]),
                ],
            },
            Section {
                id: "working-directory",
                title: T::new(
                    "Start-up and working directory",
                    "Démarrage et répertoire de travail",
                ),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "On launch you must choose a folder. Without a working directory the \
                         interface stays disabled.",
                        "Au démarrage, vous devez choisir un dossier. Sans répertoire de \
                         travail, l'interface reste désactivée.",
                    )),
                    Block::Paragraph(T::new(
                        "That folder is where everything is written: the results, and the \
                         intermediate network files under temp.",
                        "C'est dans ce dossier que tout est écrit : les résultats, et les \
                         fichiers réseau intermédiaires sous temp.",
                    )),
                    Block::Callout {
                        kind: CalloutKind::Tip,
                        text: T::new(
                            "Create a new folder at the moment you choose the working \
                             directory. That folder becomes your analysis.",
                            "Créez un nouveau dossier au moment de choisir le répertoire de \
                             travail. Ce dossier devient votre analyse.",
                        ),
                    },
                ],
            },
            Section {
                id: "input-files",
                title: T::new("Input files", "Fichiers d'entrée"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "You need at least one CSV or Parquet file listing cells with their \
                         coordinates and attributes. Tysserand turns those into nodes and \
                         edges.",
                        "Il vous faut au moins un fichier CSV ou Parquet listant les cellules \
                         avec leurs coordonnées et leurs attributs. Tysserand en tire des \
                         noeuds et des arêtes.",
                    )),
                    Block::Paragraph(T::new(
                        "If you have no coordinates but already have edges, you can skip \
                         straight to assortativity and niche analysis, provided the files \
                         follow the naming below.",
                        "Si vous n'avez pas de coordonnées mais déjà des arêtes, vous pouvez \
                         passer directement à l'assortativité et à l'analyse de niches, à \
                         condition que les fichiers respectent le nommage ci-dessous.",
                    )),
                    Block::Code {
                        caption: T::new("File naming", "Nommage des fichiers"),
                        lines: vec![
                            "# For Tysserand (step 1):",
                            "nodes_{patient-name}-{patient-id}_{sample-name}-{sample-id}.parquet",
                            "",
                            "# To start directly at step 2 or 3, two files per sample:",
                            "nodes_{patient-name}-{patient-id}_{sample-name}-{sample-id}.parquet",
                            "edges_{patient-name}-{patient-id}_{sample-name}-{sample-id}.parquet",
                        ],
                    },
                    Block::Paragraph(T::new(
                        "The second level is optional: for a dataset with patients only, \
                         leave the sample column name empty and name the files \
                         nodes_patient-01.parquet.",
                        "Le second niveau est facultatif : pour un jeu de données par patient \
                         seulement, laissez le nom de colonne d'échantillon vide et nommez les \
                         fichiers nodes_patient-01.parquet.",
                    )),
                    Block::Callout {
                        kind: CalloutKind::Warning,
                        text: T::new(
                            "If the patient or sample column name does not match what the file \
                             names actually contain, the table stays empty. That is the most \
                             common reason nothing appears.",
                            "Si le nom de colonne patient ou échantillon ne correspond pas à ce \
                             que contiennent réellement les noms de fichiers, la table reste \
                             vide. C'est la raison la plus fréquente pour laquelle rien \
                             n'apparaît.",
                        ),
                    },
                    Block::Paragraph(T::new(
                        "Press 'Refresh Nodes' to list your raw files, and 'Refresh Networks' \
                         to list reconstructed networks.",
                        "Appuyez sur « Refresh Nodes » pour lister vos fichiers bruts, et sur \
                         « Refresh Networks » pour lister les réseaux reconstruits.",
                    )),
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// How it works
// ---------------------------------------------------------------------------

fn workflow() -> Chapter {
    Chapter {
        id: "workflow",
        title: T::new("How it works", "Fonctionnement"),
        sections: vec![
            Section {
                id: "panels",
                title: T::new("The three panels", "Les trois panneaux"),
                blocks: vec![
                    Block::List(vec![
                        T::new(
                            "Browser, on the left — where your data is and how the files are \
                             named. Nothing is computed here; it prepares the paths.",
                            "Browser, à gauche — où sont vos données et comment les fichiers \
                             sont nommés. Rien n'y est calculé ; il prépare les chemins.",
                        ),
                        T::new(
                            "Viewer, in the middle — figures, the network drawn from the files \
                             themselves, the log of a running analysis, and this manual.",
                            "Viewer, au centre — les figures, le réseau tracé directement \
                             depuis les fichiers, le journal de l'analyse en cours, et ce \
                             manuel.",
                        ),
                        T::new(
                            "Parameters, on the right — every setting, plus the buttons that \
                             run the three steps.",
                            "Parameters, à droite — tous les réglages, ainsi que les boutons qui \
                             lancent les trois étapes.",
                        ),
                    ]),
                    Block::Paragraph(T::new(
                        "Selecting a file in the Browser reads its column names, so the \
                         parameter drop-downs offer real columns instead of free text.",
                        "Sélectionner un fichier dans le Browser lit ses noms de colonnes : les \
                         listes déroulantes des paramètres proposent alors de vraies colonnes \
                         plutôt qu'un texte libre.",
                    )),
                    Block::Image {
                        asset: "images/GUI.png",
                        caption: T::new(
                            "The three panels, at start-up: until a working directory is \
                             chosen, that is all the interface will do",
                            "Les trois panneaux, au démarrage : tant qu'un répertoire de \
                             travail n'est pas choisi, l'interface ne fait rien d'autre",
                        ),
                    },
                    Block::Paragraph(T::new(
                        "Click the Browser's or the Parameters' title to fold that panel down \
                         to a band carrying its name; click the band to bring it back, at the \
                         width it had. The Viewer between them takes whatever they leave, \
                         which is how the Network tab gets a screen's width to draw in. The \
                         Viewer itself does not fold.",
                        "Cliquez sur le titre du Browser ou des Parameters pour replier ce \
                         panneau en une bande portant son nom ; cliquez sur la bande pour le \
                         rouvrir, à la largeur qu'il avait. Le Viewer entre les deux prend \
                         tout ce qu'ils laissent, ce qui donne à l'onglet Network la largeur \
                         d'un écran pour tracer. Le Viewer, lui, ne se replie pas.",
                    )),
                    Block::Paragraph(T::new(
                        "The Viewer's Network tab draws a sample from its nodes and edges \
                         files rather than from a figure. Choose the patient, then the sample; \
                         a dataset with no sample column asks only for the patient. Drag to \
                         pan, scroll or use the zoom buttons, and hover a cell to read the \
                         columns you ticked in the margin. Colouring by a column of labels — a \
                         phenotype, a niche — gives a legend of its values; colouring by a \
                         measured column gives a colour bar over its range, in a ramp you \
                         choose. Tick up to four columns and each cell is drawn in a blend of \
                         their ramps, weighted by how strongly that cell expresses each one — \
                         so a cell high in the red column and the blue one is neither red nor \
                         blue. A blend cannot be read back: the bars in the margin describe \
                         each column on its own, and the tooltip carries the values. A cell \
                         with no value in any of the chosen columns stays grey. Past sixty \
                         thousand cells in view only a fraction are drawn, with the edges \
                         between them: at that size a cell is under a pixel and they overlap \
                         several deep, so the rest would cost frames and show nothing. Zoom \
                         in and they all come back.",
                        "L'onglet Network du Viewer trace un échantillon depuis ses fichiers de \
                         nœuds et d'arêtes plutôt que depuis une figure. Choisissez le patient, \
                         puis l'échantillon ; un jeu de données sans colonne d'échantillon ne \
                         demande que le patient. Glisser pour se déplacer, molette ou boutons \
                         de zoom, survoler une cellule pour lire les colonnes cochées en \
                         marge. Colorer par une colonne d'étiquettes — un phénotype, une niche \
                         — donne une légende de ses valeurs ; colorer par une colonne mesurée \
                         donne une barre de couleur sur son étendue, dans une palette que \
                         vous choisissez. Cochez jusqu'à quatre colonnes et chaque cellule est \
                         tracée dans un mélange de leurs palettes, pondéré par la force avec \
                         laquelle elle exprime chacune — une cellule forte dans la colonne \
                         rouge et dans la bleue n'est ni rouge ni bleue. Un mélange ne se \
                         relit pas : les barres en marge décrivent chaque colonne seule, et \
                         l'infobulle porte les valeurs. Une cellule sans valeur dans aucune \
                         des colonnes choisies reste grise. Au-delà de soixante mille \
                         cellules à l'écran, seule une fraction est tracée, avec les arêtes qui \
                         les relient : à cette taille une cellule fait moins d'un pixel et \
                         elles se recouvrent, le reste coûterait des images par seconde sans \
                         rien montrer. En zoomant, tout revient.",
                    )),
                ],
            },
            Section {
                id: "architecture",
                title: T::new("How the analysis is organised", "Organisation de l'analyse"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "Your working directory is also your saving directory. Each step \
                         writes into it, and the next step reads what the previous one wrote.",
                        "Votre répertoire de travail est aussi votre répertoire de sauvegarde. \
                         Chaque étape y écrit, et l'étape suivante lit ce que la précédente a \
                         écrit.",
                    )),
                    Block::Code {
                        caption: T::new(
                            "What lands in the working directory",
                            "Ce qui atterrit dans le répertoire de travail",
                        ),
                        lines: vec![
                            "temp/net_dir_mosna/      nodes_*.parquet, edges_*.parquet",
                            "Assortativity/           net_stat.csv and its figures",
                            "  assort_files/          one mixing matrix per sample",
                            "  assort_files_without_diag/   the same, diagonal blanked",
                            "Niche_Analysis/",
                            "  Aggregation/{saving directory}/   niches over the whole cohort",
                            "  Per_sample/{saving directory}/    niches sample by sample",
                        ],
                    },
                    Block::Paragraph(T::new(
                        "Step 1 writes no figure. Its only one was a picture of the network, \
                         and the Viewer's Network tab draws the network itself — from the \
                         same files, at any zoom, with every attribute still readable at the \
                         pointer. The niche networks of step 3 are figures, because their \
                         colouring is a result of the analysis and belongs in the gallery \
                         beside the rest of it.",
                        "L'étape 1 n'écrit aucune figure. Sa seule image était une vue du \
                         réseau, et l'onglet Network du Viewer trace le réseau lui-même — \
                         depuis les mêmes fichiers, à n'importe quel zoom, chaque attribut \
                         restant lisible sous le pointeur. Les réseaux de niches de l'étape 3 \
                         sont bien des figures : leur coloration est un résultat de l'analyse \
                         et a sa place dans la galerie, à côté du reste.",
                    )),
                    Block::Paragraph(T::new(
                        "Under the three step buttons sits one more. It computes nothing: it \
                         is what you do with a directory the steps have already filled.",
                        "Sous les trois boutons d'étape s'en trouve un autre. Il ne calcule \
                         rien : il sert à traiter un répertoire que les étapes ont déjà \
                         rempli.",
                    )),
                    Block::List(vec![T::new(
                        "Clear temporary data — deletes the temp folder and the \
                             intermediate networks in it. The figures and the tables are \
                             kept.",
                        "Clear temporary data — supprime le dossier temp et les réseaux \
                             intermédiaires qu'il contient. Les figures et les tableaux sont \
                             conservés.",
                    )]),
                    Block::Callout {
                        kind: CalloutKind::Tip,
                        text: T::new(
                            "Steps 2 and 3 read the temporary networks, and so does the \
                             Network tab — clear them only when you are done looking.",
                            "Les étapes 2 et 3 lisent les réseaux temporaires, et l'onglet \
                             Network aussi — ne les videz qu'une fois que vous avez fini de \
                             regarder.",
                        ),
                    },
                    Block::Image {
                        asset: "images/workflow.png",
                        caption: T::new("The analysis workflow", "Le déroulé de l'analyse"),
                    },
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

fn parameters() -> Chapter {
    let headers = [
        T::new("Parameter", "Paramètre"),
        T::new("Type", "Type"),
        T::new("Description", "Description"),
    ];

    Chapter {
        id: "parameters",
        title: T::new("Parameters", "Paramètres"),
        sections: vec![
            Section {
                id: "parameters-global",
                title: T::new("Shared parameters", "Paramètres communs"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "These five live in the Browser panel and are shared by all three \
                         steps, so the steps always agree on how your files are named.",
                        "Ces cinq paramètres se trouvent dans le panneau Browser et sont \
                         partagés par les trois étapes, afin qu'elles s'accordent toujours sur \
                         le nommage de vos fichiers.",
                    )),
                    Block::Table {
                        headers,
                        rows: vec![
                            Row::new(
                                "Nodes directory",
                                "string",
                                T::new(
                                    "Folder holding your spatial data.",
                                    "Dossier contenant vos données spatiales.",
                                ),
                            ),
                            Row::new(
                                "Network directory",
                                "string",
                                T::new(
                                    "Folder holding your nodes and edges. Leave on Default to use \
                                 what step 1 produced.",
                                    "Dossier contenant vos noeuds et arêtes. Laissez sur Default \
                                 pour utiliser ce qu'a produit l'étape 1.",
                                ),
                            ),
                            Row::new(
                                "Patient column name",
                                "string",
                                T::new(
                                    "Name of the first level of division, for example 'patient'.",
                                    "Nom du premier niveau de découpage, par exemple « patient ».",
                                ),
                            ),
                            Row::new(
                                "Sample column name",
                                "string",
                                T::new(
                                    "Name of the second level, if there is one. Leave empty \
                                 otherwise.",
                                    "Nom du second niveau, s'il existe. Laissez vide sinon.",
                                ),
                            ),
                            Row::new(
                                "Extension",
                                "string",
                                T::new(
                                    "File format of your input: csv, tsv or parquet.",
                                    "Format de vos fichiers d'entrée : csv, tsv ou parquet.",
                                ),
                            ),
                        ],
                    },
                ],
            },
            Section {
                id: "parameters-tysserand",
                title: T::new("Step 1 — Tysserand", "Étape 1 — Tysserand"),
                blocks: vec![Block::Table {
                    headers,
                    rows: vec![
                        Row::new(
                            "X coordinates column",
                            "string",
                            T::new(
                                "Column holding the X spatial coordinate.",
                                "Colonne contenant la coordonnée spatiale X.",
                            ),
                        ),
                        Row::new(
                            "Y coordinates column",
                            "string",
                            T::new(
                                "Column holding the Y spatial coordinate.",
                                "Colonne contenant la coordonnée spatiale Y.",
                            ),
                        ),
                        Row::new(
                            "Phenotype column",
                            "string",
                            T::new(
                                "Column giving the phenotype of each cell.",
                                "Colonne donnant le phénotype de chaque cellule.",
                            ),
                        ),
                        Row::new(
                            "Edges method",
                            "string",
                            T::new(
                                "How edges are drawn: delaunay triangulates, knn joins each cell \
                             to its nearest neighbours.",
                                "Comment les arêtes sont tracées : delaunay triangule, knn relie \
                             chaque cellule à ses plus proches voisines.",
                            ),
                        ),
                        Row::new(
                            "Min neighbors",
                            "int",
                            T::new(
                                "Minimum number of neighbours each cell must keep. Cells left \
                             below it are reconnected.",
                                "Nombre minimal de voisins que chaque cellule doit conserver. Les \
                             cellules en dessous sont reconnectées.",
                            ),
                        ),
                        Row::new(
                            "CPU",
                            "int",
                            T::new(
                                "How many cores to use. Capped by the machine and by the number \
                             of samples.",
                                "Nombre de coeurs à utiliser. Plafonné par la machine et par le \
                             nombre d'échantillons.",
                            ),
                        ),
                    ],
                }],
            },
            Section {
                id: "parameters-assortativity",
                title: T::new("Step 2 — Assortativity", "Étape 2 — Assortativité"),
                blocks: vec![Block::Table {
                    headers,
                    rows: vec![
                        Row::new(
                            "Phenotype column",
                            "string",
                            T::new(
                                "Column giving the phenotype of each cell.",
                                "Colonne donnant le phénotype de chaque cellule.",
                            ),
                        ),
                        Row::new(
                            "Index",
                            "string",
                            T::new(
                                "Column identifying each cell. Leave on 'index' to use row order.",
                                "Colonne identifiant chaque cellule. Laissez sur « index » pour \
                             utiliser l'ordre des lignes.",
                            ),
                        ),
                        Row::new(
                            "Number of shuffle",
                            "int",
                            T::new(
                                "How many randomisations build the null distribution. More is \
                             more precise and slower.",
                                "Nombre de permutations construisant la distribution nulle. \
                             Davantage est plus précis et plus lent.",
                            ),
                        ),
                        Row::new(
                            "Randomization diagnostic",
                            "bool",
                            T::new(
                                "Run a short timing probe instead of the full analysis, to \
                             estimate how long the real run will take.",
                                "Lance une brève mesure de temps au lieu de l'analyse complète, \
                             pour estimer la durée du vrai calcul.",
                            ),
                        ),
                    ],
                }],
            },
            Section {
                id: "parameters-niches",
                title: T::new("Step 3 — Niche Analysis", "Étape 3 — Analyse de niches"),
                blocks: vec![
                    Block::Table {
                        headers,
                        rows: vec![
                            Row::new("Saving directory", "string", T::new(
                                "Name of the folder this run writes into, under \
                                 Niche_Analysis/Aggregation or Niche_Analysis/Per_sample. A \
                                 name, not a path — letters, digits, spaces, hyphens and \
                                 underscores. Two runs given different names sit side by side \
                                 and can be compared; two given the same one overwrite each \
                                 other.",
                                "Nom du dossier dans lequel ce calcul écrit, sous \
                                 Niche_Analysis/Aggregation ou Niche_Analysis/Per_sample. Un \
                                 nom, pas un chemin — lettres, chiffres, espaces, tirets et \
                                 tirets bas. Deux calculs aux noms différents cohabitent et se \
                                 comparent ; deux calculs au même nom s'écrasent.")),
                            Row::new("Phenotype column", "string", T::new(
                                "Column giving the phenotype of each cell, used to describe \
                                 what each niche is made of.",
                                "Colonne donnant le phénotype de chaque cellule, utilisée pour \
                                 décrire la composition de chaque niche.")),
                            Row::new("Column to aggregate", "string or list", T::new(
                                "Column or columns aggregated over each neighbourhood. One \
                                 categorical column is one-hot encoded; several numeric \
                                 columns are used as they are.",
                                "Colonne ou colonnes agrégées sur chaque voisinage. Une seule \
                                 colonne catégorielle est encodée en indicatrices ; plusieurs \
                                 colonnes numériques sont utilisées telles quelles.")),
                            Row::new("Processing method", "string", T::new(
                                "Whether niches are called once over the pooled cohort, or \
                                 independently per sample. Only the pooled analysis is \
                                 available: the per-sample path is shown in the menu, greyed \
                                 out, because it has not been verified against real data.",
                                "Si les niches sont déterminées une fois sur la cohorte \
                                 entière, ou indépendamment par échantillon. Seule l'analyse \
                                 groupée est disponible : le chemin par échantillon figure \
                                 dans le menu, grisé, faute d'avoir été éprouvé sur de vraies \
                                 données.")),
                            Row::new("Niches method", "string", T::new(
                                "How neighbourhood features are computed. NAS aggregates the \
                                 attributes of each cell's neighbours.",
                                "Comment les caractéristiques de voisinage sont calculées. NAS \
                                 agrège les attributs des voisins de chaque cellule.")),
                            Row::new("Plot Network", "bool", T::new(
                                "Redraw each network coloured by niche once the niches are \
                                 found.",
                                "Redessine chaque réseau coloré par niche une fois les niches \
                                 trouvées.")),
                            Row::new("X coordinates column for niches", "string", T::new(
                                "X column used for that redraw.",
                                "Colonne X utilisée pour ce redessin.")),
                            Row::new("Y coordinates column for niches", "string", T::new(
                                "Y column used for that redraw.",
                                "Colonne Y utilisée pour ce redessin.")),
                            Row::new("CPU", "int", T::new(
                                "How many cores to use.",
                                "Nombre de coeurs à utiliser.")),
                        ],
                    },
                    Block::Heading(T::new(
                        "Reduction and clustering",
                        "Réduction et regroupement",
                    )),
                    Block::Paragraph(T::new(
                        "These settings appear twice, once for the pooled analysis and once \
                         for the per-sample one. Parameters an algorithm does not use are \
                         greyed out.",
                        "Ces réglages apparaissent deux fois, une pour l'analyse groupée et une \
                         pour celle par échantillon. Les paramètres qu'un algorithme n'utilise \
                         pas sont grisés.",
                    )),
                    Block::Table {
                        headers,
                        rows: vec![
                            Row::new("order", "string", T::new(
                                "Neighbourhood order. 1 uses direct neighbours only; higher \
                                 values reach further through the graph.",
                                "Ordre de voisinage. 1 n'utilise que les voisins directs ; des \
                                 valeurs plus élevées portent plus loin dans le graphe.")),
                            Row::new("stat_funcs", "list", T::new(
                                "Statistics applied to the aggregated neighbour features.",
                                "Statistiques appliquées aux caractéristiques agrégées des \
                                 voisins.")),
                            Row::new("stat_names", "list", T::new(
                                "Names given to those statistics in the output columns.",
                                "Noms donnés à ces statistiques dans les colonnes produites.")),
                            Row::new("reducer_type", "string", T::new(
                                "Dimensionality reduction applied before clustering: umap, or \
                                 none to cluster the aggregated features directly. With none, \
                                 metric, min_dist and dim_clust are ignored and no cluster \
                                 projection is drawn.",
                                "Réduction de dimension appliquée avant le regroupement : umap, \
                                 ou none pour regrouper directement les caractéristiques \
                                 agrégées. Avec none, metric, min_dist et dim_clust sont \
                                 ignorés et aucune projection des groupes n'est tracée.")),
                            Row::new("metric", "string", T::new(
                                "Distance used to compare neighbourhoods: euclidean, \
                                 manhattan or cosine.",
                                "Distance utilisée pour comparer les voisinages : euclidean, \
                                 manhattan ou cosine.")),
                            Row::new("n_neighbors", "int", T::new(
                                "Size of the local neighbourhood UMAP builds. Larger values \
                                 favour global structure.",
                                "Taille du voisinage local que construit UMAP. Des valeurs \
                                 élevées favorisent la structure globale.")),
                            Row::new("min_dist", "float", T::new(
                                "How tightly points may pack in the projection. Smaller gives \
                                 tighter groups.",
                                "À quel point les points peuvent se tasser dans la projection. \
                                 Plus petit donne des groupes plus serrés.")),
                            Row::new("dim_clust", "int", T::new(
                                "Number of dimensions kept after reduction, for clustering.",
                                "Nombre de dimensions conservées après réduction, pour le \
                                 regroupement.")),
                            Row::new("clusterer_type", "string", T::new(
                                "Which algorithm calls the niches: gmm, leiden, ecg, \
                                 spectral or hdbscan. The parameters below that it does not \
                                 read are greyed out as soon as it is chosen.",
                                "Quel algorithme détermine les niches : gmm, leiden, ecg, \
                                 spectral ou hdbscan. Les paramètres ci-dessous qu'il ne lit \
                                 pas sont grisés dès qu'il est choisi.")),
                            Row::new("n_clusters", "int", T::new(
                                "Number of niches to produce. Used by gmm and spectral.",
                                "Nombre de niches à produire. Utilisé par gmm et spectral.")),
                            Row::new("resolution", "float", T::new(
                                "Granularity of Leiden. Lower gives fewer niches, higher gives \
                                 more.",
                                "Granularité de Leiden. Plus bas donne moins de niches, plus \
                                 haut en donne davantage.")),
                            Row::new("k_cluster", "int", T::new(
                                "Neighbours used to build the graph leiden, ecg and spectral \
                                 partition.",
                                "Voisins utilisés pour construire le graphe que partitionnent \
                                 leiden, ecg et spectral.")),
                            Row::new("min_cluster_size", "int", T::new(
                                "Smallest niche HDBSCAN will report.",
                                "Plus petite niche que HDBSCAN acceptera de signaler.")),
                            Row::new("normalize", "string", T::new(
                                "How the niche composition is rescaled before it is plotted. \
                                 'all' produces one figure per variant.",
                                "Comment la composition des niches est remise à l'échelle avant \
                                 tracé. « all » produit une figure par variante.")),
                        ],
                    },
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------

fn results() -> Chapter {
    Chapter {
        id: "results",
        title: T::new("Reading the results", "Lire les résultats"),
        sections: vec![
            Section {
                id: "results-assortativity",
                title: T::new("Assortativity", "Assortativité"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "Assortativity asks whether cells of a given type sit next to their \
                         own kind more often than chance would give. The answer is compared \
                         against a null obtained by shuffling the phenotypes while keeping the \
                         network fixed.",
                        "L'assortativité demande si les cellules d'un type donné voisinent avec \
                         leurs semblables plus souvent que le hasard ne le voudrait. La réponse \
                         est comparée à un modèle nul obtenu en permutant les phénotypes tout \
                         en gardant le réseau fixe.",
                    )),
                    Block::List(vec![
                        T::new(
                            "A positive z-score means the pair is adjacent more often than \
                             chance: the two types cluster together in the tissue.",
                            "Un z-score positif signifie que la paire est adjacente plus \
                             souvent que le hasard : les deux types se regroupent dans le \
                             tissu.",
                        ),
                        T::new(
                            "A negative one means they avoid each other.",
                            "Un z-score négatif signifie qu'ils s'évitent.",
                        ),
                        T::new(
                            "Around zero means the arrangement is indistinguishable from \
                             chance.",
                            "Autour de zéro, l'agencement est indiscernable du hasard.",
                        ),
                    ]),
                    Block::Callout {
                        kind: CalloutKind::Note,
                        text: T::new(
                            "Grey cells in a heatmap are pairs that never occur together in \
                             that sample, so no score can be computed. They are not zeros.",
                            "Les cases grises d'une carte de chaleur sont des paires qui ne \
                             coexistent jamais dans cet échantillon : aucun score ne peut être \
                             calculé. Ce ne sont pas des zéros.",
                        ),
                    },
                    Block::Paragraph(T::new(
                        "The table itself is written to Assortativity/net_stat.csv, one row \
                         per sample, so you can take it into any other tool.",
                        "La table elle-même est écrite dans Assortativity/net_stat.csv, une \
                         ligne par échantillon, pour être reprise dans n'importe quel autre \
                         outil.",
                    )),
                ],
            },
            Section {
                id: "results-niches",
                title: T::new("Niches", "Niches"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "A niche is a recurring kind of neighbourhood. Each cell is described \
                         by what surrounds it, those descriptions are projected into a few \
                         dimensions, and the projection is clustered.",
                        "Une niche est un type de voisinage récurrent. Chaque cellule est \
                         décrite par ce qui l'entoure, ces descriptions sont projetées en \
                         quelques dimensions, et la projection est regroupée.",
                    )),
                    Block::List(vec![
                        T::new(
                            "The composition heatmap says what each niche is made of.",
                            "La carte de composition indique de quoi chaque niche est faite.",
                        ),
                        T::new(
                            "The histogram says how many cells each niche holds.",
                            "L'histogramme indique combien de cellules chaque niche contient.",
                        ),
                        T::new(
                            "The projection shows the niches as they were separated.",
                            "La projection montre les niches telles qu'elles ont été séparées.",
                        ),
                    ]),
                    Block::Paragraph(T::new(
                        "The niche of every cell is written back into the network files, so \
                         the networks can be redrawn coloured by niche and the labels can be \
                         reused elsewhere.",
                        "La niche de chaque cellule est réécrite dans les fichiers réseau : les \
                         réseaux peuvent donc être redessinés colorés par niche, et les \
                         étiquettes réutilisées ailleurs.",
                    )),
                    Block::Callout {
                        kind: CalloutKind::Tip,
                        text: T::new(
                            "A run goes into the folder its Saving directory names, under \
                             Niche_Analysis/Aggregation or Niche_Analysis/Per_sample, with a \
                             copy of the settings it was computed from beside its figures. \
                             Give two runs different names and they sit side by side and can \
                             be compared; give them the same one and the second overwrites \
                             the first.",
                            "Un calcul va dans le dossier que nomme son Saving directory, sous \
                             Niche_Analysis/Aggregation ou Niche_Analysis/Per_sample, avec une \
                             copie des réglages dont il sort à côté de ses figures. Donnez des \
                             noms différents à deux calculs et ils cohabitent et se comparent ; \
                             donnez-leur le même et le second écrase le premier.",
                        ),
                    },
                ],
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Credits
// ---------------------------------------------------------------------------

/// The crates the project stands on.
///
/// Checked against the manifests by `tests/credits.rs`: a dependency that is
/// added without being cited, or cited after being dropped, fails the build.
/// Versions are deliberately not repeated here — they live in `Cargo.toml`,
/// and a number copied into prose is a number that goes out of date.
fn credits() -> Chapter {
    Chapter {
        id: "credits",
        title: T::new("Credits", "Remerciements"),
        sections: vec![
            Section {
                id: "credits-intro",
                title: T::new("What this is built on", "Ce sur quoi c'est bâti"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "This interface is written in Rust; the analyses it starts are the \
                         Python ones. Both halves depend on the work below, and each entry \
                         says what the project actually uses it for — so this is a record of \
                         debts rather than a list of names.",
                        "Cette interface est écrite en Rust ; les analyses qu'elle lance sont \
                         les analyses Python. Les deux moitiés reposent sur le travail qui \
                         suit, et chaque entrée dit ce que le projet en fait réellement : \
                         c'est un relevé de dettes, pas une liste de noms.",
                    )),
                    Block::Paragraph(T::new(
                        "Versions are not repeated here. They are in Cargo.toml, and a number \
                         copied into prose is a number that goes out of date.",
                        "Les versions ne sont pas répétées ici. Elles sont dans Cargo.toml, et \
                         un numéro recopié dans un texte est un numéro qui se périme.",
                    )),
                    Block::Callout {
                        kind: CalloutKind::Note,
                        text: T::new(
                            "This page is checked against the manifests by the test suite: a \
                             dependency added without being credited, or credited after being \
                             dropped, fails the build.",
                            "Cette page est confrontée aux manifestes par la suite de tests : \
                             une dépendance ajoutée sans être créditée, ou créditée après avoir \
                             été retirée, fait échouer la compilation.",
                        ),
                    },
                ],
            },
            Section {
                id: "credits-interface",
                title: T::new("Interface", "Interface"),
                blocks: vec![Block::Citations(vec![
                    C::new("egui", T::new(
                        "The immediate-mode toolkit every panel, button and table is drawn \
                         with. Its immediate mode is why the interface has no widget tree to \
                         keep in sync with the configuration: what is on screen is read \
                         straight out of the document, every frame.",
                        "La boîte à outils en mode immédiat avec laquelle sont dessinés tous \
                         les panneaux, boutons et tableaux. Son mode immédiat explique que \
                         l'interface n'ait aucun arbre de widgets à tenir synchronisé avec la \
                         configuration : ce qui est à l'écran est lu directement dans le \
                         document, à chaque image.")),
                    C::new("eframe", T::new(
                        "Carries egui to a real window: it opens it, runs the event loop, and \
                         picks a graphics backend, which is what lets the same code run on Linux \
                         and on Windows.",
                        "Porte egui jusqu'à une vraie fenêtre : il l'ouvre, fait tourner la \
                         boucle d'événements et choisit un backend graphique — ce qui permet au \
                         même code de tourner sous Linux et sous Windows.")),
                    C::new("egui_extras", T::new(
                        "Two things egui itself does not carry: the tables of the manual's \
                         parameter pages, and the image loaders that display the figures an \
                         analysis produced.",
                        "Deux choses qu'egui ne porte pas lui-même : les tableaux des pages de \
                         paramètres du manuel, et les chargeurs d'images qui affichent les \
                         figures produites par une analyse.")),
                    C::new("rfd", T::new(
                        "The native folder chooser behind the working-directory button. A \
                         hand-drawn file browser would be a worse version of the one the system \
                         already has.",
                        "Le sélecteur de dossier natif derrière le bouton du répertoire de \
                         travail. Un explorateur dessiné à la main serait une version moins bonne \
                         de celui que le système fournit déjà.")),
                    C::new("image", T::new(
                        "Decodes the figures for display, and converts the .ico logo into the \
                         PNG the freedesktop icon theme expects at install time.",
                        "Décode les figures pour l'affichage, et convertit le logo .ico en le PNG \
                         qu'attend le thème d'icônes freedesktop au moment de l'installation.")),
                ])],
            },
            Section {
                id: "credits-data",
                title: T::new(
                    "Reading and writing data",
                    "Lecture et écriture des données",
                ),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "Arrow rather than a dataframe engine, deliberately: what matters when \
                         reading a file the Python wrote is that the column types come back \
                         exactly as they went in.",
                        "Arrow plutôt qu'un moteur de dataframes, délibérément : ce qui compte en \
                         lisant un fichier écrit par le Python, c'est que les types de colonnes \
                         reviennent exactement tels qu'ils sont partis.",
                    )),
                    Block::Citations(vec![
                        C::new("arrow-array", T::new(
                            "The columnar arrays every table in the project is made of, and the \
                             typed access that keeps a float column a float column.",
                            "Les tableaux en colonnes dont sont faites toutes les tables du \
                             projet, et l'accès typé qui garde une colonne de flottants telle \
                             qu'elle est.")),
                        C::new("arrow-schema", T::new(
                            "The description of what a table's columns are called and what they \
                             hold, which is what a round trip through parquet has to preserve.",
                            "La description du nom et du contenu des colonnes d'une table, ce \
                             qu'un aller-retour en parquet doit préserver.")),
                        C::new("arrow-cast", T::new(
                            "Converts between column types when a CSV column read as text turns \
                             out to be numbers.",
                            "Convertit entre types de colonnes quand une colonne CSV lue comme du \
                             texte se révèle être des nombres.")),
                        C::new("arrow-select", T::new(
                            "Takes rows and concatenates batches, which is how a sample is \
                             filtered out of a cohort without copying it column by column.",
                            "Prélève des lignes et concatène des lots : c'est ainsi qu'un \
                             échantillon est extrait d'une cohorte sans être recopié colonne par \
                             colonne.")),
                        C::new("parquet", T::new(
                            "Reads and writes the format the pipelines exchange, so a file \
                             written by this implementation opens in pandas and the other way \
                             round.",
                            "Lit et écrit le format que s'échangent les pipelines : un fichier \
                             écrit par cette implémentation s'ouvre dans pandas, et \
                             réciproquement.")),
                        C::new("csv", T::new(
                            "Reads the CSV and TSV inputs, with the same treatment of blank \
                             lines and empty cells that pandas applies.",
                            "Lit les entrées CSV et TSV, avec le même traitement des lignes vides \
                             et des cellules vides que celui de pandas.")),
                        C::new("serde", T::new(
                            "The serialisation traits the configuration model is built on.",
                            "Les traits de sérialisation sur lesquels repose le modèle de \
                             configuration.")),
                        C::new("serde_yaml", T::new(
                            "Parses configuration.yaml. The emitter is written by hand instead, \
                             to reproduce PyYAML's formatting byte for byte.",
                            "Analyse configuration.yaml. L'écriture, elle, est faite à la main \
                             pour reproduire la mise en forme de PyYAML octet pour octet.")),
                        C::new("serde_json", T::new(
                            "Writes the figure specifications the analyses hand the renderer: \
                             the values, the labels, the colours and the file to write, in a \
                             form that can be read back and redrawn without recomputing \
                             anything.",
                            "Écrit les spécifications de figures que les analyses transmettent \
                             au moteur de rendu : les valeurs, les étiquettes, les couleurs et \
                             le fichier à écrire, sous une forme relisible et redessinable sans \
                             rien recalculer.")),
                        C::new("indexmap", T::new(
                            "A map that remembers its insertion order, which is what lets the \
                             configuration be rewritten with its keys in the order the user's \
                             file had them.",
                            "Une table qui se souvient de son ordre d'insertion : c'est ce qui \
                             permet de réécrire la configuration avec ses clés dans l'ordre du \
                             fichier de l'utilisateur.")),
                    ]),
                ],
            },
            Section {
                id: "credits-science",
                title: T::new(
                    "The analyses and the figures",
                    "Les analyses et les figures",
                ),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "The science is not in this binary. Tysserand builds the networks and \
                         MOSNA computes the assortativity and the niches, both in Python, both \
                         unchanged — the interface starts them as sub-processes and reads their \
                         output. What did change is the drawing: an analysis no longer produces \
                         a PNG, it describes its figures, and the packages below turn each \
                         description into an image and an interactive chart at once.",
                        "La science n'est pas dans ce binaire. Tysserand construit les réseaux \
                         et MOSNA calcule l'assortativité et les niches, tous deux en Python, \
                         tous deux inchangés — l'interface les lance comme sous-processus et lit \
                         leur sortie. Ce qui a changé, c'est le dessin : une analyse ne produit \
                         plus de PNG, elle décrit ses figures, et les paquets ci-dessous \
                         transforment chaque description en une image et un graphique \
                         interactif à la fois.",
                    )),
                    Block::Citations(vec![
                        C::new("rayon", T::new(
                            "Runs a cohort's samples in parallel wherever this side of the \
                             project touches them — reading a table's columns, scanning a \
                             network directory. Its work stealing is why that uses every core \
                             without anything here knowing how many there are.",
                            "Fait tourner en parallèle les échantillons d'une cohorte partout où \
                             ce côté du projet y touche : lecture des colonnes d'une table, \
                             parcours d'un répertoire de réseaux. Son vol de travail explique \
                             que cela occupe tous les coeurs sans que rien ici sache combien il \
                             y en a.")),
                        C::new("xy", T::new(
                            "Draws every figure: the networks, the assortativity heatmaps, the \
                             niche compositions and the projections. It produces the interactive \
                             chart and the image from a single description, which is what lets a \
                             figure be explored rather than only looked at.",
                            "Dessine toutes les figures : réseaux, cartes de chaleur \
                             d'assortativité, compositions de niches et projections. Il produit \
                             le graphique interactif et l'image à partir d'une seule \
                             description, ce qui permet d'explorer une figure et pas seulement \
                             de la regarder.")),
                        C::new("numpy", T::new(
                            "Reads the arrays the analyses hand the renderer. The coordinates of \
                             a hundred thousand cells are written as raw doubles and read back \
                             in one call, which is what keeps drawing a cohort quick.",
                            "Lit les tableaux que les analyses transmettent au moteur de rendu. \
                             Les coordonnées de cent mille cellules sont écrites en réels bruts \
                             et relues en un appel, ce qui garde le dessin d'une cohorte \
                             rapide.")),
                    ]),
                ],
            },
            Section {
                id: "credits-plumbing",
                title: T::new("Plumbing", "Plomberie"),
                blocks: vec![Block::Citations(vec![
                    C::new("anyhow", T::new(
                        "Carries an error up to whoever can report it, keeping the chain of \
                         causes that says which file and which stage failed.",
                        "Fait remonter une erreur jusqu'à qui saura la signaler, en conservant la \
                         chaîne de causes qui dit quel fichier et quelle étape ont échoué.")),
                    C::new("thiserror", T::new(
                        "Declares the error types the library crates expose, so a caller can \
                         match on what went wrong instead of reading a message.",
                        "Déclare les types d'erreur qu'exposent les caisses bibliothèques, pour \
                         qu'un appelant puisse filtrer sur ce qui a échoué plutôt que lire un \
                         message.")),
                    C::new("regex", T::new(
                        "Recognises the sample identifiers in file names, and the \
                         [QT_PROGRESS] lines the interface reads from the analysis process.",
                        "Reconnaît les identifiants d'échantillon dans les noms de fichiers, et \
                         les lignes [QT_PROGRESS] que l'interface lit du processus d'analyse.")),
                ])],
            },
            Section {
                id: "credits-testing",
                title: T::new("Testing", "Tests"),
                blocks: vec![
                    Block::Paragraph(T::new(
                        "The project was written test first throughout. These are what that \
                         rests on.",
                        "Le projet a été écrit en commençant par les tests, du début à la fin. \
                         Voici ce sur quoi cela repose.",
                    )),
                    Block::Citations(vec![
                        C::new("proptest", T::new(
                            "Property-based testing: it invents inputs rather than taking the \
                             ones an author thought of, and shrinks a failure to its smallest \
                             form. It is what found the file-name decoder failing silently on an \
                             identifier that spelt part of a separator.",
                            "Tests par propriétés : il invente les entrées au lieu de prendre \
                             celles auxquelles un auteur a pensé, et réduit un échec à sa forme \
                             la plus simple. C'est lui qui a trouvé le décodeur de noms de \
                             fichiers échouant silencieusement sur un identifiant contenant une \
                             partie d'un séparateur.")),
                        C::new("tempfile", T::new(
                            "Gives every test that touches the disk its own directory, cleaned \
                             up afterwards. It is also what keeps the installer's tests off the \
                             real desktop.",
                            "Donne à chaque test qui touche au disque son propre dossier, nettoyé \
                             ensuite. C'est aussi ce qui tient les tests de l'installeur à l'écart \
                             du vrai bureau.")),
                    ]),
                    Block::Paragraph(T::new(
                        "And the tools around them: rustfmt for the formatting, clippy for the \
                         lints, and GitHub Actions to run all of it on Linux and Windows before \
                         a change lands.",
                        "Et les outils autour : rustfmt pour la mise en forme, clippy pour les \
                         analyses, et GitHub Actions pour exécuter le tout sous Linux et Windows \
                         avant qu'un changement n'entre.",
                    )),
                ],
            },
        ],
    }
}
