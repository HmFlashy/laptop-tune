# laptop-tune

**English** | [Français](docs/README.fr.md) | [Español](docs/README.es.md) | [Deutsch](docs/README.de.md) | [中文](docs/README.zh.md) | [日本語](docs/README.ja.md)

Rust CLI, native desktop GUI, and event-driven daemon for auditing and tuning Linux laptops: hybrid GPU runtime power management (Intel / AMD / NVIDIA), CPU power profiles, PCIe ASPM, sleep states, thermals, and battery charge thresholds.

## Features

- **`laptop-tune status`**: Live readout of battery draw (`W`) and charge thresholds, PCI GPUs (`driver`, `runtime_status`, D3 `suspended` check), `mem_sleep`, active CPU profile (`platform_profile`, `EPP`, `turbo`), and thermal zones.
- **`laptop-tune gui [--tray]`**: Native `egui` desktop dashboard (follows system light/dark theme and 14 system locales) with live monitoring, one-click profile switching, zero-polling system tray icon, autostart on login, and automatic AC/Battery switching.
- **`laptop-tune profile <eco|balanced|performance|max> [--dry-run]`**: Applies universal kernel `sysfs` profiles (Intel `intel_pstate`/EPP, AMD `amd-pstate`/`cpufreq/boost`, and generic `scaling_governor` fallbacks).
- **`laptop-tune reset [--dry-run]`**: Restores safe default (`balanced`) kernel settings.
- **`laptop-tune thresholds <start> <stop>`**: Sets battery start/stop charge thresholds (e.g. `50 80` or `0 100`).
- **`laptop-tune daemon [--once]`**: Event-driven daemon (`udevadm monitor`, zero polling) for automatic AC/Battery profile switching, GPU runtime PM, and battery thresholds.
- **`laptop-tune generate-rules`**: Detects GPUs and `BAT*` devices to generate `udev` runtime PM rules and `tlp.d` configs.

## Power Modes

| Mode | ACPI `platform_profile` | CPU (`EPP` / `governor`) | Turbo Boost | PCIe ASPM | Best for |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`eco`** | `quiet` *(fallback: `low-power`, `cool`)* | `power` *(or `powersave`)* | Off | `powersupersave` | Maximum battery life & silent fans |
| **`balanced`** / **`reset`** | `balanced` | `balance_performance` *(or `schedutil`)* | On | `default` | Everyday use & safe default |
| **`performance`** | `performance` | `performance` | On | `default` | Compilation, rendering & heavy CPU loads |
| **`max`** | `performance` | `performance` | On | `performance` | Zero PCIe sleep latency (gaming, CUDA/AI, audio, heavy I/O) |

## Installation

### Universal (`install.sh`)

```bash
git clone https://github.com/HmFlashy/laptop-tune.git
cd laptop-tune
./install.sh
```

Uninstall with `./install.sh --uninstall`.

### Arch Linux / Manjaro

Using the included [`packaging/arch/PKGBUILD`](packaging/arch/PKGBUILD):

```bash
cd packaging/arch
makepkg -si
```

### Ubuntu / Debian (`.deb`)

Download the `.deb` from [GitHub Releases](https://github.com/HmFlashy/laptop-tune/releases) or build it locally with [`packaging/deb/build-deb.sh`](packaging/deb/build-deb.sh):

```bash
./packaging/deb/build-deb.sh
sudo apt install ./target/deb/laptop-tune_*.deb
```

## Usage

```bash
laptop-tune status
laptop-tune gui --tray
laptop-tune profile eco --dry-run
sudo laptop-tune profile eco
sudo laptop-tune reset
sudo laptop-tune thresholds 50 80
laptop-tune generate-rules
```

## License

[MIT](LICENSE)
