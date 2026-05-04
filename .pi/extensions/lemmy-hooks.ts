/**
 * Lemmy/Brehon project hooks for pi-coding.
 *
 * Code-repo profile: preserve Claude support, add pi-only guardrails.
 * - inject .pi/PROJECT_CONTEXT.md plus a rule-file index into the system prompt
 * - block or ask before destructive bash commands
 * - auto-commit successful edit/write tool changes only for the touched file
 * - remind after several edits without a readback
 */

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import * as fs from "node:fs";
import * as path from "node:path";
import { spawnSync } from "node:child_process";

const REPO_ROOT = path.resolve(__dirname, "..", "..");

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
];

const EDIT_READBACK_THRESHOLD = 5;

function safeNotify(ctx: any, msg: string, level: "info" | "warning" | "error" = "info") {
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
  const normalized = filePath.startsWith("/") ? filePath : path.join(REPO_ROOT, filePath);
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

export default function lemmyHooks(pi: ExtensionAPI) {
  let projectContext = "";
  let ruleFiles: string[] = [];
  let editsSinceRead = 0;

  pi.on("session_start", async (_event, ctx) => {
    try {
      projectContext = readIfExists(path.join(REPO_ROOT, ".pi", "PROJECT_CONTEXT.md"));
      ruleFiles = findMarkdownFiles(path.join(REPO_ROOT, ".claude", "rules"));
      if (ruleFiles.length > 0) safeNotify(ctx, `Found ${ruleFiles.length} .claude/rules file(s)`, "info");
    } catch (err) {
      console.error("[lemmy-hooks] session_start failed:", err);
    }
  });

  pi.on("before_agent_start", async (event) => {
    try {
      const additions: string[] = [];

      if (projectContext) {
        additions.push(`## Pi Project Context\n\n${projectContext}`);
      }

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
      if (event.toolName !== "bash") return undefined;
      const command = (event.input as any)?.command;
      if (typeof command !== "string") return undefined;
      return await confirmDangerousBash(command, ctx);
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
      spawnSync(
        "bash",
        [
          "-lc",
          `git add -- ${JSON.stringify(filePath)} && (git diff --cached --quiet || git commit -m ${JSON.stringify(`auto(pi): update ${basename}`)} --no-gpg-sign)`,
        ],
        { cwd: REPO_ROOT, encoding: "utf8", timeout: 30_000 },
      );

      return undefined;
    } catch (err) {
      console.error("[lemmy-hooks] tool_result failed:", err);
      return undefined;
    }
  });
}
