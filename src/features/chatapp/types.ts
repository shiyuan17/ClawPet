export type SidebarSection = "chat" | "dashboard" | "recruitment" | "skills" | "tasks" | "market";
export type AgentGroupKind = "staff" | "group";
export type AgentStatusTone = "online" | "busy" | "offline";
export type ChatRole = "assistant" | "user" | "system";
export type ChatStatus = "pending" | "done" | "error";
export type ChatMessageKind = "default" | "runtime_tool";
export type ChatToolStatus = "running" | "done" | "error";
export type AgentPaneTab = "staff" | "group" | "channel";
export type ChatQuickCreateActionId = "role" | "group" | "friend";
export type RelatedResourceTarget = "memory" | "skills" | "tools" | "model" | "channel" | "schedule";
export type RelatedSkillCategory = "builtIn" | "installed";
export type RelatedToolsFilter = string;
export type RelatedScheduleFilter = "all" | "enabled" | "stopped" | "disabled";
export type UtilityModalType = "history" | "logs";
export type ChatSettingsPanelMode = "agent" | "group" | "logs" | "history";
export type UtilityLogTab = "runtime" | "errorAnalysis";
export type UtilityLogDetailTab = "request" | "response" | "stream" | "raw";
export type UtilityLogCategory = "all" | "message" | "tool";
export type UtilityLogScheduleFilter = "all" | "scheduled" | "manual";
export type SidebarSettingsMenuGroupId = "general" | "providers" | "about";
export type SidebarSettingsAppearance = "system" | "light" | "dark";
export type SidebarSettingsLanguage = "zh-CN" | "en-US";
export type SidebarThemePreset = "elegant" | "frosted" | "pure-white";
export type SidebarThemeMode = "day" | "night";
export type SidebarAvatarCategoryId = "pixel" | "illustration" | "animal" | "cute";
export type SidebarSettingsMenuGroup = {
  id: SidebarSettingsMenuGroupId;
  label: string;
};
export type SidebarAvatarOption = {
  id: string;
  label: string;
  category: SidebarAvatarCategoryId;
  url: string;
};
export type OpenClawProviderProtocol = "openai" | "anthropic";
export type OpenClawProviderApiKind = "openai-completions" | "openai-responses" | "anthropic-messages";
export type TaskBoardStatus = "todo" | "in_progress" | "in_review" | "done" | "cancelled";
export type TaskBoardGroupMode = "status" | "agent" | "team";
export type SidebarItem = {
  id: SidebarSection;
  label: string;
};
export type CreateEmployeeModalStep = 1 | 2 | 3;
export type CreateEmployeeIdentityOption = {
  id: string;
  icon: string;
  name: string;
  desc: string;
};
export type CreateEmployeeRuleItem = {
  id: string;
  text: string;
  enabled: boolean;
  custom?: boolean;
};
export type ChannelPaneConfigField = {
  key: string;
  label: string;
  placeholder: string;
  required?: boolean;
  secret?: boolean;
  description?: string;
};
export type ChannelPaneConnectionMode = "form" | "qr";
export type ChannelPaneCatalogItem = {
  id: string;
  name: string;
  description: string;
  icon: string;
  connectionMode?: ChannelPaneConnectionMode;
  aliases?: string[];
  backendChannelType?: string;
  docsUrl?: string;
  instructions?: string[];
  fields?: ChannelPaneConfigField[];
};
