import channelDingtalkIcon from "../../images/channels/dingtalk.svg";
import channelDiscordIcon from "../../images/channels/discord.svg";
import channelFeishuIcon from "../../images/channels/feishu.svg";
import channelQqIcon from "../../images/channels/qq.svg";
import channelTelegramIcon from "../../images/channels/telegram.svg";
import channelWhatsappIcon from "../../images/channels/whatsapp.svg";
import channelWecomIcon from "../../images/channels/wecom.svg";
import channelWeixinIcon from "../../images/channels/weixin.svg";
import packageJson from "../../../package.json";
import type {
  AgentPaneTab,
  ChannelPaneCatalogItem,
  CreateEmployeeIdentityOption,
  CreateEmployeeModalStep,
  CreateEmployeeRuleItem,
  SidebarAvatarCategoryId,
  SidebarItem,
  SidebarSettingsAppearance,
  SidebarSettingsLanguage,
  SidebarSettingsMenuGroup,
  SidebarSettingsMenuGroupId,
  SidebarThemeMode,
  SidebarThemePreset,
  TaskBoardGroupMode,
  TaskBoardStatus
} from "./types";

export const sidebarItems: SidebarItem[] = [
  { id: "dashboard", label: "仪表盘" },
  { id: "chat", label: "聊天" },
  { id: "tasks", label: "任务管理" },
  { id: "recruitment", label: "数字员工" },
  { id: "skills", label: "技能市场" },
  { id: "market", label: "商城" }
];
export const sidebarSettingsMenuGroups: SidebarSettingsMenuGroup[] = [
  { id: "general", label: "通用设置" },
  { id: "providers", label: "模型商管理" },
  { id: "about", label: "关于我们" }
];
export const sidebarSettingsAppearanceOptions: Array<{ id: SidebarSettingsAppearance; label: string }> = [
  { id: "system", label: "跟随系统" },
  { id: "light", label: "浅色" },
  { id: "dark", label: "深色" }
];
export const sidebarThemePresetOptions: Array<{ id: SidebarThemePreset; label: string }> = [
  { id: "elegant", label: "淡雅" },
  { id: "frosted", label: "磨砂" },
  { id: "pure-white", label: "纯白" }
];
export const sidebarThemeModeOptions: Array<{ id: SidebarThemeMode; label: string }> = [
  { id: "day", label: "日间" },
  { id: "night", label: "夜间" }
];
export const sidebarSettingsLanguageOptions: Array<{ id: SidebarSettingsLanguage; label: string }> = [
  { id: "zh-CN", label: "简体中文" },
  { id: "en-US", label: "English" }
];
export const sidebarSettingsShortcutItems = [
  { id: "toggle-window", label: "显示或隐藏窗口", value: "Ctrl+` / Alt+`", note: "全局快捷键，可在任意应用中呼出。" },
  { id: "open-chat", label: "打开聊天窗口", value: "Alt+1", note: "快速回到主聊天界面。" }
];
export const sidebarSettingsTips = [
  "在聊天页右键会话，点击“信息”可查看员工状态与排班信息。",
  "日志页支持复制请求/响应详情，便于排查问题。",
  "技能市场可按分类和评分筛选，优先启用高分技能。"
];
export const SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY = "keai.desktop-pet.sidebar-settings.appearance";
export const SIDEBAR_SETTINGS_LANGUAGE_STORAGE_KEY = "keai.desktop-pet.sidebar-settings.language";
export const SIDEBAR_THEME_PRESET_STORAGE_KEY = "keai.desktop-pet.sidebar-theme.preset";
export const SIDEBAR_THEME_MODE_STORAGE_KEY = "keai.desktop-pet.sidebar-theme.mode";
export const SIDEBAR_AGENT_AVATAR_OVERRIDES_STORAGE_KEY = "keai.desktop-pet.sidebar-avatar.overrides";
export const packageVersionFallback =
  typeof packageJson.version === "string" && packageJson.version.trim() ? packageJson.version.trim() : "0.2.0";
