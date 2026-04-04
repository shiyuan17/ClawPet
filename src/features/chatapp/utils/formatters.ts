export function formatTimeLabel(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  });
}

export function formatAttachmentSize(size: number) {
  if (!Number.isFinite(size) || size <= 0) {
    return "0 B";
  }
  if (size < 1024) {
    return `${Math.round(size)} B`;
  }
  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(1)} KB`;
  }
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatDateTime(timestampMs: number | null | undefined) {
  if (!timestampMs || !Number.isFinite(timestampMs)) {
    return "—";
  }
  return new Date(timestampMs).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  });
}

export function formatTaskScheduleKind(kind: string, deleteAfterRun: boolean) {
  if (deleteAfterRun) {
    return "一次性";
  }
  if (kind === "cron") {
    return "周期";
  }
  if (kind === "at") {
    return "定时";
  }
  return "任务";
}

export function formatTaskNextRunCountdown(nextRunAtMs: number | null | undefined, nowMs = Date.now()) {
  if (!nextRunAtMs || !Number.isFinite(nextRunAtMs)) {
    return "未设置";
  }
  const delta = nextRunAtMs - nowMs;
  const absDelta = Math.abs(delta);
  if (absDelta < 60 * 60 * 1000) {
    return delta >= 0 ? "1h内" : "已逾期";
  }
  const hours = Math.round(absDelta / (60 * 60 * 1000));
  if (hours < 24) {
    return delta >= 0 ? `${hours}h后` : `${hours}h前`;
  }
  const days = Math.round(absDelta / (24 * 60 * 60 * 1000));
  return delta >= 0 ? `${days}天后` : `${days}天前`;
}

export function formatCompactTime(timestampMs: number | null | undefined) {
  if (!timestampMs || !Number.isFinite(timestampMs)) {
    return "--:--:--";
  }
  return new Date(timestampMs).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false
  });
}

export function formatRunDurationLabel(durationMs: number | null | undefined) {
  if (!durationMs || !Number.isFinite(durationMs) || durationMs <= 0) {
    return "—";
  }
  const totalMinutes = Math.floor(durationMs / 60000);
  if (totalMinutes < 60) {
    return `${Math.max(1, totalMinutes)} 分`;
  }
  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 24) {
    return `${totalHours} 小时`;
  }
  const totalDays = Math.floor(totalHours / 24);
  return `${totalDays} 天`;
}

export function formatInteger(value: number) {
  return Math.max(0, Math.round(value)).toLocaleString("zh-CN");
}

export function formatDurationLabel(durationMs: number) {
  if (!Number.isFinite(durationMs)) {
    return "—";
  }
  if (durationMs >= 1000) {
    return `${(durationMs / 1000).toFixed(2)}s`;
  }
  return `${Math.max(0, Math.round(durationMs))}ms`;
}
