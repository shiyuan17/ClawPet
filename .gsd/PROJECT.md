# 项目说明

## 这是什么

ClawPet 是一个基于 Tauri 的桌面陪伴应用，将可交互桌宠与 OpenClaw 控制台整合在同一个图形界面中。

## 核心价值

用户可以在可视化桌宠界面中运行、配置并与 OpenClaw 助手交互，而不必手动维护原始配置文件。

## 当前状态

应用基于 Vue + Tauri 实现。当前已包含桌宠交互/动画、OpenClaw 对话接入、模型提供商与平台配置、消息渠道与绑定、日志查看、安装与维护等桌面端能力。

## 架构 / 关键模式

- 前端：Vue 3 + Pinia + Vite + TypeScript。
- 渲染与交互：PixiJS + GSAP 驱动桌宠动效与反馈。
- 桌面后端壳层：Tauri v2（Rust），通过命令处理 OpenClaw 运行时、配置读写与系统操作。
- 打包：构建阶段将 OpenClaw runtime / plugins 打包至 `src-tauri/resources`。

## 能力契约

显式能力契约、需求状态与覆盖映射见 `.gsd/REQUIREMENTS.md`。

## 里程碑序列

- [ ] M001: 基线建立 —— 建立需求契约并产出首个可执行路线切片
- [ ] M002: 稳定化 —— 降低超大文件改动风险并提升验证覆盖
