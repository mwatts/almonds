#!/usr/bin/env node
import { execSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { DIRECTORIES } from "./directories.mjs";
import { contains } from "./prelude.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function cleanConsole() {
  execSync("rm -rf node_modules dist .nuxt", {
    cwd: resolve(ROOT, "console"),
    stdio: "inherit",
  });
  execSync("cargo clean", {
    cwd: resolve(ROOT, "console/src-tauri"),
    stdio: "inherit",
  });
}

function cleanServer() {
  execSync("cargo clean", { cwd: resolve(ROOT, "server"), stdio: "inherit" });
}

function clean() {
  const target = process.argv[2];

  if (!contains(DIRECTORIES, target)) {
    console.error(
      `Invalid target '${target ?? ""}'. Use one of: console, server, all`,
    );
    process.exit(1);
  }

  console.log(`Cleaning ${target} build assets`);

  try {
    if (target === "console") {
      cleanConsole();
    } else if (target === "server") {
      cleanServer();
    } else if (target === "all") {
      cleanConsole();
      cleanServer();
    }
  } catch (err) {
    console.error(err.stderr ?? err.message);
    process.exit(1);
  }
}

clean();
