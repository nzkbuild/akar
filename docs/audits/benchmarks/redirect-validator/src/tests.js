// Stage 1 — functional tests: basic validation + known blind spots.
const { safeRedirect, isAllowedHost, redirectUrl } = require("./validator");

const hosts = ["example.com", "sub.example.com"];

let passed = 0;
let failed = 0;

function check(cond, label) {
  if (cond) passed++;
  else { failed++; console.error("FAIL:", label); }
}

// --- isAllowedHost baseline ---
// The code uses .every() instead of .some(), so only a host that ends with
// ALL entries passes. This means "example.com" does NOT pass because it
// doesn't end with "sub.example.com".
check(
  !isAllowedHost("example.com", hosts),
  "host must end with ALL entries (due to .every bug)"
);
check(
  isAllowedHost("sub.example.com", ["example.com", "sub.example.com"]),
  "sub.example.com ends with both entries"
);
check(
  isAllowedHost("x.example.com", ["example.com"]),
  "single allowed host with matching suffix"
);

// Open-redirect edge case: isAllowedHost("", []) returns true vacuously.
check(isAllowedHost("any", []), "empty allow-list passes everything");

// --- safeRedirect (wired to isAllowedHost) ---
// Only hosts that pass isAllowedHost (the .every bug) are accepted.
check(!safeRedirect("https://example.com/page", hosts),          "example.com REJECTED (every bug)");
check(safeRedirect("https://sub.example.com/page", hosts),       "sub.example.com accepted");
check(!safeRedirect("https://evil.com", hosts),                  "blocked host rejected");
check(!safeRedirect("", hosts),                                   "empty url rejected");
check(!safeRedirect(null, hosts),                                 "null url rejected");
check(!safeRedirect("not-a-url", hosts),                          "garbage rejected");

// --- redirectUrl builder ---
check(redirectUrl("https://example.com/home").startsWith("/redirect?to="), "redirectUrl prefix");

// NOTE: isAllowedHost("", hosts) passes the .every check vacuously but is
// not reachable via safeRedirect() because new URL(...).hostname is never
// empty for valid URLs. The CLRF injection in redirectUrl/locationHeader
// is an audit (Stage 2) concern.

console.log(`Stage 1 (functional): ${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
