import { CHAT_MARKDOWN_RENDER_CACHE_MAX } from "../constants";

const chatMarkdownRenderCache = new Map<string, string>();

function setLimitedCacheEntry<T>(cache: Map<string, T>, key: string, value: T, maxEntries: number) {
  cache.set(key, value);
  if (cache.size <= maxEntries) {
    return value;
  }
  const firstKey = cache.keys().next().value;
  if (typeof firstKey === "string") {
    cache.delete(firstKey);
  }
  return value;
}

export function escapeHtml(raw: string) {
  return raw
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeHtmlAttribute(raw: string) {
  return escapeHtml(raw).replace(/`/g, "&#96;");
}

function encodeTextForHtmlDataAttribute(raw: string) {
  return escapeHtmlAttribute(encodeURIComponent(raw));
}

function sanitizeMarkdownHref(rawHref: string) {
  const candidate = rawHref.trim().replace(/&amp;/g, "&");
  if (!candidate) {
    return null;
  }
  if (
    candidate.startsWith("http://") ||
    candidate.startsWith("https://") ||
    candidate.startsWith("mailto:") ||
    candidate.startsWith("file://")
  ) {
    return candidate;
  }
  return null;
}

function renderInlineMarkdown(raw: string) {
  const codeTokens: string[] = [];
  const withTokens = raw.replace(/`([^`]+?)`/g, (_match: string, codeText: string) => {
    const token = `@@CODE_${codeTokens.length}@@`;
    const encodedCodeText = encodeTextForHtmlDataAttribute(codeText);
    codeTokens.push(
      `<span class="message-md-inline-code"><code>${escapeHtml(codeText)}</code><button type="button" class="message-md-copy message-md-inline-code__copy" data-copy-code="${encodedCodeText}">复制</button></span>`
    );
    return token;
  });
  let html = escapeHtml(withTokens);
  html = html.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_match: string, label: string, href: string) => {
    const normalizedHref = sanitizeMarkdownHref(href);
    if (!normalizedHref) {
      return label;
    }
    return `<a href="${escapeHtmlAttribute(normalizedHref)}" target="_blank" rel="noopener noreferrer">${label}</a>`;
  });
  html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, "$1<em>$2</em>");
  html = html.replace(/~~([^~]+)~~/g, "<del>$1</del>");
  for (let index = 0; index < codeTokens.length; index += 1) {
    html = html.replace(`@@CODE_${index}@@`, codeTokens[index]);
  }
  return html;
}

function sanitizeMarkdownLanguage(raw: string) {
  return raw.trim().replace(/[^a-z0-9_-]+/gi, "").slice(0, 24);
}

export function renderMarkdownToHtml(markdown: string) {
  const normalized = markdown.replace(/\r\n/g, "\n").replace(/\r/g, "\n").trim();
  if (!normalized) {
    return "";
  }

  const cached = chatMarkdownRenderCache.get(normalized);
  if (typeof cached === "string") {
    return cached;
  }

  const lines = normalized.split("\n");
  const blocks: string[] = [];
  const paragraphLines: string[] = [];
  const quoteLines: string[] = [];
  let listType: "ul" | "ol" | null = null;
  let listItems: string[] = [];
  let inCodeBlock = false;
  let codeLanguage = "";
  let codeLines: string[] = [];

  const flushParagraph = () => {
    if (paragraphLines.length === 0) {
      return;
    }
    const text = paragraphLines.map((line) => renderInlineMarkdown(line.trim())).join("<br />");
    blocks.push(`<p>${text}</p>`);
    paragraphLines.length = 0;
  };

  const flushQuote = () => {
    if (quoteLines.length === 0) {
      return;
    }
    const content = quoteLines.map((line) => renderInlineMarkdown(line)).join("<br />");
    blocks.push(`<blockquote><p>${content}</p></blockquote>`);
    quoteLines.length = 0;
  };

  const flushList = () => {
    if (!listType || listItems.length === 0) {
      listType = null;
      listItems = [];
      return;
    }
    blocks.push(`<${listType}>${listItems.map((item) => `<li>${item}</li>`).join("")}</${listType}>`);
    listType = null;
    listItems = [];
  };

  const flushCodeBlock = () => {
    if (!inCodeBlock) {
      return;
    }
    const languageClass = codeLanguage ? ` class="language-${escapeHtmlAttribute(codeLanguage)}"` : "";
    const codeText = codeLines.join("\n");
    const encodedCodeText = encodeTextForHtmlDataAttribute(codeText);
    blocks.push(
      `<div class="message-md-codeblock"><button type="button" class="message-md-copy message-md-codeblock__copy" data-copy-code="${encodedCodeText}">复制</button><pre><code${languageClass}>${escapeHtml(codeText)}</code></pre></div>`
    );
    inCodeBlock = false;
    codeLanguage = "";
    codeLines = [];
  };

  const flushTextBlocks = () => {
    flushParagraph();
    flushQuote();
    flushList();
  };

  for (const line of lines) {
    const trimmed = line.trim();

    if (inCodeBlock) {
      if (/^```/.test(trimmed)) {
        flushCodeBlock();
      } else {
        codeLines.push(line);
      }
      continue;
    }

    const fenceMatch = trimmed.match(/^```([a-z0-9_-]+)?\s*$/i);
    if (fenceMatch) {
      flushTextBlocks();
      inCodeBlock = true;
      codeLanguage = sanitizeMarkdownLanguage(fenceMatch[1] ?? "");
      codeLines = [];
      continue;
    }

    if (!trimmed) {
      flushTextBlocks();
      continue;
    }

    const headingMatch = trimmed.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      flushTextBlocks();
      const level = headingMatch[1].length;
      blocks.push(`<h${level}>${renderInlineMarkdown(headingMatch[2].trim())}</h${level}>`);
      continue;
    }

    if (/^(?:-{3,}|\*{3,}|_{3,})$/.test(trimmed)) {
      flushTextBlocks();
      blocks.push("<hr />");
      continue;
    }

    const quoteMatch = line.match(/^\s*>\s?(.*)$/);
    if (quoteMatch) {
      flushParagraph();
      flushList();
      quoteLines.push(quoteMatch[1].trim());
      continue;
    }
    flushQuote();

    const orderedMatch = line.match(/^\s*\d+[.)]\s+(.+)$/);
    if (orderedMatch) {
      flushParagraph();
      if (listType !== "ol") {
        flushList();
        listType = "ol";
      }
      listItems.push(renderInlineMarkdown(orderedMatch[1].trim()));
      continue;
    }

    const unorderedMatch = line.match(/^\s*[-*+]\s+(.+)$/);
    if (unorderedMatch) {
      flushParagraph();
      if (listType !== "ul") {
        flushList();
        listType = "ul";
      }
      listItems.push(renderInlineMarkdown(unorderedMatch[1].trim()));
      continue;
    }

    flushList();
    paragraphLines.push(line);
  }

  flushTextBlocks();
  flushCodeBlock();
  const rendered = blocks.join("");
  return setLimitedCacheEntry(chatMarkdownRenderCache, normalized, rendered, CHAT_MARKDOWN_RENDER_CACHE_MAX);
}

