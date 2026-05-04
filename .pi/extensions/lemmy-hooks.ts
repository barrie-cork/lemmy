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

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
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

function shouldSkipAutoCommit(filePath: string): boolean {
  const normalized = path.resolve(REPO_ROOT, filePath);
  const withSlashes = normalized.replaceAll(path.sep, "/");
  return AUTO_COMMIT_SKIP_FRAGMENTS.some((fragment) => withSlashes.includes(fragment));
}

function matchedBlockedPattern(command: string): string | undefined {
  const normalized = command.replace(/\s+/g, " ").trim();
  return BASH_BLOCKLIST.find((pattern) => normalized.includes(pattern));
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

  pi.on("session_start", async (_event, ctx) => {
    try {
      projectContext = readIfExists(path.join(REPO_ROOT, ".pi", "PROJECT_CONTEXT.md"));
      ruleFiles = findMarkdownFiles(path.join(REPO_ROOT, ".claude", "rules"));
      prePhaseReminder = extractAdditionalContext(runHookScript("pre-phase-audit.sh", undefined, 10_000).stdout);

      const loaded: string[] = [];
      if (ruleFiles.length > 0) loaded.push(`${ruleFiles.length} rule index entries`);
      if (prePhaseReminder) loaded.push("pre-phase audit reminder");
      if (loaded.length > 0) safeNotify(ctx, `lemmy-hooks loaded: ${loaded.join(", ")}`, "info");
    } catch (err) {
      console.error("[lemmy-hooks] session_start failed:", err);
    }
  });

  pi.on("before_agent_start", async (event) => {
    try {
      const additions: string[] = [];

      if (projectContext) additions.push(`## Pi Project Context\n\n${projectContext}`);
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

  pi.on("tool_call", async (event, ctx) => {
    try {
      if (event.toolName === "bash") {
        const command = (event.input as any)?.command;
        if (typeof command === "string") {
          const firewall = await confirmDangerousBash(command, ctx);
          if (firewall) return firewall;
        }
      }

      if (event.toolName === "bash" || event.toolName === "edit" || event.toolName === "write") {
        const guard = runHookScript("worktree-guard.sh", claudeCompatibleInput(event, ctx.cwd));
        const decision = worktreeGuardDecision(guard.stdout);
        if (decision) return decision;
      }

      return undefined;
    } catch (err) {
      console.error("[lemmy-hooks] tool_call failed:", err);
      return undefined;
    }
  });

  pi.on("user_bash", async (event, ctx) => {
    try {
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

  pi.on("tool_result", async (event, ctx) => {
    try {
      const tool = event.toolName;

      runHookScript("observation-capture.sh", claudeCompatibleInput({ toolName: tool, input: event.input }, ctx.cwd));

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

  pi.on("session_shutdown", async (_event, ctx) => {
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
