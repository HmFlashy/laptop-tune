# laptop-tune

[English](../README.md) | **Français** | [Español](README.es.md) | [Deutsch](README.de.md) | [中文](README.zh.md) | [日本語](README.ja.md)

CLI, interface graphique native et démon événementiel en Rust pour auditer et optimiser les PC portables sous Linux : gestion d'énergie des GPU hybrides (Intel / AMD / NVIDIA), profils CPU, PCIe ASPM, états de veille, températures et seuils de charge batterie.

## Fonctionnalités

- **`laptop-tune status`** : lecture en direct de la consommation batterie (`W`) et des seuils de charge, des GPU PCI (`driver`, `runtime_status`, veille `D3 suspended`), de `mem_sleep`, du profil CPU actif et des sondes thermiques.
- **`laptop-tune gui [--tray]`** : tableau de bord natif `egui` (suit le thème clair/sombre et la langue du système parmi 14 langues) avec suivi en direct, changement de profil en 1 clic, icône dans la barre des tâches sans polling, lancement au démarrage et bascule auto Batterie / Secteur.
- **`laptop-tune profile <eco|balanced|performance|max> [--dry-run]`** : applique les profils universels via `/sys` (Intel `intel_pstate`/EPP, AMD `amd-pstate`/`cpufreq/boost`, et repli `scaling_governor`).
- **`laptop-tune reset [--dry-run]`** : restaure les valeurs par défaut (`balanced`).
- **`laptop-tune thresholds <début> <arrêt>`** : définit les seuils de charge batterie (ex. `50 80` ou `0 100`).
- **`laptop-tune daemon [--once]`** : démon événementiel (`udevadm monitor`, zéro polling) pour la bascule AC/Batterie, le runtime PM des GPU et les seuils batterie.
- **`laptop-tune generate-rules`** : détecte le matériel et génère les règles `udev` et `tlp.d`.

## Modes d'alimentation

| Mode | ACPI `platform_profile` | CPU (`EPP` / `governor`) | Turbo Boost | PCIe ASPM | Idéal pour |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`eco`** | `quiet` *(repli : `low-power`, `cool`)* | `power` *(ou `powersave`)* | Inactif | `powersupersave` | Autonomie maximale et silence |
| **`balanced`** / **`reset`** | `balanced` | `balance_performance` *(ou `schedutil`)* | Actif | `default` | Usage quotidien équilibré |
| **`performance`** | `performance` | `performance` | Actif | `default` | Compilation, rendu et charges CPU lourdes |
| **`max`** | `performance` | `performance` | Actif | `performance` | Latence PCIe nulle (jeux, CUDA/IA, audio, E/S intensives) |

## Installation

### Universelle (`install.sh`)

```bash
git clone https://github.com/HmFlashy/laptop-tune.git
cd laptop-tune
./install.sh
```

Désinstallation : `./install.sh --uninstall`.

### Arch Linux / Manjaro

```bash
cd packaging/arch
makepkg -si
```

### Ubuntu / Debian (`.deb`)

Téléchargez le `.deb` depuis les [Releases GitHub](https://github.com/HmFlashy/laptop-tune/releases) ou compilez-le localement :

```bash
./packaging/deb/build-deb.sh
sudo apt install ./target/deb/laptop-tune_*.deb
```

## Licence

[MIT](../LICENSE)