export const FEISHU_CHANNEL_ID = "feishu";
export const FEISHU_DEFAULT_ACCOUNT_ID = "default";
export const FEISHU_PLUGIN_PACKAGE_NAME = "@larksuite/openclaw-lark";
export const FEISHU_DOCS_URL = "https://www.feishu.cn/content/article/7613711414611463386";
export const FEISHU_APP_ID_PLACEHOLDER = "cli_xxxxxxxxxxxxxxxx";
export const FEISHU_APP_SECRET_PLACEHOLDER = "请输入飞书应用的 Secret";
export const CHANNEL_QR_BINDING_POLL_INTERVAL_MS = 2000;
export const WHATSAPP_QR_BINDING_RETRY_HINT_DELAY_MS = 60 * 1000;

export const taskStatusFlow: TaskBoardStatus[] = ["todo", "in_progress", "in_review", "done", "cancelled"];
export const TASK_BOARD_FILTER_ALL = "__all__";
export const taskBoardGroupModeOptions: Array<{ id: TaskBoardGroupMode; label: string }> = [
  { id: "status", label: "按状态" },
  { id: "agent", label: "按 Agent" },
  { id: "team", label: "按团队" }
];
export const taskBoardColumns: Array<{ id: TaskBoardStatus; label: string; subtitle: string; emptyText: string }> = [
  { id: "todo", label: "To do", subtitle: "待办事项", emptyText: "暂无待办任务。" },
  { id: "in_progress", label: "In progress", subtitle: "进行中", emptyText: "暂无进行中的任务。" },
  { id: "in_review", label: "In review", subtitle: "回顾", emptyText: "暂无待回顾任务。" },
  { id: "done", label: "Done", subtitle: "完成", emptyText: "暂无已完成任务。" },
  { id: "cancelled", label: "Cancelled", subtitle: "取消", emptyText: "暂无已取消任务。" }
];
export const agentPaneTabs: Array<{ id: AgentPaneTab; label: string }> = [
  { id: "staff", label: "数字员工" },
  { id: "channel", label: "频道" },
  { id: "group", label: "团队" }
];
export const chatChannelCatalog: ChannelPaneCatalogItem[] = [
  {
    id: "weixin",
    name: "微信",
    description: "微信消息触达与机器人接入",
    icon: channelWeixinIcon,
    connectionMode: "qr",
    aliases: ["wechat", "wx", "wechat_official_account", "wechat-official-account", "openclaw-weixin", "openclaw_weixin"],
    instructions: [],
    fields: []
  },
  {
    id: "feishu",
    name: "飞书",
    description: "飞书机器人与消息通知",
    icon: channelFeishuIcon,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/GKn8wOvHnibpPNkNkPzcAvGlnzK#Py88dTltfoJc1jxAhIBcW3Pkn7b",
    instructions: [
      "前往 飞书开放平台 (open.feishu.cn) 并创建企业自建应用",
      "在应用详情页获取 App ID 和 App Secret 并填入",
      "确保应用已开通“机器人”能力",
      "保存配置后，根据网关提示完成机器人创建"
    ],
    fields: [
      { key: "appId", label: "应用 ID (App ID)", placeholder: "cli_xxxxxx", required: true },
      { key: "appSecret", label: "应用密钥 (App Secret)", placeholder: "输入应用密钥", required: true, secret: true }
    ]
  },
  {
    id: "wecom",
    name: "企业微信",
    description: "企业微信应用与群机器人",
    icon: channelWecomIcon,
    connectionMode: "qr",
    aliases: ["workwechat", "wechatwork", "qywx", "openclaw-wecom", "openclaw_wecom"],
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/JTGnwoV0RixKPtkr4w7c7gpAnDc",
    instructions: [],
    fields: []
  },
  {
    id: "dingtalk",
    name: "钉钉",
    description: "钉钉机器人与工作通知",
    icon: channelDingtalkIcon,
    aliases: ["dingding"],
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/Y5eNwiSiZidkLskrwtJc1rUln0b#doxcnr8KfaA2mNPeQUeHO83eDPh",
    instructions: [
      "点击“安装钉钉插件”并完成组件安装",
      "在钉钉开放平台创建应用，获取 Client ID 与 Client Secret",
      "保存后返回频道列表检查连接状态"
    ],
    fields: [
      { key: "clientId", label: "Client ID", placeholder: "dingxxxx（在钉钉开放平台应用信息中获取）", required: true },
      { key: "clientSecret", label: "Client Secret", placeholder: "在钉钉开放平台应用凭证中获取", required: true, secret: true }
    ]
  },
  {
    id: "qq",
    name: "QQ",
    description: "QQ群机器人与私聊触达",
    icon: channelQqIcon,
    backendChannelType: "qqbot",
    aliases: ["qqbot", "qq-bot"],
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/KPIJwlyiGiupMrkiS9ice39Zn2c",
    instructions: ["前往 QQ 机器人开放平台创建应用", "获取 App ID 与 Client Secret", "填写凭证后保存并连接"],
    fields: [
      { key: "appId", label: "App ID", placeholder: "输入 QQ 机器人 App ID", required: true },
      { key: "clientSecret", label: "Client Secret", placeholder: "输入 Client Secret", required: true, secret: true }
    ]
  },
  {
    id: "telegram",
    name: "Telegram",
    description: "Bot API 多账号接入",
    icon: channelTelegramIcon,
    aliases: ["tg"],
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/TjiGwxsMWi7hpDkDAQBc0ydMnEf#PL8ndvsEwoYVWIx1T4mcB1EvnSb",
    instructions: [
      "在 Telegram 中搜索 @BotFather，发送 /newbot 创建机器人",
      "按提示设置机器人名称，成功后 BotFather 会返回 Bot Token",
      "将 Bot Token 填入下方表单并保存",
      "把机器人拉入目标群，或私聊机器人完成授权"
    ],
    fields: [
      { key: "botToken", label: "Bot Token", placeholder: "粘贴来自 @BotFather 的 Token", required: true, secret: true }
    ]
  },
  {
    id: "whatsapp",
    name: "WhatsApp",
    description: "WhatsApp 消息收发与会话接入",
    icon: channelWhatsappIcon,
    connectionMode: "qr",
    aliases: ["wa", "wacli", "openclaw-whatsapp", "openclaw_whatsapp"],
    docsUrl: "https://docs.openclaw.ai/channels/whatsapp",
    instructions: [],
    fields: []
  },
  {
    id: "discord",
    name: "Discord",
    description: "Guild / Channel 事件联动",
    icon: channelDiscordIcon,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/BkOywJYCAiYRN9k4KTTceKPMnxg#C9zjdBRT1oqZ4VxF8q7ceRxQnLk",
    instructions: [
      "打开 Discord Developer Portal，创建应用并添加 Bot",
      "进入应用 > Bot 页面点击 Reset Token 获取 Bot Token，并开启 Message Content Intent",
      "在下方填写凭证并生成授权链接，将机器人添加到目标服务器"
    ],
    fields: [
      { key: "appId", label: "Application ID", placeholder: "例如 1470267845714645176", required: true },
      { key: "token", label: "Bot Token", placeholder: "从开发者门户 > Bot > Token 粘贴", required: true, secret: true }
    ]
  }
];
export const CHANNEL_CONFIG_GUIDED_LAYOUT_IDS = new Set(["dingtalk", "telegram", "discord"]);
export const CHANNEL_CONFIG_GUIDED_FIELD_ORDER: Record<string, string[]> = {
  dingtalk: ["clientId", "clientSecret"],
  telegram: ["botToken"],
  discord: ["appId", "token"]
};
export const chatChannelCatalogMap = new Map(chatChannelCatalog.map((channel) => [channel.id, channel]));
export const chatChannelAliasMap = new Map<string, string>();
for (const channel of chatChannelCatalog) {
  chatChannelAliasMap.set(channel.id, channel.id);
  for (const alias of channel.aliases ?? []) {
    chatChannelAliasMap.set(alias, channel.id);
  }
}
for (const alias of ["work-wechat", "work_wechat"]) {
  chatChannelAliasMap.set(alias, "wecom");
}
for (const alias of ["ding-talk", "ding_talk"]) {
  chatChannelAliasMap.set(alias, "dingtalk");
}
for (const alias of ["wechat-official", "wechat_service"]) {
  chatChannelAliasMap.set(alias, "weixin");
}
for (const alias of ["tencent-qq"]) {
  chatChannelAliasMap.set(alias, "qq");
}
for (const alias of ["qqbot"]) {
  chatChannelAliasMap.set(alias, "qq");
}
for (const alias of ["lark"]) {
  chatChannelAliasMap.set(alias, "feishu");
}
for (const alias of ["whats-app"]) {
  chatChannelAliasMap.set(alias, "whatsapp");
}