function stripLeadingUntrustedMetadataEnvelope(text: string) {
  const normalized = text.replace(/\r\n/g, "\n");
  const hasMetadataMarker = /\(untrusted metadata\):/i.test(normalized);
  const hasCodeBlock = /```(?:json)?/i.test(normalized);
  const likelyEnvelopeStart = /^System:\s*\[/.test(normalized) || /^[^\n]*\(untrusted metadata\):/i.test(normalized);
  if (!hasMetadataMarker || !hasCodeBlock || !likelyEnvelopeStart) {
    return text;
  }

  let rest = normalized;
  rest = rest.replace(/^System:[\s\S]*?\n{2}/, "");

  const metadataSectionPattern = /^[^\n]*\(untrusted metadata\):\s*\n```(?:json)?\n[\s\S]*?\n```(?:\n{1,2})?/i;
  let previous = "";
  while (rest !== previous) {
    previous = rest;
    rest = rest.replace(metadataSectionPattern, "");
  }

  return rest.trimStart();
}

function stripLeadingMessageIdLabel(text: string) {
  const normalized = text.replace(/\r\n/g, "\n");
  const markerPattern = /^\s*\[(?:message|msg)[_\s-]?id\s*[:：]\s*[^\]\n]+\]\s*(?:\n+)?/i;
  let rest = normalized;
  let previous = "";
  while (rest !== previous) {
    previous = rest;
    rest = rest.replace(markerPattern, "");
  }
  return rest.trimStart();
}

function stripLeadingChannelSenderIdLabel(text: string) {
  const normalized = text.replace(/\r\n/g, "\n");
  const labeledSenderPattern =
    /^\s*(?:feishu:(?:direct|group):)?(?:user:|chat:)?((?:ou|oc)_[a-z0-9]{8,})\s*(?:[:：-]\s*|\s+)([\s\S]+)$/i;
  const labeledSenderMatch = normalized.match(labeledSenderPattern);
  if (labeledSenderMatch?.[2]?.trim()) {
    return labeledSenderMatch[2].trimStart();
  }

  const compactSenderPattern = /^\s*((?:ou|oc)_[a-z0-9]{16,})(?=[\u4e00-\u9fff])/i;
  const compactSenderMatch = normalized.match(compactSenderPattern);
  if (compactSenderMatch?.[1]) {
    const rest = normalized.slice(compactSenderMatch[0].length).trimStart();
    if (rest) {
      return rest;
    }
  }

  return text;
}

function extractLatestUserLineFromPromptEnvelope(text: string) {
  const normalized = text.replace(/\r\n/g, "\n");
  const markerPattern = /\[Current message - respond to this\]/i;
  const markerMatch = markerPattern.exec(normalized);
  const scope = markerMatch ? normalized.slice(markerMatch.index + markerMatch[0].length) : normalized;

  const userLinePattern = /^(?:User|用户)\s*[:：]\s*(.+)$/gim;
  let lastContent = "";
  let matched: RegExpExecArray | null = null;
  while ((matched = userLinePattern.exec(scope)) !== null) {
    const content = (matched[1] ?? "").trim();
    if (content) {
      lastContent = content;
    }
  }
  return lastContent;
}

function stripChatContextEnvelope(text: string) {
  const normalized = text.replace(/\r\n/g, "\n");
  const hasContextHeader = /^\s*\[Chat messages since your last reply - for context\]/i.test(normalized);
  const hasCurrentMarker = /\[Current message - respond to this\]/i.test(normalized);
  const hasConversationTurns = /(?:^|\n)(?:User|用户)\s*[:：]/i.test(normalized) && /(?:^|\n)(?:Assistant|助手)\s*[:：]/i.test(normalized);
  if (!hasContextHeader && !hasCurrentMarker && !hasConversationTurns) {
    return text;
  }

  const extractedCurrentMessage = extractLatestUserLineFromPromptEnvelope(normalized);
  if (extractedCurrentMessage) {
    return extractedCurrentMessage;
  }
  return text;
}

export function sanitizeMessageTextForDisplay(rawText: string) {
  return stripChatContextEnvelope(
    stripLeadingChannelSenderIdLabel(stripLeadingMessageIdLabel(stripLeadingUntrustedMetadataEnvelope(rawText)))
  ).trim();
}
