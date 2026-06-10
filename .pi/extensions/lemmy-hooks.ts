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
 * - DQ/task-hopper coordination-state injection
 * - destructive bash firewall
 * - Junior worktree guard via copied .pi/hook-scripts/worktree-guard.sh
 * - observation shadow telemetry via copied .pi/hook-scripts/observation-capture.sh
 * - auto-commit on successful edit/write
 * - warn-only retro nudge on shutdown
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import * as fs from "node:fs";
import * as path from "node:path";
import { spawnSync } from "node:child_process";

const REPO_ROOT = path.resolve(__dirname, "..", "..");
const PI_HOOKS_DIR = path.join(REPO_ROOT, ".pi", "hook-scripts");

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

const AUTO_COMMIT_SKIP_FRAGMENTS = [
  "/.git/",
  "/node_modules/",
  "/target/",
  "/dist/",
  "/build/",
  "/.DS_Store",
  "/.pi/sessions/",
  "/.pi/tmp/",
  "/.pi/logs/",
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
    description: "CI debug mode: workflow/script diagnostics with auto-commit suppressed.",
    instructions: [
      "Prefer delegating GitHub Actions work to the .pi/agents/ci-debug.md project agent when available.",
      "Keep edits focused on .github/workflows/, .github/scripts/, or explicit CI docs/artifacts.",
      "Auto-commit is suppressed; manually commit only after the fix is real.",
    ],
  },
  "harness-maintenance": {
    label: "BREHON:HARNESS",
    description: "Harness metadata mode: skill frontmatter/docs and pi harness scripts only; no app code.",
    instructions: [
      "Use only for explicit harness-maintenance requests such as skill frontmatter fixes.",
      "Allowed writes are .claude/skills/, .pi/skills/, .pi/scripts/, .pi/extensions/, and .pi/harness-factory/.",
      "Validate skill metadata with python3 .pi/scripts/validate-skills.py after skill edits.",
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
const HARNESS_MAINTENANCE_WRITE_PREFIXES = [".claude/skills/", ".pi/skills/", ".pi/scripts/", ".pi/extensions/", ".pi/harness-factory/"];
const CODE_PREFIXES = ["crates/", "src/", "migrations/", "diesel_migrations/"];
const SOURCE_EXTENSIONS = new Set([".rs", ".ts", ".tsx", ".js", ".jsx", ".sql", ".toml", ".yml", ".yaml"]);
const FACTORY_ACTIVE_PATH = path.join(REPO_ROOT, ".pi", "harness-factory", "active.json");
const FACTORY_PROFILE_TO_MODE: Record<string, BrehonMode> = {
  "brehon-main-safe": "main-safe",
  "brehon-planning": "planning",
  "brehon-impl-task": "impl-task",
  "brehon-review-readonly": "review-readonly",
  "brehon-bm": "bm",
  "brehon-ci-debug": "ci-debug",
  "brehon-harness-maintenance": "harness-maintenance",
};

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

function shouldSkipAutoCommit(filePath: string): boolean {
  const normalized = path.resolve(REPO_ROOT, filePath);
  const withSlashes = normalized.replaceAll(path.sep, "/");
  return AUTO_COMMIT_SKIP_FRAGMENTS.some((fragment) => withSlashes.includes(fragment));
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
    return { block: true, reason: `Brehon dual-harness boundary blocks .claude writes in ${mode} mode unless user switches to planning or gives an explicit manual override: ${relPath}` };
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
  // When true, the tool_result handler skips its auto-commit-per-edit
  // step. Toggled by `/ci-debug-mode`. Reason: speculative edits during
  // CI debugging push commits that re-fire push-trigger workflows; if
  // the agent is operating on a wrong premise, each iteration creates
  // a fresh failure to react to and the loop accelerates rather than
  // converges. See .claude/lessons/feedback_gha_pi_loop_postmortem.md §6.
  let ciDebugMode = false;
  let brehonMode: BrehonMode = "main-safe";
  let lastFactoryActiveMtime = 0;

  const setModeStatus = (ctx: any) => {
    try {
      ctx?.ui?.setStatus?.("brehon-mode", BREHON_MODES[brehonMode].label);
      ctx?.ui?.setStatus?.("ci-debug", ciDebugMode ? "CI-AUTOCOMMIT-OFF" : undefined);
    } catch {
      // status is best-effort
    }
  };

  const setBrehonMode = (mode: BrehonMode, ctx: any) => {
    brehonMode = mode;
    ciDebugMode = mode === "ci-debug";
    setModeStatus(ctx);
  };

  const syncFactoryActiveMode = (ctx: any, notify = false) => {
    try {
      if (!fs.existsSync(FACTORY_ACTIVE_PATH)) return;
      const stat = fs.statSync(FACTORY_ACTIVE_PATH);
      if (stat.mtimeMs === lastFactoryActiveMtime) return;
      lastFactoryActiveMtime = stat.mtimeMs;

      const activeProfile = JSON.parse(fs.readFileSync(FACTORY_ACTIVE_PATH, "utf8"));
      const profileId = typeof activeProfile?.id === "string" ? activeProfile.id : "";
      const mappedMode = FACTORY_PROFILE_TO_MODE[profileId];
      if (!mappedMode) return;

      setBrehonMode(mappedMode, ctx);
      if (notify) safeNotify(ctx, `pi-harness-factory active profile ${profileId} mapped to /brehon-mode ${mappedMode}`, "info");
    } catch (err) {
      console.error("[lemmy-hooks] factory active sync failed:", err);
    }
  };

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
      "Toggle CI-debug mode. When ON, the auto-commit-per-edit hook is suppressed so iterating on .github/workflows/*.yml or .github/scripts/*.sh doesn't spam commits + retrigger CI on each save. Run again to turn back OFF when the fix is real.",
    handler: async (_args: string, ctx: any) => {
      ciDebugMode = !ciDebugMode;
      if (ciDebugMode) brehonMode = "ci-debug";
      else if (brehonMode === "ci-debug") brehonMode = "main-safe";
      setModeStatus(ctx);
      const state = ciDebugMode ? "ON" : "OFF";
      const detail = ciDebugMode
        ? "Auto-commit suppressed. Manual git add/commit when ready."
        : "Auto-commit re-enabled (per-edit auto(pi): commits resume).";
      safeNotify(ctx, `ci-debug-mode: ${state}. ${detail}`, "info");
    },
  });

  pi.on("session_start", async (_event: any, ctx: any) => {
    try {
      projectContext = readIfExists(path.join(REPO_ROOT, ".pi", "PROJECT_CONTEXT.md"));
      ruleFiles = findMarkdownFiles(path.join(REPO_ROOT, ".claude", "rules"));
      prePhaseReminder = extractAdditionalContext(runHookScript("pre-phase-audit.sh", undefined, 10_000).stdout);

      syncFactoryActiveMode(ctx, true);
      setModeStatus(ctx);
      const loaded: string[] = [];
      if (ruleFiles.length > 0) loaded.push(`${ruleFiles.length} rule index entries`);
      if (prePhaseReminder) loaded.push("pre-phase audit reminder");
      loaded.push(`Brehon mode ${brehonMode}`);
      if (loaded.length > 0) safeNotify(ctx, `lemmy-hooks loaded: ${loaded.join(", ")}`, "info");
    } catch (err) {
      console.error("[lemmy-hooks] session_start failed:", err);
    }
  });

  pi.on("before_agent_start", async (event: any) => {
    try {
      syncFactoryActiveMode(undefined);
      const additions: string[] = [];

      if (projectContext) additions.push(`## Pi Project Context\n\n${projectContext}`);
      additions.push(`## Active Brehon Pi Mode\n\nMode: ${brehonMode}\n${BREHON_MODES[brehonMode].description}\n\nMandatory mode instructions:\n${BREHON_MODES[brehonMode].instructions.map((line) => `- ${line}`).join("\n")}\n\nPath-policy enforcement is active in .pi/extensions/lemmy-hooks.ts; use /brehon-mode list to inspect or switch modes.`);
      if (prePhaseReminder) additions.push(`## Pre-Phase Audit Reminder\n\n${prePhaseReminder}`);

      const coord = coordinationStateSummary();
      if (coord) additions.push(`## Coordination State\n\n${coord}`);

      if (ruleFiles.length > 0) {
        additions.push(
          `## Available Project Rules\n\nThe repo has additional Claude-era rule files. Read relevant files with the read tool when a task touches their topic:\n\n${ruleFiles
            .map((file) => `- .claude/rules/${file}`)
            .join("\n")}`,
        );
      }

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
      syncFactoryActiveMode(ctx);
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
      syncFactoryActiveMode(ctx);
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

      runHookScript("observation-capture.sh", claudeCompatibleInput({ toolName: tool, input: event.input }, cwd));

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
      // ci-debug-mode suppresses auto-commit so iteration on workflow
      // files / CI scripts doesn't spam commits + retrigger workflows
      // on each save. See .claude/lessons/feedback_gha_pi_loop_postmortem.md §6.
      if (ciDebugMode) return undefined;

      const filePath = (event.input as any)?.path;
      if (typeof filePath !== "string" || shouldSkipAutoCommit(filePath)) return undefined;

      const basename = path.basename(filePath);
      // Invoke git directly via argument arrays so filePath/basename are never
      // shell-interpolated. Avoids command injection through model-controlled
      // tool inputs (cr-24 on PR #111). Both the diff probe and the commit are
      // scoped to filePath so unrelated pre-staged changes are never swept into
      // the auto-commit (cr-67 on PR #111).
      const gitOpts = { cwd: REPO_ROOT, encoding: "utf8" as const, timeout: 30_000 };
      const addRes = spawnSync("git", ["add", "--", filePath], gitOpts);
      if (addRes.status !== 0) return undefined;
      const diffRes = spawnSync(
        "git",
        ["diff", "--cached", "--quiet", "--", filePath],
        gitOpts,
      );
      if (diffRes.status === 1) {
        spawnSync(
          "git",
          [
            "commit",
            "-m",
            `auto(pi): update ${basename}`,
            "--no-gpg-sign",
            "--",
            filePath,
          ],
          gitOpts,
        );
      }

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
