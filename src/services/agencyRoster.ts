const agencyRosterReadmeEnModules = import.meta.glob("../agent-data/en/README*.md", {
  eager: true,
  query: "?raw",
  import: "default"
}) as Record<string, string>;
const agencyRosterReadmeZhModules = import.meta.glob("../agent-data/zh/README*.md", {
  eager: true,
  query: "?raw",
  import: "default"
}) as Record<string, string>;

function getReadmeCandidateScore(path: string, locale: "en" | "zh") {
  const normalizedPath = path.toLowerCase();
  if (normalizedPath.endsWith("/readme.md")) {
    return 0;
  }
  if (locale === "zh" && (normalizedPath.includes("readme.zh-tw") || normalizedPath.includes("zh_tw"))) {
    return 3;
  }
  if (normalizedPath.includes("/readme(")) {
    return 1;
  }
  return 2;
}

function resolveRosterReadmeRaw(modules: Record<string, string>, locale: "en" | "zh") {
  const candidates = Object.entries(modules).filter(([, raw]) => typeof raw === "string" && raw.trim().length > 0);
  if (candidates.length === 0) {
    return "";
  }

  return candidates
    .sort(([leftPath], [rightPath]) => {
      const leftScore = getReadmeCandidateScore(leftPath, locale);
      const rightScore = getReadmeCandidateScore(rightPath, locale);
      if (leftScore !== rightScore) {
        return leftScore - rightScore;
      }
      if (leftPath.length !== rightPath.length) {
        return leftPath.length - rightPath.length;
      }
      return leftPath.localeCompare(rightPath, "en");
    })[0][1];
}

const agencyRosterReadmeEnRaw = resolveRosterReadmeRaw(agencyRosterReadmeEnModules, "en");
const agencyRosterReadmeZhRaw = resolveRosterReadmeRaw(agencyRosterReadmeZhModules, "zh");

export type AgencyRosterRole = {
  id: string;
  nameEn: string;
  nameZh: string;
  workflowZh: string;
  sourcePath: string;
};

export type AgencyRosterGroup = {
  id: string;
  titleEn: string | null;
  titleZh: string | null;
  roles: AgencyRosterRole[];
};

export type AgencyRosterDivision = {
  id: string;
  titleEn: string;
  titleZh: string;
  groups: AgencyRosterGroup[];
  count: number;
};

type ParsedRole = {
  name: string;
  workflow: string;
  sourcePath: string;
};

type ParsedGroup = {
  title: string | null;
  roles: ParsedRole[];
  order: number;
};

type ParsedDivision = {
  title: string;
  groups: Map<string, ParsedGroup>;
  order: number;
};

type EnRoleMeta = {
  nameEn: string;
  divisionTitleEn: string;
  groupTitleEn: string | null;
};

const ROSTER_SECTION_RE = /(The Agency Roster|智能体阵容)/i;

function stripHeadingPrefix(raw: string) {
  return raw.replace(/^[^A-Za-z0-9\u4e00-\u9fa5]+\s*/u, "").trim();
}

function slugify(value: string) {
  const slug = value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  if (slug) {
    return slug;
  }
  let hash = 0;
  for (const char of value) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return `u-${hash.toString(36)}`;
}

