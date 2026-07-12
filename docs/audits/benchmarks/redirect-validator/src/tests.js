// Stage 1 — functional tests: validates CORRECT redirect behavior.
// MUST FAIL on the known-defect baseline (the .every bug rejects valid hosts).
// After fix, these all pass.
const { safeRedirect, isAllowedHost, redirectUrl } = require("./validator");

const hosts = ["example.com", "sub.example.com"];

let passed = 0;
let failed = 0;

function check(cond, label) {
  if (cond) passed++;
  else { failed++; console.error("FAIL:", label); }
}

// --- Host matching (correct behavior — after .every→.some fix) ---
// With .some: "example.com" matches "example.com" entry → accepted.
// With .every (buggy): "example.com" does NOT end with "sub.example.com" → rejected.
check(
  safeRedirect("https://example.com/page", hosts),
  "accepts exact host match in allow-list"
);
check(
  safeRedirect("https://sub.example.com/page", hosts),
  "accepts subdomain host in allow-list"
);
check(
  safeRedirect("https://www.example.com/page", hosts),
  "accepts matching subdomain (ends with .example.com)"
);

// --- Blocking — unknown hosts rejected ---
check(
  !safeRedirect("https://evil.com/page", hosts),
  "rejects host not in allow-list"
);
check(
  !safeRedirect("https://example.com.evil.com/page", hosts),
  "rejects suffix-confusable host"
);

// --- Edge cases ---
check(!safeRedirect("", hosts),     "rejects empty url");
check(!safeRedirect(null, hosts),   "rejects null url");
check(!safeRedirect("not-a-url", hosts), "rejects garbage url");

// --- redirectUrl builder ---
check(
  redirectUrl("https://example.com/home").startsWith("/redirect?to="),
  "redirectUrl produces expected prefix"
);

console.log(`Stage 1 (functional): ${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
