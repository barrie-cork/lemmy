/**
 * Lemmy/Brehon project hooks for pi-coding.
 *
 * Dual-harness profile: preserve Claude Code/Junior assets, but expose their
 * useful guardrails to pi.
 *
 * Provides:
 * - system-prompt context from .pi/PROJECT_CONTEXT.md
 * - .claude/rules index injection
 * - pre-phase audit reminder from the existing Claude hook
 * - PMD HTTP reachability guard for Recursive Learning System writes
 * - DQ/task-hopper coordination-state injection
 * - destructive bash firewall
 * - Junior worktree guard via copied .pi/hook-scripts/worktree-guard.sh
 * - observation shadow telemetry via copied .pi/hook-scripts/observation-capture.sh
 * - lesson frontmatter + PMD sync hooks for Recursive Learning System parity
 * - warn-only retro nudge on shutdown
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import * as fs from "node:fs";
import * as path from "node:path";
import { spawnSync } from "node:child_process";

const REPO_ROOT = path.resolve(__dirname, "..", "..");
const PI_HOOKS_DIR = path.join(REPO_ROOT, ".pi", "hook-scripts");

const CONTEXT_HEADINGS = {
  constraints: "Non-negotiable Brehon constraints",
  workflow: "Coding workflow constraints",
  harness: "Dual-harness boundary",
  rls: "Recursive Learning System parity for pi",
  rust: "Pi session Rust quick-reference",
  subagents: "Project subagents \u2014 delegate, don\u2019t load",
  setup: "Setup decisions log (do not re-litigate)",
} as const;

interface ModeContext {
  persona: string;
  projectContextHeadings: string[];
  ruleFilter: string[];
  recommendedSkill: string | null;
  authorizedReadPaths: string[];
}

const MODE_CONTEXT: Record<BrehonMode, ModeContext> = {
  "main-safe": {
    persona: "",
    projectContextHeadings: [],
    ruleFilter: [],
    recommendedSkill: null,
    authorizedReadPaths: [],
  },
  planning: {
    persona: "You are the Planning agent for Brehon. Author plan files from briefs; never write Rust code. Use rg/find/read for codebase exploration; use read for crate docs.",
    projectContextHeadings: [
      CONTEXT_HEADINGS.constraints,
      CONTEXT_HEADINGS.workflow,
      CONTEXT_HEADINGS.harness,
    ],
    ruleFilter: [
      "handover.md",
      "pmd-invariants.md",
      "pmd-search-strategy.md",
      "multi-lane-worktree.md",
    ],
    recommendedSkill: ".pi/skills/planning/SKILL.md",
    authorizedReadPaths: [
      ".claude/commands/prp-core/prp-plan.md",
      ".claude/lessons/",
      ".claude/PRPs/briefs/",
      ".claude/PRPs/plans/",
      ".claude/PRPs/reports/",
      ".claude/PRPs/templates/",
    ],
  },
  "impl-task": {
    persona: "You are the Implementation agent for Brehon. Execute one scoped task from an approved plan. Use MIRROR refs as patterns; fall back to a DQ entry instead of guessing.",
    projectContextHeadings: [
      CONTEXT_HEADINGS.constraints,
      CONTEXT_HEADINGS.workflow,
      CONTEXT_HEADINGS.harness,
      CONTEXT_HEADINGS.rust,
    ],
    ruleFilter: [
      "phase-branch.md",
      "no-cargo-output-paste.md",
      "cargo-output-capture.md",
      "view-crate-selectable-template.md",
      "pmd-invariants.md",
    ],
    recommendedSkill: ".pi/skills/impl-task/SKILL.md",
    authorizedReadPaths: [
      ".claude/commands/prp-core/prp-implement.md",
      ".claude/lessons/",
      ".claude/PRPs/plans/",
      ".claude/decision-queue.json",
    ],
  },
  "review-readonly": {
    persona: "You are in read-only review mode. Report findings with file paths and recommendations; do not mutate files.",
    projectContextHeadings: [
      CONTEXT_HEADINGS.constraints,
      CONTEXT_HEADINGS.workflow,
      CONTEXT_HEADINGS.harness,
      CONTEXT_HEADINGS.rust,
    ],
    ruleFilter: [
      "pi-harness-constraints.md",
      "no-destructive-defaults.md",
      "multi-lane-worktree.md",
      "pmd-invariants.md",
    ],
    recommendedSkill: null,
    authorizedReadPaths: [
      ".claude/rules/",
    ],
  },
  bm: {
    persona: "You are the Branch Manager agent for Brehon. Manage git/PR lifecycle; prefer delegating to .pi/agents/bm-pi.md; never write code or plans.",
    projectContextHeadings: [
      CONTEXT_HEADINGS.constraints,
      CONTEXT_HEADINGS.workflow,
      CONTEXT_HEADINGS.harness,
      CONTEXT_HEADINGS.subagents,
      CONTEXT_HEADINGS.setup,
    ],
    ruleFilter: [
      "branch-manager.md",
      "gh-pr-fork-target.md",
      "decision-queue.md",
      "phase-branch.md",
      "pmd-invariants.md",
      "universal-guards.md",
    ],
    recommendedSkill: ".pi/skills/bm-task/SKILL.md",
    authorizedReadPaths: [
      ".claude/rules/branch-manager.md",
      ".claude/rules/decision-queue.md",
      ".claude/rules/phase-branch.md",
      ".claude/rules/gh-pr-fork-target.md",
      ".claude/commands/bm/",
      ".claude/runlog/",
      ".claude/PRPs/reviews/",
    ],
  },
  "ci-debug": {
    persona: "You are the CI Debug agent for Brehon. Diagnose GitHub Actions workflows; prefer delegating to .pi/agents/ci-debug.md.",
    projectContextHeadings: [
      CONTEXT_HEADINGS.constraints,
      CONTEXT_HEADINGS.workflow,
      CONTEXT_HEADINGS.harness,
      CONTEXT_HEADINGS.subagents,
      CONTEXT_HEADINGS.setup,
    ],
    ruleFilter: [
      "phase-branch.md",
      "pmd-invariants.md",
    ],
    recommendedSkill: null,
    authorizedReadPaths: [
      ".claude/lessons/feedback_gha_pi_loop_postmortem.md",
      ".claude/rules/",
    ],
  },
  "harness-maintenance": {
    persona: "You are in harness maintenance mode. Edit harness metadata (.claude/skills/, .claude/lessons/, .pi/) only; no app code.",
    projectContextHeadings: [
      CONTEXT_HEADINGS.constraints,
      CONTEXT_HEADINGS.workflow,
      CONTEXT_HEADINGS.harness,
      CONTEXT_HEADINGS.rls,
      CONTEXT_HEADINGS.subagents,
      CONTEXT_HEADINGS.setup,
    ],
    ruleFilter: [
      "pi-harness-constraints.md",
      "pmd-invariants.md",
      "pre-phase-harness-audit.md",
    ],
    recommendedSkill: null,
    authorizedReadPaths: [
      ".claude/skills/",
      ".claude/lessons/",
      ".claude/PRPs/reports/",
    ],
  },
};

const BASH_BLOCKLIST = [
  "rm -rf",
  "rm -fr",
  "git reset --hard",
  "git push --force",
  "git push -f",
  "git clean -fd",
  "git clean -f",
  "chmod 777",
  "mkfs",
  "dd if=",
  "> /dev/sd",
];

const EDIT_READBACK_THRESHOLD = 5;

type BrehonMode = "main-safe" | "planning" | "impl-task" | "review-readonly" | "bm" | "ci-debug" | "harness-maintenance";
type PathPolicyDecision = { block: true; reason: string } | undefined;

const BREHON_MODES: Record<BrehonMode, { label: string; description: string; instructions: string[] }> = {
  "main-safe": {
    label: "BREHON:SAFE",
    description: "Default Pi mode: normal repo work with Brehon hard constraints and focused edits.",
    instructions: [
      "Use this as the default mixed implementation/review mode.",
      "Rust edits still require a plan file under .claude/PRPs/plans/.",
      "Do not modify .claude/ ownership areas unless the user explicitly asks.",
    ],
  },
  planning: {
    label: "BREHON:PLAN",
    description: "Planning/docs mode: write plan/spec/doc artifacts only; never implementation code.",
    instructions: [
      "Read Brehon design docs, ADRs, PRDs, and prior reports before changing plans/specs.",
      "Allowed writes are docs/.pi planning artifacts and .claude/PRPs plan/report/review/brief artifacts.",
      "Do not edit Rust, migrations, source crates, or implementation code in this mode.",
    ],
  },
  "impl-task": {
    label: "BREHON:IMPL",
    description: "Implementation mode: execute a scoped task from an existing Brehon plan.",
    instructions: [
      "Before Rust edits, verify a relevant plan exists under .claude/PRPs/plans/.",
      "Use scripts/brehon/cargo-*.sh wrappers with scope flags; redirect cargo output to .pi/*.log.",
      "Keep edits scoped to the named task; do not write plans or Branch Manager artifacts.",
    ],
  },
  "review-readonly": {
    label: "BREHON:REVIEW",
    description: "Read-only review/audit mode: inspect and report findings, do not mutate files.",
    instructions: [
      "Do not use write/edit or mutate the worktree.",
      "Report high-confidence findings with exact file paths and actionable recommendations.",
      "Prefer read and narrowly-scoped bash inspection commands.",
    ],
  },
  bm: {
    label: "BREHON:BM",
    description: "Branch Manager mode: git/PR lifecycle only; no code or plan authoring.",
    instructions: [
      "Prefer delegating BM verbs to the .pi/agents/bm-pi.md project agent when available.",
      "Do not author implementation code or write Brehon plans.",
      "Ask before outbound-visible actions such as PR comments, pushes, merges, or pings.",
    ],
  },
  "ci-debug": {
    label: "BREHON:CI",
    description: "CI debug mode: workflow/script diagnostics without changing the repo-wide manual commit workflow.",
    instructions: [
      "Prefer delegating GitHub Actions work to the .pi/agents/ci-debug.md project agent when available.",
      "Keep edits focused on .github/workflows/, .github/scripts/, or explicit CI docs/artifacts.",
      "Follow the repo-wide manual git add/commit workflow when the fix is real.",
    ],
  },
  "harness-maintenance": {
    label: "BREHON:HARNESS",
    description: "Harness/RLS metadata mode: skill, lesson, retro, and pi harness files only; no app code.",
    instructions: [
      "Use only for explicit harness-maintenance or Recursive Learning System artifact work.",
      "Allowed writes are .claude/skills/, .claude/lessons/, .claude/PRPs/reports/, .pi/skills/, .pi/scripts/, .pi/extensions/.",
      "Validate skill metadata with python3 .pi/scripts/validate-skills.py after skill edits; lesson writes trigger PMD sync hooks.",
    ],
  },
};

const PLANNING_WRITE_PREFIXES = [
  "docs/",
  ".pi/",
  ".claude/PRPs/plans/",
  ".claude/PRPs/briefs/",
  ".claude/PRPs/reports/",
  ".claude/PRPs/reviews/",
  ".claude/PRPs/PRDs/",
];

const CI_DEBUG_WRITE_PREFIXES = [".github/workflows/", ".github/scripts/", ".pi/", "docs/"];
const HARNESS_MAINTENANCE_WRITE_PREFIXES = [".claude/skills/", ".claude/lessons/", ".claude/PRPs/reports/", ".pi/skills/", ".pi/scripts/", ".pi/extensions/", ".pi/PROJECT_CONTEXT.md", "AGENTS.md"];
const CODE_PREFIXES = ["crates/", "src/", "migrations/", "diesel_migrations/"];
const SOURCE_EXTENSIONS = new Set([".rs", ".ts", ".tsx", ".js", ".jsx", ".sql", ".toml", ".yml", ".yaml"]);

type NotifyLevel = "info" | "warning" | "error";

function safeNotify(ctx: any, msg: string, level: NotifyLevel = "info") {
  try {
    ctx?.ui?.notify?.(msg, level);
  } catch {
    console.error(msg);
  }
}

function findMarkdownFiles(dir: string, basePath = ""): string[] {
  if (!fs.existsSync(dir)) return [];
  const results: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const rel = basePath ? `${basePath}/${entry.name}` : entry.name;
    const abs = path.join(dir, entry.name);
    if (entry.isDirectory()) results.push(...findMarkdownFiles(abs, rel));
    else if (entry.isFile() && entry.name.endsWith(".md")) results.push(rel);
  }
  return results.sort();
}

function readIfExists(filePath: string): string {
  try {
    return fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf8").trim() : "";
  } catch {
    return "";
  }
}

function repoRelativePath(filePath: string): string {
  const absolute = path.isAbsolute(filePath) ? filePath : path.resolve(REPO_ROOT, filePath);
  return path.relative(REPO_ROOT, absolute).replaceAll(path.sep, "/");
}

function hasPrefix(relPath: string, prefixes: string[]): boolean {
  return prefixes.some((prefix) => relPath === prefix.slice(0, -1) || relPath.startsWith(prefix));
}

function isSourceLikePath(relPath: string): boolean {
  return hasPrefix(relPath, CODE_PREFIXES) || SOURCE_EXTENSIONS.has(path.extname(relPath));
}

function planFileExists(): boolean {
  const planDir = path.join(REPO_ROOT, ".claude", "PRPs", "plans");
  try {
    return fs.existsSync(planDir) && fs.readdirSync(planDir).some((file) => file.endsWith(".md"));
  } catch {
    return false;
  }
}

function matchedBlockedPattern(command: string): string | undefined {
  const normalized = command.replace(/\s+/g, " ").trim();
  return BASH_BLOCKLIST.find((pattern) => normalized.includes(pattern));
}

function sensitiveShellPattern(command: string): string | undefined {
  const normalized = command.replace(/\s+/g, " ").trim();
  const sensitivePath = String.raw`(?:^|[\s'\"])(?:\.env(?:\.[^\s'\"]*)?|[^\s'\"]*(?:id_rsa|id_ed25519|credentials|secret|\.pem|\.key))`;
  const readers = String.raw`\b(?:cat|less|more|head|tail|sed|awk|grep|rg|find)\b`;
  return new RegExp(`${readers}[^;&|]*${sensitivePath}`, "i").test(normalized) ? "shell reads secret-like path" : undefined;
}

function rawCargoPattern(command: string): string | undefined {
  const normalized = command.replace(/\s+/g, " ").trim();
  if (/scripts\/brehon\/cargo-(check|clippy|test|nextest)\.sh\b/.test(normalized)) return undefined;
  if (/\bcargo\s+(check|clippy|test|nextest|build)\b/.test(normalized)) return "raw cargo invocation; use scripts/brehon/cargo-*.sh wrappers and redirect output to .pi/*.log";
  return undefined;
}

function pathPolicyDecision(mode: BrehonMode, toolName: string, filePath: unknown): PathPolicyDecision {
  if (toolName !== "edit" && toolName !== "write") return undefined;
  if (typeof filePath !== "string" || filePath.length === 0) return undefined;

  const relPath = repoRelativePath(filePath);
  if (relPath.startsWith("..")) return { block: true, reason: `Brehon path policy blocks writes outside repo: ${filePath}` };

  // Check manual override before mode-specific rules (per session-retro-2026-06-10 Change #3)
  if (overridePaths.has(relPath)) {
    overridePaths.delete(relPath); // one-time use
    return undefined;
  }

  if (mode === "review-readonly") return { block: true, reason: "Brehon review-readonly mode blocks write/edit" };

  if (mode === "planning") {
    if (!hasPrefix(relPath, PLANNING_WRITE_PREFIXES)) return { block: true, reason: `Brehon planning mode writes only planning/doc artifacts, not ${relPath}` };
    if (isSourceLikePath(relPath)) return { block: true, reason: `Brehon planning mode blocks source/code writes: ${relPath}` };
  }

  if (mode === "ci-debug" && !hasPrefix(relPath, CI_DEBUG_WRITE_PREFIXES)) {
    return { block: true, reason: `Brehon ci-debug mode writes only CI/debug artifacts, not ${relPath}` };
  }

  if (mode === "harness-maintenance" && !hasPrefix(relPath, HARNESS_MAINTENANCE_WRITE_PREFIXES)) {
    return { block: true, reason: `Brehon harness-maintenance mode writes only harness metadata/tooling, not ${relPath}` };
  }

  if (mode === "bm") {
    if (isSourceLikePath(relPath)) return { block: true, reason: `Brehon BM mode blocks source/code writes: ${relPath}` };
    if (relPath.startsWith(".claude/PRPs/plans/")) return { block: true, reason: `Brehon BM mode must not write plans: ${relPath}` };
  }

  if (relPath.endsWith(".rs") && !planFileExists()) {
    return { block: true, reason: "Brehon hard rule: Rust edits require a plan file under .claude/PRPs/plans/" };
  }

  if (relPath.startsWith(".claude/") && mode !== "planning" && mode !== "harness-maintenance") {
    return { block: true, reason: `Brehon dual-harness boundary blocks .claude writes in ${mode} mode. Switch to planning/harness-maintenance or use /brehon-override ${relPath}` };
  }

  return undefined;
}

async function confirmDangerousBash(command: string, ctx: any): Promise<{ block: true; reason: string } | undefined> {
  const pattern = matchedBlockedPattern(command);
  if (!pattern) return undefined;
  if (!ctx.hasUI) return { block: true, reason: `Blocked by Lemmy bash firewall (no UI): ${pattern}` };
  const choice = await ctx.ui.select(
    `⚠️ Bash firewall: command contains "${pattern}":\n\n  ${command}\n\nAllow?`,
    ["No (block)", "Yes (allow once)"],
  );
  if (choice !== "Yes (allow once)") return { block: true, reason: `Blocked by Lemmy bash firewall: ${pattern}` };
  return undefined;
}

function piToolToClaudeTool(toolName: string): string {
  const map: Record<string, string> = {
    bash: "Bash",
    edit: "Edit",
    write: "Write",
    read: "Read",
  };
  return map[toolName] ?? toolName;
}

function claudeCompatibleInput(event: { toolName: string; input: unknown }, cwd: string) {
  const input = (event.input ?? {}) as Record<string, unknown>;
  const toolInput: Record<string, unknown> = { ...input };

  // Claude hooks historically use file_path; pi tools use path.
  if (typeof input.path === "string" && typeof toolInput.file_path !== "string") {
    toolInput.file_path = input.path;
  }

  return {
    cwd,
    tool_name: piToolToClaudeTool(event.toolName),
    tool_input: toolInput,
  };
}

function runHookScript(scriptName: string, payload?: unknown, timeout = 5_000) {
  const scriptPath = path.join(PI_HOOKS_DIR, scriptName);
  if (!fs.existsSync(scriptPath)) return { status: 0, stdout: "", stderr: "" };

  const res = spawnSync("bash", [scriptPath], {
    cwd: REPO_ROOT,
    input: payload === undefined ? undefined : JSON.stringify(payload),
    encoding: "utf8",
    timeout,
  });

  return {
    status: res.status ?? 0,
    stdout: res.stdout ?? "",
    stderr: res.stderr ?? "",
  };
}

function parseHookJson(stdout: string): any | undefined {
  const trimmed = stdout.trim();
  if (!trimmed.startsWith("{")) return undefined;
  try {
    return JSON.parse(trimmed);
  } catch {
    return undefined;
  }
}

function extractAdditionalContext(stdout: string): string {
  const parsed = parseHookJson(stdout);
  return parsed?.hookSpecificOutput?.additionalContext ?? "";
}

function worktreeGuardDecision(stdout: string): { block: true; reason: string } | undefined {
  const parsed = parseHookJson(stdout);
  const out = parsed?.hookSpecificOutput;
  if (out?.permissionDecision === "deny") {
    return { block: true, reason: out.permissionDecisionReason || "Blocked by worktree-guard" };
  }
  return undefined;
}

function coordinationStateSummary(): string {
  const parts: string[] = [];
  const dqPath = path.join(REPO_ROOT, ".claude", "decision-queue.json");
  const hopperPath = path.join(REPO_ROOT, ".claude", "task-hopper.json");

  try {
    if (fs.existsSync(dqPath)) {
      const dq = JSON.parse(fs.readFileSync(dqPath, "utf8"));
      const pending = Array.isArray(dq.pending) ? dq.pending : [];
      if (pending.length > 0) {
        const ids = pending.slice(0, 3).map((p: any) => `#${p?.id ?? "?"}`).join(", ");
        const more = pending.length <= 3 ? "" : ` (+${pending.length - 3} more)`;
        parts.push(`DQ pending: ${pending.length} [${ids}${more}]`);
      }
    }
  } catch {
    // stay silent on malformed transient coordination files
  }

  try {
    if (fs.existsSync(hopperPath)) {
      const hopper = JSON.parse(fs.readFileSync(hopperPath, "utf8"));
      const tasks = Array.isArray(hopper.tasks) ? hopper.tasks : [];
      const inProgress = tasks.filter((t: any) => t?.status === "in_progress");
      const escalated = tasks.filter((t: any) => t?.status === "escalated");
      if (inProgress.length > 0) parts.push(`hopper in_progress: ${inProgress.length} [${inProgress.slice(0, 3).map((t: any) => t?.id ?? "?").join(", ")}]`);
      if (escalated.length > 0) parts.push(`hopper escalated: ${escalated.length} [${escalated.slice(0, 3).map((t: any) => t?.id ?? "?").join(", ")}]`);
    }
  } catch {
    // stay silent on malformed transient coordination files
  }

  return parts.join(" | ");
}

export default function lemmyHooks(pi: ExtensionAPI) {
  let projectContext = "";
  let ruleFiles: string[] = [];
  let prePhaseReminder = "";
  let editsSinceRead = 0;
  // Tracks whether the user has toggled focused CI-debug mode. Auto-commit
  // is repo-wide disabled; `/ci-debug-mode` now only changes mode/status so
  // CI work follows the same manual Brehon commit workflow as other work.
  let ciDebugMode = false;
  let brehonMode: BrehonMode = "main-safe";
  let overridePaths: Set<string> = new Set();  // manual path-policy overrides (one-time use)


  const setModeStatus = (ctx: any) => {
    try {
      ctx?.ui?.setStatus?.("brehon-mode", BREHON_MODES[brehonMode].label);
      ctx?.ui?.setStatus?.("ci-debug", ciDebugMode ? "CI-MODE" : undefined);
    } catch {
      // status is best-effort
    }
  };

  const setBrehonMode = (mode: BrehonMode, ctx: any) => {
    brehonMode = mode;
    ciDebugMode = mode === "ci-debug";
    setModeStatus(ctx);
  };

  // pi-harness-factory removed 2026-06-10 — the active-profile mechanism
  // duplicated Claude Code's advisor-orchestrator role discipline and was
  // too restrictive (locked pi advisor out of advisor-authored paths like
  // .claude/lessons/ in main-safe). The four-role model now lives in
  // AGENTS.md + .pi/PROJECT_CONTEXT.md + .claude/rules/advisor-orchestrator.md;
  // the /brehon-mode slash command sets the mode directly.

  pi.registerCommand("brehon-mode", {
    description: "Show or switch first-class Brehon Pi modes: main-safe, planning, impl-task, review-readonly, bm, ci-debug, harness-maintenance.",
    handler: async (args: string, ctx: any) => {
      const requested = args.trim();
      if (!requested || requested === "list") {
        const lines = Object.entries(BREHON_MODES).map(([mode, def]) => `${mode}${mode === brehonMode ? " *" : ""}: ${def.description}`);
        safeNotify(ctx, `Current Brehon mode: ${brehonMode}\n\n${lines.join("\n")}`, "info");
        setModeStatus(ctx);
        return;
      }
      if (!(requested in BREHON_MODES)) {
        safeNotify(ctx, `Unknown Brehon mode: ${requested}. Run /brehon-mode list.`, "error");
        return;
      }
      const mode = requested as BrehonMode;
      setBrehonMode(mode, ctx);
      safeNotify(ctx, `Brehon mode set to ${mode}: ${BREHON_MODES[mode].description}`, "info");
    },
  });

  pi.registerCommand("ci-debug-mode", {
    description:
      "Toggle CI-debug mode for focused workflow/script diagnostics. Auto-commit is disabled repo-wide; use the normal Brehon manual commit workflow when the fix is real.",
    handler: async (_args: string, ctx: any) => {
      ciDebugMode = !ciDebugMode;
      if (ciDebugMode) brehonMode = "ci-debug";
      else if (brehonMode === "ci-debug") brehonMode = "main-safe";
      setModeStatus(ctx);
      const state = ciDebugMode ? "ON" : "OFF";
      const detail = ciDebugMode
        ? "Manual git add/commit when ready."
        : "Back to main-safe mode. Manual git add/commit remains in effect.";
      safeNotify(ctx, `ci-debug-mode: ${state}. ${detail}`, "info");
    },
  });

  // Convenience aliases for /brehon-mode <role>
  const roleAliases: [string, BrehonMode, string][] = [
    ["advisor", "main-safe", "Advisor mode: normal pi repo work with Brehon constraints."],
    ["planner", "planning", "Planning mode: author plan files from briefs; never write Rust code."],
    ["impl", "impl-task", "Implementation mode: execute scoped tasks from approved plans."],
    ["bm", "bm", "Branch Manager mode: git/PR lifecycle; prefer delegating to .pi/agents/bm-pi.md."],
  ];
  for (const [alias, mode, desc] of roleAliases) {
    pi.registerCommand(alias, {
      description: `Switch to ${mode} Brehon mode. ${desc}`,
      handler: async (_args: string, ctx: any) => {
        setBrehonMode(mode, ctx);
        safeNotify(ctx, `Switched to ${BREHON_MODES[mode].label}: ${desc}`, "info");
      },
    });
  }

  pi.registerCommand("brehon-override", {
    description:
      "Approve a one-time write to a path normally blocked by Brehon path policy. Use sparingly — switch mode instead for repeated writes.",
    handler: async (args: string, ctx: any) => {
      const targetPath = args.trim();
      if (!targetPath) {
        safeNotify(ctx, "Usage: /brehon-override <relative-path> — e.g. /brehon-override .claude/PRPs/reports/my-retro.md", "warning");
        return;
      }
      overridePaths.add(targetPath);
      safeNotify(ctx, `Override approved for: ${targetPath} (valid for one write, this session only). Write now.`, "info");
    },
  });

  pi.on("session_start", async (_event: any, ctx: any) => {
    try {
      projectContext = readIfExists(path.join(REPO_ROOT, ".pi", "PROJECT_CONTEXT.md"));
      ruleFiles = findMarkdownFiles(path.join(REPO_ROOT, ".claude", "rules"));
      prePhaseReminder = extractAdditionalContext(runHookScript("pre-phase-audit.sh", undefined, 10_000).stdout);
      const pmdGuard = runHookScript("pmd-http-guard.sh", undefined, 5_000);
      const pmdNotice = [pmdGuard.stdout.trim(), pmdGuard.stderr.trim()].filter(Boolean).join("\n");

      setModeStatus(ctx);
      const loaded: string[] = [];
      if (ruleFiles.length > 0) loaded.push(`${ruleFiles.length} rule index entries`);
      if (prePhaseReminder) loaded.push("pre-phase audit reminder");
      if (pmdNotice) {
        loaded.push("PMD HTTP guard");
        safeNotify(ctx, pmdNotice.slice(0, 700), pmdGuard.stderr.trim() ? "warning" : "info");
      }
      loaded.push(`Brehon mode ${brehonMode}`);
      if (loaded.length > 0) safeNotify(ctx, `lemmy-hooks loaded: ${loaded.join(", ")}`, "info");
    } catch (err) {
      console.error("[lemmy-hooks] session_start failed:", err);
    }
  });

  // --- Progressive-disclosure helpers (pi-harness-context-injection plan, Tasks 1-4) ---

  function sliceProjectContext(headings: string[]): string {
    if (!projectContext || headings.length === 0) return projectContext;
    const parts = projectContext.split(/\n(?=## )/);
    const preamble = parts[0];
    const wanted = new Set(headings);
    const included: string[] = [preamble];
    for (let i = 1; i < parts.length; i++) {
      const headingLine = parts[i].split("\n")[0].replace(/^## /, "");
      if (wanted.has(headingLine)) included.push(parts[i]);
    }
    return included.join("\n");
  }

  function authorizedPathsNotice(mode: BrehonMode): string {
    const ctx = MODE_CONTEXT[mode];
    if (!ctx || ctx.authorizedReadPaths.length === 0) return "";
    const lines = ctx.authorizedReadPaths.map((p) => `- ${p}`);
    return `**Authorized .claude/ paths for this session:**\n${lines.join("\n")}\n\n(Override of AGENTS.md's blanket .claude/ read restriction. These paths are authorised because the current Brehon mode requires them.)`;
  }

  function filteredRuleIndex(mode: BrehonMode): string {
    const ctx = MODE_CONTEXT[mode];
    if (ruleFiles.length === 0) return "";
    const filtered = (!ctx || ctx.ruleFilter.length === 0)
      ? ruleFiles
      : ruleFiles.filter((f) => ctx.ruleFilter.some((rf) => f.endsWith(rf)));
    if (filtered.length === 0) return "";
    return `## Available Project Rules\n\nThe repo has additional Claude-era rule files. Read relevant files with the read tool when a task touches their topic:\n\n${filtered.map((file) => `- .claude/rules/${file}`).join("\n")}`;
  }

  function autoInjectSkill(mode: BrehonMode): string | null {
    const ctx = MODE_CONTEXT[mode];
    if (!ctx?.recommendedSkill) return null;
    const skillPath = path.join(REPO_ROOT, ctx.recommendedSkill);
    if (!fs.existsSync(skillPath)) return null;
    try {
      const content = fs.readFileSync(skillPath, "utf8").trim();
      const skillName = ctx.recommendedSkill.replace(/^\.pi\/skills\//, "").replace(/\/SKILL\.md$/, "");
      // Strip YAML frontmatter (--- ... ---) to avoid confusing the LLM
      const body = content.replace(/^---\n[\s\S]*?\n---\n?/, "").trim();
      return `## Active Skill: ${skillName}\n\nAuto-loaded because Brehon mode is ${BREHON_MODES[mode].label}. The full skill content follows:\n\n${body}`;
    } catch {
      return null;
    }
  }

  // --- before_agent_start (progressive disclosure by mode) ---

  pi.on("before_agent_start", async (event: any) => {
    try {
      const ctx = MODE_CONTEXT[brehonMode];
      const additions: string[] = [];

      // 1. Project context sliced by mode
      if (projectContext) {
        const headings = ctx?.projectContextHeadings ?? [];
        const sliced = sliceProjectContext(headings);
        additions.push(`## Pi Project Context\n\n${sliced}`);
      }

      // 2. Mode instructions + persona
      const personaLine = ctx?.persona ? `\n\n${ctx.persona}` : "";
      const authPaths = authorizedPathsNotice(brehonMode);
      const authSection = authPaths ? `\n\n${authPaths}` : "";
      additions.push(
        `## Active Brehon Pi Mode\n\nMode: ${brehonMode}\n${BREHON_MODES[brehonMode].description}${personaLine}\n\nMandatory mode instructions:\n${BREHON_MODES[brehonMode].instructions.map((line) => `- ${line}`).join("\n")}${authSection}\n\nPath-policy enforcement is active in .pi/extensions/lemmy-hooks.ts; use /brehon-mode list to inspect or switch modes.`,
      );

      // 3. Pre-phase audit reminder (if applicable)
      if (prePhaseReminder) additions.push(`## Pre-Phase Audit Reminder\n\n${prePhaseReminder}`);

      // 4. Coordination state
      const coord = coordinationStateSummary();
      if (coord) additions.push(`## Coordination State\n\n${coord}`);

      // 5. Auto-inject role skill
      const skillInjection = autoInjectSkill(brehonMode);
      if (skillInjection) additions.push(skillInjection);

      // 6. Filtered rule index
      const ruleSection = filteredRuleIndex(brehonMode);
      if (ruleSection) additions.push(ruleSection);

      if (additions.length === 0) return undefined;
      return { systemPrompt: `${event.systemPrompt}\n\n${additions.join("\n\n")}` };
    } catch (err) {
      console.error("[lemmy-hooks] before_agent_start failed:", err);
      return undefined;
    }
  });

  pi.on("tool_call", async (event: any, ctx: any) => {
    // Capture cwd synchronously before any await — ctx becomes stale if a
    // compaction reload fires during an awaited confirmDangerousBash call.
    const cwd = ctx.cwd;
    try {
      if (event.toolName === "bash") {
        const command = (event.input as any)?.command;
        if (typeof command === "string") {
          const rawCargo = rawCargoPattern(command);
          if (rawCargo) return { block: true, reason: `Brehon cargo policy: ${rawCargo}` };
          const secretShell = sensitiveShellPattern(command);
          if (secretShell) return { block: true, reason: `Brehon secret policy blocks ${secretShell}` };
          const firewall = await confirmDangerousBash(command, ctx);
          if (firewall) return firewall;
        }
      }

      if (event.toolName === "edit" || event.toolName === "write") {
        const policy = pathPolicyDecision(brehonMode, event.toolName, (event.input as any)?.path ?? (event.input as any)?.filePath);
        if (policy) return policy;
      }

      if (event.toolName === "read") {
        const relPath = typeof (event.input as any)?.path === "string" ? repoRelativePath((event.input as any).path) : "";
        if (/\.env|id_rsa|id_ed25519|credentials|secret|\.pem$|\.key$/i.test(relPath)) {
          return { block: true, reason: `Brehon secret policy blocks direct read of secret-like path: ${relPath}` };
        }
      }

      if (event.toolName === "bash" || event.toolName === "edit" || event.toolName === "write") {
        const guard = runHookScript("worktree-guard.sh", claudeCompatibleInput(event, cwd));
        const decision = worktreeGuardDecision(guard.stdout);
        if (decision) return decision;
      }

      return undefined;
    } catch (err) {
      console.error("[lemmy-hooks] tool_call failed:", err);
      return undefined;
    }
  });

  pi.on("user_bash", async (event: any, ctx: any) => {
    try {
      const rawCargo = rawCargoPattern(event.command);
      if (rawCargo) {
        return {
          result: {
            output: `Brehon cargo policy: ${rawCargo}`,
            exitCode: 1,
            cancelled: false,
            truncated: false,
          },
        };
      }
      const secretShell = sensitiveShellPattern(event.command);
      if (secretShell) {
        return {
          result: {
            output: `Brehon secret policy blocks ${secretShell}`,
            exitCode: 1,
            cancelled: false,
            truncated: false,
          },
        };
      }
      const decision = await confirmDangerousBash(event.command, ctx);
      if (!decision) return undefined;
      return {
        result: {
          output: decision.reason,
          exitCode: 1,
          cancelled: false,
          truncated: false,
        },
      };
    } catch (err) {
      console.error("[lemmy-hooks] user_bash failed:", err);
      return undefined;
    }
  });

  pi.on("tool_result", async (event: any, ctx: any) => {
    // Capture cwd synchronously before any await — reload safety (same rationale as tool_call).
    const cwd = ctx.cwd;
    try {
      const tool = event.toolName;
      const hookPayload = claudeCompatibleInput({ toolName: tool, input: event.input }, cwd);

      runHookScript("observation-capture.sh", hookPayload);

      if (tool === "read") {
        editsSinceRead = 0;
      } else if (tool === "edit" || tool === "write") {
        editsSinceRead += 1;
        if (editsSinceRead >= EDIT_READBACK_THRESHOLD) {
          safeNotify(ctx, `${EDIT_READBACK_THRESHOLD} edits/writes without a read. Spot-check modified files.`, "warning");
          editsSinceRead = 0;
        }
      }

      if (tool !== "edit" && tool !== "write") return undefined;
      if (event.isError) return undefined;

      const lessonReminder = runHookScript("lesson-frontmatter-reminder.sh", hookPayload, 5_000);
      const lessonReminderNotice = [lessonReminder.stdout.trim(), lessonReminder.stderr.trim()].filter(Boolean).join("\n");
      if (lessonReminderNotice) safeNotify(ctx, lessonReminderNotice.slice(0, 700), "warning");

      // Auto-commit is intentionally disabled. Pi edits now remain as normal
      // working-tree changes so the existing Brehon commit/push workflow stays
      // in control of staging, commit grouping, and branch publication.

      const lessonSync = runHookScript("lesson-pmd-sync.sh", hookPayload, 15_000);
      const lessonSyncNotice = [lessonSync.stdout.trim(), lessonSync.stderr.trim()].filter(Boolean).join("\n");
      if (lessonSyncNotice) safeNotify(ctx, lessonSyncNotice.slice(0, 700), lessonSyncNotice.includes("unreachable") || lessonSyncNotice.includes("not auto-synced") ? "warning" : "info");

      return undefined;
    } catch (err) {
      console.error("[lemmy-hooks] tool_result failed:", err);
      return undefined;
    }
  });

  pi.on("session_shutdown", async (_event: any, ctx: any) => {
    try {
      const retro = runHookScript("retro-check.sh", undefined, 10_000);
      if (retro.status === 2 && retro.stderr.trim()) {
        safeNotify(ctx, `Retro reminder (warn-only in pi): ${retro.stderr.trim().slice(0, 500)}`, "warning");
      }
    } catch (err) {
      console.error("[lemmy-hooks] session_shutdown failed:", err);
    }
  });
}
