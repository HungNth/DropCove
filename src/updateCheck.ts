export type UpdateCheckResult =
  | { kind: "up-to-date" }
  | { kind: "available"; version: string; url: string }
  | { kind: "no-release" }
  | { kind: "rate-limited" }
  | { kind: "failed" };

type Version = [number, number, number];

const FAILED: UpdateCheckResult = { kind: "failed" };
const REPOSITORY = /^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/;
const SEMVER = /^v?(\d+)\.(\d+)\.(\d+)$/;

function parseVersion(tag: string): Version | null {
  const match = SEMVER.exec(tag);
  return match ? [Number(match[1]), Number(match[2]), Number(match[3])] : null;
}

function compare(a: Version, b: Version): number {
  return a[0] - b[0] || a[1] - b[1] || a[2] - b[2];
}

function isRelease(value: unknown): value is { tag_name: string; draft: boolean; prerelease: boolean; html_url: string } {
  const release = value as Record<string, unknown> | null;
  return (
    typeof release === "object" &&
    release !== null &&
    typeof release.tag_name === "string" &&
    typeof release.draft === "boolean" &&
    typeof release.prerelease === "boolean" &&
    typeof release.html_url === "string"
  );
}

export async function checkForUpdates(
  runningVersion: string,
  repositoryUrl: string,
  fetchImpl: typeof fetch = (input, init) => fetch(input, init),
  timeoutMs = 10_000,
): Promise<UpdateCheckResult> {
  const repository = REPOSITORY.exec(repositoryUrl);
  const running = parseVersion(runningVersion);
  if (!repository || !running) return FAILED;
  const [, owner, repo] = repository;

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  let body: unknown;
  try {
    const response = await fetchImpl(`https://api.github.com/repos/${owner}/${repo}/releases`, {
      headers: { Accept: "application/vnd.github+json" },
      signal: controller.signal,
    });
    if (response.status === 403 || response.status === 429) return { kind: "rate-limited" };
    if (!response.ok) return FAILED;
    body = await response.json();
  } catch {
    return FAILED;
  } finally {
    clearTimeout(timer);
  }

  if (!Array.isArray(body) || !body.every(isRelease)) return FAILED;

  let latest: { version: Version; url: string } | null = null;
  for (const release of body) {
    if (release.draft || release.prerelease) continue;
    const version = parseVersion(release.tag_name);
    if (version && (!latest || compare(version, latest.version) > 0)) {
      latest = { version, url: release.html_url };
    }
  }
  if (!latest) return { kind: "no-release" };
  if (compare(latest.version, running) <= 0) return { kind: "up-to-date" };
  if (!latest.url.startsWith(`https://github.com/${owner}/${repo}/releases/`)) return FAILED;
  return { kind: "available", version: latest.version.join("."), url: latest.url };
}
