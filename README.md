# laptop-tune

Rust CLI & native desktop GUI for auditing and tuning Linux laptops: hybrid GPU runtime power management (Intel / AMD / NVIDIA), CPU power profiles, PCIe ASPM, sleep states, thermals, and battery charge thresholds.

## Features

- **`laptop-tune status`**: Live readout of battery draw (`W`) and charge thresholds, PCI GPUs (`driver`, `runtime_status`, D3 `suspended` check), `mem_sleep`, active CPU profile (`platform_profile`, `EPP`, `turbo`), and thermal zones.
- **`laptop-tune gui [--tray]`**: Native `egui` desktop dashboard with live monitoring, one-click profile switching (`pkexec`), system tray icon, autostart on login, and last-profile persistence.
- **`laptop-tune profile <eco|balanced|performance|gaming> [--dry-run]`**: Applies universal kernel `sysfs` profiles (Intel `intel_pstate`/EPP, AMD `amd-pstate`/`cpufreq/boost`, and generic `scaling_governor` fallbacks).
- **`laptop-tune reset [--dry-run]`**: Restores safe default (`balanced`) kernel settings.
- **`laptop-tune generate-rules`**: Detects GPUs and `BAT*` devices to generate `udev` runtime PM rules and `tlp.d` configs.

## Power Modes

| Mode | ACPI `platform_profile` | CPU (`EPP` / `governor`) | Turbo Boost | PCIe ASPM | Best for |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`eco`** | `quiet` *(fallback: `low-power`, `cool`)* | `power` *(or `powersave`)* | Off | `powersupersave` | Maximum battery life & silent fans |
| **`balanced`** / **`reset`** | `balanced` | `balance_performance` *(or `schedutil`)* | On | `default` | Everyday use & safe default |
| **`performance`** | `performance` | `performance` | On | `default` | Compilation, rendering & heavy CPU loads |
| **`gaming`** | `performance` | `performance` | On | `performance` | Gaming (avoids PCIe sleep micro-stutters on dGPU/NVMe) |

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

> **Note on Flatpak:** Because `laptop-tune` writes directly to kernel `/sys` interfaces, a sandboxed Flatpak would block hardware access; native packages (`.deb`, `PKGBUILD`, or `install.sh`) are recommended.

## Usage

```bash
laptop-tune status
laptop-tune gui --tray
laptop-tune profile eco --dry-run
sudo laptop-tune profile eco
sudo laptop-tune reset
laptop-tune generate-rules
```

## License

[MIT](LICENSE)