const agentAvatarModules = import.meta.glob("../../images/avatar/*.{png,jpg,jpeg,webp,avif,svg}", {
  eager: true,
  import: "default"
}) as Record<string, string>;
export const agentAvatarPool = Object.entries(agentAvatarModules)
  .sort(([leftPath], [rightPath]) => leftPath.localeCompare(rightPath, "en"))
  .map(([, url]) => url)
  .filter((url) => typeof url === "string" && url.trim().length > 0);
export const SIDEBAR_AVATAR_PRESET_COUNT_PER_CATEGORY = 14;
export const SIDEBAR_AVATAR_UPLOAD_MAX_BYTES = 2 * 1024 * 1024;
export const sidebarAvatarCategoryTabs: Array<{ id: SidebarAvatarCategoryId; label: string }> = [
  { id: "pixel", label: "像素风" },
  { id: "illustration", label: "插画" },
  { id: "animal", label: "动物" },
  { id: "cute", label: "手绘风" }
];
export const SIDEBAR_PIXEL_AVATAR_PALETTES = [
  { bg: "#6f85ff", frame: "#4156aa", skin: "#f6cfaf", hair: "#22273d", eye: "#0d1224", top: "#eef2ff", accent: "#9eb0ff" },
  { bg: "#5db4ff", frame: "#2f709f", skin: "#f8d8ba", hair: "#5e3a24", eye: "#122039", top: "#f8f1e7", accent: "#9ad5ff" },
  { bg: "#7fda9a", frame: "#3b8a57", skin: "#f2cdb2", hair: "#253243", eye: "#152238", top: "#f4fff3", accent: "#9ce8b4" },
  { bg: "#ffa6a6", frame: "#b35f63", skin: "#f9d5be", hair: "#3b2b2b", eye: "#23161b", top: "#fff3f3", accent: "#ffc9c9" },
  { bg: "#c695ff", frame: "#6c4ba8", skin: "#f7d7c5", hair: "#2c2148", eye: "#101026", top: "#f5ecff", accent: "#dac1ff" },
  { bg: "#f4ca6b", frame: "#ad8435", skin: "#f4cfb2", hair: "#2f2a1e", eye: "#1b1a15", top: "#fff7e4", accent: "#f8de9f" },
  { bg: "#6dd0c8", frame: "#2d857d", skin: "#f5d8c0", hair: "#2f3c31", eye: "#0d1c16", top: "#ebfffb", accent: "#9be5de" },
  { bg: "#8db6ff", frame: "#4f71b8", skin: "#f6d1b9", hair: "#2f2f4f", eye: "#121a33", top: "#eef5ff", accent: "#c0d4ff" },
  { bg: "#ffb774", frame: "#b8713f", skin: "#f7d9c1", hair: "#3f2f21", eye: "#20150f", top: "#fff2e3", accent: "#ffd3a8" },
  { bg: "#97e58e", frame: "#51914a", skin: "#f5d4ba", hair: "#2a3b2d", eye: "#132018", top: "#f3fff0", accent: "#bdf0b7" },
  { bg: "#ff95d8", frame: "#b75493", skin: "#f8d6c6", hair: "#3e2442", eye: "#1f1231", top: "#fff0fb", accent: "#ffc5ea" },
  { bg: "#79d5ff", frame: "#3f7ea6", skin: "#f6d7bd", hair: "#213246", eye: "#122130", top: "#ecf8ff", accent: "#afe4ff" },
  { bg: "#a6b1ff", frame: "#5f6ab7", skin: "#f7d8c0", hair: "#2e2d54", eye: "#171734", top: "#f1f3ff", accent: "#cad0ff" },
  { bg: "#ffae98", frame: "#b86455", skin: "#f8d4bc", hair: "#3f3028", eye: "#241a16", top: "#fff1ed", accent: "#ffd2c7" }
] as const;
export const SIDEBAR_HANDDRAWN_AVATAR_PALETTES = [
  { bg: "#2d6ff5", skin: "#f6dfe2", hair: "#f8f8f8", top: "#071737", lens: "#7262df", accent: "#f08e8e" },
  { bg: "#4e7dff", skin: "#f9dfcc", hair: "#273245", top: "#0f233f", lens: "#6cb5ff", accent: "#ef9a87" },
  { bg: "#4abf9a", skin: "#f7dbc8", hair: "#232d2f", top: "#102635", lens: "#8f7cff", accent: "#f19680" },
  { bg: "#7b74ff", skin: "#f3d6db", hair: "#efeef9", top: "#101d44", lens: "#5a78ff", accent: "#e88692" },
  { bg: "#5a9df6", skin: "#f8ddc9", hair: "#f2e1c5", top: "#112548", lens: "#6f8fff", accent: "#f2a57f" },
  { bg: "#3e8ddf", skin: "#f6d7c9", hair: "#412f28", top: "#0d1b3a", lens: "#8b6ee9", accent: "#ee8f8c" },
  { bg: "#3570cf", skin: "#f5dad2", hair: "#f0f5ff", top: "#0a1a38", lens: "#7d6ce3", accent: "#eb9287" }
] as const;

