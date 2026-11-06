import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const sourceDirectory = path.join(path.dirname(fileURLToPath(import.meta.url)), "../src");

/** Reads a file from client/src. The tests compile the real sources inside a vm sandbox. */
export const readSource = (file: string): string => fs.readFileSync(path.join(sourceDirectory, file), "utf8");

/** Drops ES import lines so a module can run in a sandbox whose context already provides them. */
export const withoutImports = (source: string): string => source.replace(/^import .*\r?\n/gm, "");
