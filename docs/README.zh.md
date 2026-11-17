# laptop-tune

[English](../README.md) | [Français](README.fr.md) | [Español](README.es.md) | [Deutsch](README.de.md) | **中文** | [日本語](README.ja.md)

基于 Rust 编写的 Linux 笔记本电源与性能调优工具（含命令行、原生桌面 GUI 与事件驱动守护进程）：支持混合显卡运行时电源管理（Intel / AMD / NVIDIA）、CPU 性能模式、PCIe ASPM、休眠状态、温度监控以及电池充电阈值管理。

## 功能特性

- **`laptop-tune status`**：实时查看电池功耗（`W`）、充电阈值、PCI 显卡状态（`D3 suspended` 休眠检测）、`mem_sleep`、当前 CPU 策略与温度传感器。
- **`laptop-tune gui [--tray]`**：原生 `egui` 桌面面板（自动跟随系统浅色/深色主题及 14 种系统语言），支持一键切换模式、零轮询系统托盘图标、开机自启以及电池/电源自动切换。
- **`laptop-tune profile <eco|balanced|performance|max> [--dry-run]`**：应用通用 Linux `/sys` 内核性能配置（兼容 Intel、AMD 及通用调频器）。
- **`laptop-tune reset [--dry-run]`**：恢复安全默认设置（`balanced`）。
- **`laptop-tune thresholds <start> <stop>`**：设置电池起止充电阈值（例如 `50 80`）。
- **`laptop-tune daemon [--once]`**：基于 `udevadm monitor` 的零轮询事件驱动守护进程。

## 电源模式

| 模式 | ACPI `platform_profile` | CPU (`EPP` / `governor`) | 睿频加速 (Turbo) | PCIe ASPM | 适用场景 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`eco`** | `quiet` | `power` | 关闭 | `powersupersave` | 最长续航与静音风扇 |
| **`balanced`** / **`reset`** | `balanced` | `balance_performance` | 开启 | `default` | 日常均衡使用 |
| **`performance`** | `performance` | `performance` | 开启 | `default` | 编译、渲染与高负载计算 |
| **`max`** | `performance` | `performance` | 开启 | `performance` | 零 PCIe 唤醒延迟（游戏、CUDA/AI、低延迟音频） |

## 安装

```bash
git clone https://github.com/HmFlashy/laptop-tune.git
cd laptop-tune
./install.sh
```

## 许可证

[MIT](../LICENSE)