export const CHAT_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-history";
export const SESSION_STORAGE_PREFIX = "keai.desktop-pet.openclaw.session-id";
export const CHAT_ARCHIVE_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-archives";
export const STARTUP_OPENCLAW_HEALTHY_MARK_STORAGE_KEY = "keai.desktop-pet.openclaw.startup-healthy.v1";
export const VIRTUAL_OPENCLAW_SESSION_CHANNEL_TYPE = "__openclaw_session__";
export const CHAT_USER_GROUPS_STORAGE_KEY = "keai.desktop-pet.chat-user-groups";
export const CHAT_USER_GROUP_MEMBERSHIP_STORAGE_KEY = "keai.desktop-pet.chat-user-group-membership";
export const CHAT_CONVERSATION_GROUPS_STORAGE_KEY = "keai.desktop-pet.chat-conversation-groups";
export const CHAT_USER_AGENT_ORDER_STORAGE_KEY = "keai.desktop-pet.chat-user-agent-order";
export const CHAT_USER_COLLAPSED_SECTIONS_STORAGE_KEY = "keai.desktop-pet.chat-user-collapsed-sections";
export const CHAT_USER_CUSTOM_AGENTS_STORAGE_KEY = "keai.desktop-pet.chat-user-custom-agents";
export const CHAT_USER_PINNED_GROUP_ID = "builtin:pinned";
export const CHAT_CONVERSATION_GROUP_AGENT_PREFIX = "chat-group:";
export const CHAT_CONVERSATION_GROUP_DEFAULT_POLICY = "allowlist";
export const CHAT_CONVERSATION_GROUP_DEFAULT_ALLOW_FROM = ["local:*"] as const;
export const CHAT_CONVERSATION_GROUP_DEFAULT_REQUIRE_MENTION = true;
export const CHAT_CONVERSATION_GROUP_DEFAULT_ACTIVATION_COMMAND = "/activation";
export const CHAT_CONVERSATION_GROUP_MIN_HISTORY_LIMIT = 8;
export const CHAT_CONVERSATION_GROUP_MAX_HISTORY_LIMIT = 80;
export const CHAT_CONVERSATION_GROUP_DEFAULT_HISTORY_LIMIT = 40;
export const CHAT_MARKDOWN_RENDER_CACHE_MAX = 220;
export const CHAT_FILE_DETECTION_CACHE_MAX = 220;
export const CHAT_FILE_UNIX_PATH_PATTERN = /(?:^|[\s"'`([{<])((?:\/|~\/)[^\s"'`<>|]+?\.[a-z0-9]{1,12})(?=$|[\s"')\]}>.,;:!?，。；：！？`])/gim;
export const CHAT_FILE_WINDOWS_PATH_PATTERN = /(?:^|[\s"'`([{<])([a-z]:\\[^\s"'`<>|]+?\.[a-z0-9]{1,12})(?=$|[\s"')\]}>.,;:!?，。；：！？`])/gim;
export const CHAT_FILE_URL_PATTERN = /(?:^|[\s"'`([{<])(file:\/\/\/?[^\s"'`<>|]+?\.[a-z0-9]{1,12})(?=$|[\s"')\]}>.,;:!?，。；：！？`])/gim;
export const CHAT_FILE_PATH_TRAILING_TOKENS = new Set([".", ",", ";", ":", "!", "?", "，", "。", "；", "：", "！", "？", ")", "]", "}", ">", "`"]);
export const CHAT_FILE_IMAGE_EXTENSIONS = new Set([
  "png",
  "jpg",
  "jpeg",
  "gif",
  "webp",
  "bmp",
  "avif",
  "svg",
  "heic",
  "heif",
  "tif",
  "tiff",
  "ico",
  "jfif"
]);
export const CHAT_FILE_AUDIO_EXTENSIONS = new Set([
  "mp3",
  "wav",
  "ogg",
  "m4a",
  "aac",
  "flac",
  "opus",
  "wma",
  "amr",
  "aif",
  "aiff",
  "caf",
  "alac"
]);
export const CHAT_FILE_VIDEO_EXTENSIONS = new Set([
  "mp4",
  "webm",
  "mov",
  "m4v",
  "mkv",
  "avi",
  "wmv",
  "flv",
  "mpg",
  "mpeg",
  "3gp",
  "ogv",
  "ts",
  "m2ts",
  "mts"
]);
export const CHAT_FILE_HTML_EXTENSIONS = new Set(["html", "htm"]);
export const ROLE_WORKFLOW_OVERRIDES_STORAGE_KEY = "keai.desktop-pet.role-workflow-overrides";
export const RECRUITMENT_DIVISION_FILTER_ALL = "__all__";
export const CREATE_EMPLOYEE_IDENTITY_OPTIONS: CreateEmployeeIdentityOption[] = [
  { id: "efficiency", icon: "🧮", name: "效率专家", desc: "擅长分析、规划、总结" },
  { id: "creative", icon: "🎨", name: "创意伙伴", desc: "擅长写作、头脑风暴、设计" },
  { id: "coach", icon: "👨‍🏫", name: "知识教练", desc: "擅长教学、解释、答疑" },
  { id: "advisor", icon: "💼", name: "专业顾问", desc: "模拟法律、财务、技术等角色" },
  { id: "companion", icon: "😄", name: "趣味玩伴", desc: "擅长聊天、游戏、角色扮演" },
  { id: "researcher", icon: "🔍", name: "研究助手", desc: "擅长查找、对比、调研" }
];
export const CREATE_EMPLOYEE_RANDOM_NAMES = [
  "财税小专家",
  "灵感火花",
  "决策导航仪",
  "行程百事通",
  "代码医生",
  "创意引擎",
  "学习伙伴",
  "分析大师",
  "写作助手",
  "策划专家",
  "翻译官",
  "记忆管家"
];
export const CREATE_EMPLOYEE_TASK_EXAMPLES = [
  "生成会议纪要",
  "润色邮件文案",
  "生成Python代码片段",
  "进行SWOT分析",
  "制定旅行计划",
  "总结长篇文章"
];
export const CREATE_EMPLOYEE_STEPS: Array<{ id: CreateEmployeeModalStep; label: string }> = [
  { id: 1, label: "定义身份" },
  { id: 2, label: "设定能力" },
  { id: 3, label: "调整性格" }
];
export const CREATE_EMPLOYEE_DEFAULT_RULES: CreateEmployeeRuleItem[] = [
  { id: "rule-reference", text: "所有建议仅供参考，不构成专业（医疗/法律/财务）意见。", enabled: true },
  { id: "rule-safe", text: "回答应安全、合规、健康。", enabled: true },
  { id: "rule-honest", text: "在不确定答案时，应诚实告知。", enabled: true }
];
export const CREATE_EMPLOYEE_MAX_TASKS = 20;
export const CREATE_EMPLOYEE_MAX_RULES = 12;
export const UTILITY_LOG_ROLE_FILTER_ACTIVE = "__active__";
export const UTILITY_LOG_ROLE_FILTER_ALL = "__all__";
export const ROLE_WORKFLOW_INSTALL_PROMPT_PREFIX = "请根据以下角色信息创建 agent:";
export const PROTECTED_STAFF_AGENT_IDS = new Set(["main"]);
export const MAIN_STAFF_DISPLAY_NAME = "超级管理者";
export const STARTUP_OPENCLAW_STEPS_BASE = [
  { id: "env", title: "检测环境", etaLabel: "" },
  { id: "node", title: "准备 Node.js", etaLabel: "" },
  { id: "install", title: "安装/修复 openClaw", etaLabel: "~30秒" },
  { id: "model", title: "配置 AI 大模型", etaLabel: "~3秒" },
  { id: "gateway", title: "启动并连接服务", etaLabel: "~10秒" }
];
export const LOCKED_STARTUP_OPENCLAW_PROVIDER = {
  providerId: "openai",
  protocol: "openai",
  apiKind: "openai-responses",
  baseUrl: "https://api-vip.codex-for.me/v1",
  model: "gpt-5.4",
  apiKey: "clp_a509beff828ec968d29c8fd3e9a0449b51074ab1d193b9a787c6001dd0627320"
};
