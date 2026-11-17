# laptop-tune

[English](../README.md) | [Français](README.fr.md) | [Español](README.es.md) | **Deutsch** | [中文](README.zh.md) | [日本語](README.ja.md)

Rust-CLI, native Desktop-GUI und ereignisgesteuerter Daemon zur Analyse und Optimierung von Linux-Laptops: Hybrid-GPU-Energiemanagement (Intel / AMD / NVIDIA), CPU-Energieprofile, PCIe ASPM, Standby-Modi, Temperaturen und Akku-Ladeschwellen.

## Funktionen

- **`laptop-tune status`**: Echtzeit-Anzeige von Akkuverbrauch (`W`), Ladeschwellen, PCI-GPUs (`D3 suspended`), `mem_sleep`, aktivem Profil und Temperaturen.
- **`laptop-tune gui [--tray]`**: Natives `egui`-Dashboard (folgt dem hellen/dunklen Systemthema und 14 Systemsprachen) mit Live-Überwachung, 1-Klick-Profilwechsel, System-Tray-Icon ohne Polling und automatischem Akku-/Netzbetrieb-Wechsel.
- **`laptop-tune profile <eco|balanced|performance|max> [--dry-run]`**: Wendet universelle Kernel-`/sys`-Profile für Intel, AMD und generische CPUs an.
- **`laptop-tune reset [--dry-run]`**: Stellt die sicheren Standardwerte (`balanced`) wieder her.
- **`laptop-tune thresholds <start> <stop>`**: Setzt Akku-Ladeschwellen (z. B. `50 80`).
- **`laptop-tune daemon [--once]`**: Ereignisgesteuerter Daemon (`udevadm monitor`, kein Polling).

## Entwickelt für Single-GPU- & Hybrid-Dual-GPU-Laptops (Optimus / PRIME)

- **Alle Linux-Laptops (nur Intel / AMD iGPU)**: Volle Unterstützung für CPU-Profile (`EPP` / `scaling_governor`), Turbo Boost, ACPI `platform_profile`, PCIe ASPM, Temperatursensoren und Akku-Ladeschwellen (`50 % – 80 %`).
- **Speziell für Hybrid-Dual-GPU-Laptops (NVIDIA Optimus / AMD PRIME)**: Hält die dedizierte GPU über `udev`-Regeln (`/etc/udev/rules.d/90-laptop-tune-gpu.rules`) dauerhaft im **`D3cold`-Schlaf (`SUSPENDED · auto`)** und nutzt OpenGL (`glow`) auf der iGPU, damit das Dashboard die NVIDIA/AMD-dGPU niemals aufweckt.
- **Dedizierte GPU bei Bedarf nutzen (`prime-run` / Gaming / CUDA)**:
  - **CLI (NVIDIA)**: `prime-run <app>`
  - **CLI (AMD dGPU)**: `DRI_PRIME=1 <app>`
  - **Steam (Startoptionen)**: `prime-run %command%`
  - **Lutris / Heroic / KDE / GNOME**: Option *Mit dedizierter Grafikkarte starten* (*Prime Render Offload*) aktivieren.
- **Konfliktfreie Koexistenz mit TLP**: Installiert automatisch [`99-laptop-tune-override.conf`](../packaging/99-laptop-tune-override.conf), falls TLP vorhanden ist.
- **Akku-Bypass (`Angeschlossen (lädt nicht)` zwischen Schwellenwerten)**: Zwischen z. B. `50 %` und `80 %` läuft der Laptop direkt über Netzstrom (`~0.00 W`), um Mikro-Ladezyklen zu vermeiden.

## Energiemodi

| Modus | ACPI `platform_profile` | CPU (`EPP` / `governor`) | Turbo Boost | PCIe ASPM | Ideal für |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`eco`** | `quiet` | `power` | Aus | `powersupersave` | Maximale Akkulaufzeit & leise Lüfter |
| **`balanced`** / **`reset`** | `balanced` | `balance_performance` | Ein | `default` | Täglicher Gebrauch & Standard |
| **`performance`** | `performance` | `performance` | Ein | `default` | Kompilierung, Rendering & hohe CPU-Last |
| **`max`** | `performance` | `performance` | Ein | `performance` | Keine PCIe-Schlaflatenz (Gaming, CUDA/KI, Audio) |

## Installation

```bash
git clone https://github.com/HmFlashy/laptop-tune.git
cd laptop-tune
./install.sh
```

## Lizenz

[MIT](../LICENSE)
