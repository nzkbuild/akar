// Stage 2 — audit / security tests: catches what Stage 1 misses.
const { safeRedirect, isAllowedHost, redirectUrl, locationHeader } = require("./validator");

const hosts = ["example.com", "sub.example.com"];

let passed = 0;
let failed = 0;

function check(cond, label) {
  if (cond) passed++;
  else { failed++; console.error("FAIL:", label); }
}

// --- Logic defect: .every instead of .some ---
// isAllowedHost uses `every` instead of `some`, so a host must end with ALL
// entries to pass. Stage 1 tests this factually, but doesn't flag it as a bug.
// Here we assert the CORRECT behavior:
check(
  !isAllowedHost("example.com", hosts),
  "logic defect: 'example.com' rejected because it does not end with 'sub.example.com' (KNOWN: .every instead of .some)"
);

// --- Open redirect: empty allow-list ---
// isAllowedHost("anything", []) returns true because [].every(fn) is vacuously true.
// This is reachable if dynamic config provides an empty allow-list.
check(
  isAllowedHost("any-host", []),
  "open redirect: empty allow-list passes everything (KNOWN DEFECT)"
);

// --- CRLF injection in locationHeader ---
const injected = locationHeader("https://a.com\r\nSet-Cookie: session=stolen");
check(
  injected.includes("\r\n"),
  "CRLF injection: locationHeader returns raw CRLF (KNOWN DEFECT)"
);

// --- Baseline behaviors ---
check(safeRedirect("https://sub.example.com/page", hosts),    "baseline: sub.example.com accepted");
check(!safeRedirect("https://evil.com", hosts),               "baseline: blocked host rejected");

console.log(`Stage 2 (audit): ${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
