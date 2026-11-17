# laptop-tune

Zero-dependency Rust CLI for auditing and tuning Linux laptops: hybrid GPU runtime power management (Intel / AMD / NVIDIA), CPU power profiles, PCIe ASPM, sleep states, thermals, and battery charge thresholds.

## Features

- **`laptop-tune status`**: Live readout of battery draw (`W`) and charge thresholds, PCI GPUs (`driver`, `runtime_status`, D3 `suspended` check), `mem_sleep`, active CPU profile (`platform_profile`, `EPP`, `turbo`), and thermal zones.
- **`laptop-tune profile <eco|balanced|performance|gaming> [--dry-run]`**: Applies universal kernel `sysfs` profiles (works with Intel `intel_pstate`/EPP, AMD `amd-pstate`/`cpufreq/boost`, and generic `scaling_governor` fallbacks).
- **`laptop-tune generate-rules`**: Detects GPUs and `BAT*` devices to generate `udev` runtime PM rules and `tlp.d` configs.

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

Build a `.deb` package with [`packaging/deb/build-deb.sh`](packaging/deb/build-deb.sh):

```bash
./packaging/deb/build-deb.sh
sudo dpkg -i target/deb/laptop-tune_*.deb
```

> **Note on Flatpak:** Because `laptop-tune` writes directly to kernel `/sys` interfaces, a sandboxed Flatpak would block hardware access; native packages (`.deb`, `PKGBUILD`, or `install.sh`) are recommended.

## Usage

```bash
laptop-tune status
laptop-tune profile eco --dry-run
sudo laptop-tune profile eco
laptop-tune generate-rules
```

## License

[MIT](LICENSE)
