# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-03-16

### Added
- 🐾 **宠物远程绑定与聊天功能**
  - 6位数字绑定码生成（10分钟有效期）
  - 双向绑定关系管理
  - 实时 WebSocket 聊天消息
  - Agent 权限配置（AI对话、技能调用）
  - 后端服务（Node.js + WebSocket）

- 💐 **宠物互动功能**
  - 送花💐 - 花瓣飘落粒子效果
  - 送蛋糕🎂 - 金色粒子特效
  - 拥抱🤗 - 温暖蓝色粒子效果
  - 亲亲😘 - 心形飘升动画
  - 支持本地互动和远程宠物间互动

### Changed
- 🔇 移除宠物默认交互提示（如"呼呼睡觉"、"太久没有新互动"等）

### Technical
- 新增 `BindingPanel.vue` 组件
- 新增 `ChatWindow.vue` 组件
- 新增 `InteractionPanel.vue` 组件
- 扩展宠物情绪系统（添加 `loved` 状态）
- 实现粒子动画特效系统

## [0.1.0] - 2026-03-12

### Added
- 初始版本发布
- 基础桌面宠物功能
- PixiJS 渲染引擎
- GSAP 动画系统
- Pinia 状态管理
- 多皮肤支持
- OpenClaw AI 集成