import fs from "node:fs";

const SEMVER_PATTERN =
  "(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-((?:0|[1-9]\\d*|\\d*[A-Za-z-][0-9A-Za-z-]*)(?:\\.(?:0|[1-9]\\d*|\\d*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\\+([0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*))?";
const SEMVER_RE = new RegExp(`^${SEMVER_PATTERN}$`);

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, "utf8"));
}

function writeJson(path, value) {
  fs.writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function updatePackageVersionInCargoToml(path, version) {
  const lines = fs.readFileSync(path, "utf8").split(/\r?\n/);
  let inPackage = false;
  let changed = false;

  const nextLines = lines.map((line) => {
    const section = line.match(/^\s*\[([^\]]+)\]\s*$/);
    if (section) {
      inPackage = section[1].trim() === "package";
      return line;
    }

    if (inPackage && /^\s*version\s*=/.test(line)) {
      changed = true;
      return line.replace(/"[^"]+"/, `"${version}"`);
    }

    return line;
  });

  if (!changed) {
    throw new Error(`Could not find [package].version in ${path}`);
  }

  fs.writeFileSync(path, nextLines.join("\n"));
}

function updatePackageVersionInCargoLock(path, packageName, version) {
  const lines = fs.readFileSync(path, "utf8").split(/\r?\n/);
  let matchedPackage = false;
  let changed = false;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];

    if (/^\s*\[\[package\]\]\s*$/.test(line)) {
      matchedPackage = false;
      continue;
    }

    const name = line.match(/^\s*name\s*=\s*"([^"]+)"\s*$/);
    if (name) {
      matchedPackage = name[1] === packageName;
      continue;
    }

    if (matchedPackage && /^\s*version\s*=/.test(line)) {
      lines[index] = line.replace(/"[^"]+"/, `"${version}"`);
      changed = true;
      break;
    }
  }

  if (!changed) {
    throw new Error(`Could not find ${packageName}.version in ${path}`);
  }

  fs.writeFileSync(path, lines.join("\n"));
}

function main() {
  const rawVersion = process.argv[2];
  if (!rawVersion) {
    throw new Error("Usage: npm run version:set -- <semver>");
  }

  const version = rawVersion.startsWith("v") ? rawVersion.slice(1) : rawVersion;
  if (!SEMVER_RE.test(version)) {
    throw new Error(`Version must be SemVer, got: ${rawVersion}`);
  }

  const packageJson = readJson("package.json");
  packageJson.version = version;
  writeJson("package.json", packageJson);

  const packageLock = readJson("package-lock.json");
  packageLock.version = version;
  if (packageLock.packages?.[""]) {
    packageLock.packages[""].version = version;
  }
  writeJson("package-lock.json", packageLock);

  const tauriConfig = readJson("src-tauri/tauri.conf.json");
  tauriConfig.version = version;
  writeJson("src-tauri/tauri.conf.json", tauriConfig);

  updatePackageVersionInCargoToml("src-tauri/Cargo.toml", version);
  updatePackageVersionInCargoLock("src-tauri/Cargo.lock", "skill-notebook", version);

  console.log(`Updated SkillNotebook version to ${version}`);
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
