import fs from "node:fs";

const SEMVER_PATTERN =
  "(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-((?:0|[1-9]\\d*|\\d*[A-Za-z-][0-9A-Za-z-]*)(?:\\.(?:0|[1-9]\\d*|\\d*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\\+([0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*))?";
const SEMVER_RE = new RegExp(`^${SEMVER_PATTERN}$`);
const TAG_RE = new RegExp(`^v${SEMVER_PATTERN}$`);

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, "utf8"));
}

function packageVersionFromCargoToml(path) {
  const lines = fs.readFileSync(path, "utf8").split(/\r?\n/);
  let inPackage = false;

  for (const line of lines) {
    const section = line.match(/^\s*\[([^\]]+)\]\s*$/);
    if (section) {
      inPackage = section[1].trim() === "package";
      continue;
    }

    if (!inPackage) {
      continue;
    }

    const version = line.match(/^\s*version\s*=\s*"([^"]+)"\s*$/);
    if (version) {
      return version[1];
    }
  }

  throw new Error(`Could not find [package].version in ${path}`);
}

function packageVersionFromCargoLock(path, packageName) {
  const lines = fs.readFileSync(path, "utf8").split(/\r?\n/);
  let inPackage = false;
  let currentName = null;
  let currentVersion = null;

  for (const line of lines) {
    if (/^\s*\[\[package\]\]\s*$/.test(line)) {
      if (currentName === packageName && currentVersion) {
        return currentVersion;
      }
      inPackage = true;
      currentName = null;
      currentVersion = null;
      continue;
    }

    if (!inPackage) {
      continue;
    }

    const name = line.match(/^\s*name\s*=\s*"([^"]+)"\s*$/);
    if (name) {
      currentName = name[1];
      continue;
    }

    const version = line.match(/^\s*version\s*=\s*"([^"]+)"\s*$/);
    if (version) {
      currentVersion = version[1];
    }
  }

  if (currentName === packageName && currentVersion) {
    return currentVersion;
  }

  throw new Error(`Could not find ${packageName} in ${path}`);
}

function parseArgs(argv) {
  const args = { tag: null };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--tag") {
      if (!argv[index + 1]) {
        throw new Error("Missing value for --tag");
      }
      args.tag = argv[index + 1];
      index += 1;
      continue;
    }
    if (arg.startsWith("--tag=")) {
      args.tag = arg.slice("--tag=".length);
      continue;
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return args;
}

function main() {
  const { tag } = parseArgs(process.argv.slice(2));
  const packageJson = readJson("package.json");
  const packageLock = readJson("package-lock.json");
  const tauriConfig = readJson("src-tauri/tauri.conf.json");

  const appVersion = packageJson.version;
  const versions = [
    ["package.json", packageJson.version],
    ["package-lock.json", packageLock.version],
    ['package-lock.json packages[""]', packageLock.packages?.[""]?.version],
    ["src-tauri/tauri.conf.json", tauriConfig.version],
    ["src-tauri/Cargo.toml", packageVersionFromCargoToml("src-tauri/Cargo.toml")],
    [
      "src-tauri/Cargo.lock",
      packageVersionFromCargoLock("src-tauri/Cargo.lock", "skill-notebook"),
    ],
  ];

  if (!SEMVER_RE.test(appVersion)) {
    throw new Error(`package.json version is not valid SemVer: ${appVersion}`);
  }

  const mismatches = versions.filter(([, version]) => version !== appVersion);
  if (mismatches.length > 0) {
    const detail = mismatches
      .map(([source, version]) => `- ${source}: ${version ?? "<missing>"}`)
      .join("\n");
    throw new Error(`Version files do not match ${appVersion}:\n${detail}`);
  }

  if (tag) {
    if (!TAG_RE.test(tag)) {
      throw new Error(`Release tag must be v-prefixed SemVer, got: ${tag}`);
    }

    const tagVersion = tag.slice(1);
    if (tagVersion !== appVersion) {
      throw new Error(`Release tag ${tag} does not match app version ${appVersion}`);
    }
  }

  console.log(`Version check passed: ${appVersion}${tag ? ` (${tag})` : ""}`);
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