function normalizeSourcePath(sourcePath: string) {
  const trimmed = sourcePath.trim().replace(/^\/+/, "");
  const withoutPrefix = trimmed.replace(/^\.?\/*agent-data\//i, "");
  return withoutPrefix.replace(/^(en|zh)\//i, "");
}

function parseMarkdownTableRow(line: string): string[] | null {
  const trimmed = line.trim();
  if (!trimmed.startsWith("|")) {
    return null;
  }

  const cells = trimmed
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());

  if (cells.length < 3) {
    return null;
  }

  const isSeparator = cells.every((cell) => /^:?-{2,}:?$/.test(cell));
  if (isSeparator) {
    return null;
  }

  return cells;
}

function extractAgentFromCell(cell: string): { name: string; sourcePath: string } | null {
  const match = cell.match(/\[([^\]]+)\]\(([^)]+\.md)\)/);
  if (!match) {
    return null;
  }
  return {
    name: match[1].trim(),
    sourcePath: normalizeSourcePath(match[2])
  };
}

function extractGroupTitleFromBoldLine(line: string): string | null {
  const match = line.trim().match(/^\*\*(.+?)\*\*[:：]?$/);
  if (!match) {
    return null;
  }
  return stripHeadingPrefix(match[1]);
}

function createDivisionId(title: string, index: number) {
  return `division-${slugify(title || `division-${index}`)}`;
}

function createGroupId(divisionId: string, title: string, index: number) {
  return `${divisionId}-group-${slugify(title || `group-${index}`)}`;
}

function parseRosterFromReadme(raw: string): ParsedDivision[] {
  const lines = raw.replace(/\r\n/g, "\n").split("\n");
  const divisions = new Map<string, ParsedDivision>();

  let inRoster = false;
  let currentDivisionTitle: string | null = null;
  let currentGroupTitle: string | null = null;
  let divisionOrder = 0;
  let groupOrder = 0;

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();
    const trimmed = line.trim();

    if (!inRoster) {
      if (/^##\s+/.test(trimmed) && ROSTER_SECTION_RE.test(trimmed)) {
        inRoster = true;
      }
      continue;
    }

    if (/^##\s+/.test(trimmed)) {
      break;
    }

    const level3 = trimmed.match(/^###\s+(.+)$/);
    if (level3) {
      currentDivisionTitle = stripHeadingPrefix(level3[1]) || level3[1].trim();
      currentGroupTitle = null;
      if (!divisions.has(currentDivisionTitle)) {
        divisions.set(currentDivisionTitle, {
          title: currentDivisionTitle,
          groups: new Map<string, ParsedGroup>(),
          order: divisionOrder
        });
        divisionOrder += 1;
      }
      continue;
    }

    const level4 = trimmed.match(/^####\s+(.+)$/);
    if (level4) {
      currentGroupTitle = stripHeadingPrefix(level4[1]) || level4[1].trim();
      continue;
    }

    const boldGroupTitle = extractGroupTitleFromBoldLine(trimmed);
    if (boldGroupTitle) {
      currentGroupTitle = boldGroupTitle;
      continue;
    }

    const cells = parseMarkdownTableRow(trimmed);
    if (!cells || !currentDivisionTitle) {
      continue;
    }

    const division = divisions.get(currentDivisionTitle);
    if (!division) {
      continue;
    }

    const agent = extractAgentFromCell(cells[0]);
    if (!agent) {
      continue;
    }

    const groupTitle = currentGroupTitle;
    const groupKey = groupTitle ?? "__default";
    if (!division.groups.has(groupKey)) {
      division.groups.set(groupKey, {
        title: groupTitle,
        roles: [],
        order: groupOrder
      });
      groupOrder += 1;
    }

    const group = division.groups.get(groupKey);
    if (!group) {
      continue;
    }

    group.roles.push({
      name: agent.name,
      workflow: cells[2]?.trim() ?? "",
      sourcePath: agent.sourcePath
    });
  }

  return Array.from(divisions.values())
    .sort((a, b) => a.order - b.order)
    .map((division) => ({
      ...division,
      groups: new Map(
        Array.from(division.groups.entries()).sort(([, left], [, right]) => left.order - right.order)
      )
    }));
}

function collectEnRoleMeta(divisions: ParsedDivision[]) {
  const metaByPath = new Map<string, EnRoleMeta>();
  for (const division of divisions) {
    for (const group of division.groups.values()) {
      for (const role of group.roles) {
        metaByPath.set(role.sourcePath, {
          nameEn: role.name,
          divisionTitleEn: division.title,
          groupTitleEn: group.title
        });
      }
    }
  }
  return metaByPath;
}

function increment(counter: Map<string, number>, key: string | null) {
  if (!key) {
    return;
  }
  counter.set(key, (counter.get(key) ?? 0) + 1);
}

function pickDominantTitle(counter: Map<string, number>, fallback: string) {
  if (counter.size === 0) {
    return fallback;
  }
  return Array.from(counter.entries())
    .sort((left, right) => {
      if (right[1] !== left[1]) {
        return right[1] - left[1];
      }
      return left[0].localeCompare(right[0], "en");
    })[0][0];
}

function parseAgencyRoster(): AgencyRosterDivision[] {
  const zhDivisions = parseRosterFromReadme(agencyRosterReadmeZhRaw);
  const enRoleMeta = collectEnRoleMeta(parseRosterFromReadme(agencyRosterReadmeEnRaw));

  let roleOrder = 0;
  return zhDivisions.map<AgencyRosterDivision>((division, divisionIndex) => {
    const divisionTitleCounter = new Map<string, number>();
    const groups = Array.from(division.groups.values()).map<AgencyRosterGroup>((group, groupIndex) => {
      const groupTitleCounter = new Map<string, number>();
      const roles = group.roles.map<AgencyRosterRole>((role) => {
        const enMeta = enRoleMeta.get(role.sourcePath);
        increment(divisionTitleCounter, enMeta?.divisionTitleEn ?? null);
        increment(groupTitleCounter, enMeta?.groupTitleEn ?? null);

        const roleId = `role-${roleOrder}`;
        roleOrder += 1;
        return {
          id: roleId,
          nameEn: enMeta?.nameEn ?? role.name,
          nameZh: role.name,
          workflowZh: role.workflow || `${division.title}相关任务`,
          sourcePath: role.sourcePath
        };
      });

      const titleEn = pickDominantTitle(groupTitleCounter, group.title ?? "");
      const titleZh = group.title;
      return {
        id: "",
        titleEn: titleEn || null,
        titleZh,
        roles
      };
    });

    const divisionTitleEn = pickDominantTitle(divisionTitleCounter, division.title);
    const divisionId = createDivisionId(divisionTitleEn, divisionIndex);
    const hydratedGroups = groups.map((group, groupIndex) => ({
      ...group,
      id: createGroupId(divisionId, group.titleEn ?? group.titleZh ?? "", groupIndex)
    }));
    const count = hydratedGroups.reduce((sum, group) => sum + group.roles.length, 0);

    return {
      id: divisionId,
      titleEn: divisionTitleEn,
      titleZh: division.title,
      groups: hydratedGroups,
      count
    };
  });
}

const AGENCY_ROSTER_CACHE = parseAgencyRoster();

export function loadAgencyRosterZh() {
  return AGENCY_ROSTER_CACHE;
}
