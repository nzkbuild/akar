// Stage 2 — security / audit tests: validates security properties.
// MUST FAIL on the known-defect baseline:
//   - CRLF injection in locationHeader
//   - empty allow-list passes everything (open redirect)
// After fix, these all pass.
const { isAllowedHost, locationHeader } = require("./validator");

let passed = 0;
let failed = 0;

function check(cond, label) {
  if (cond) passed++;
  else { failed++; console.error("FAIL:", label); }
}

// --- CRLF injection ---
// locationHeader must sanitize CRLF to prevent header injection.
const injected = locationHeader("https://a.com\r\nSet-Cookie: session=stolen");
check(
  !injected.includes("\r") && !injected.includes("\n"),
  "CRLF stripped from locationHeader"
);

const injected2 = locationHeader("https://a.com\nX-Injected: true");
check(
  !injected2.includes("\r") && !injected2.includes("\n"),
  "LF stripped from locationHeader"
);

// --- Empty allow-list guard ---
// An empty allow-list must not accept any host.
check(
  !isAllowedHost("any-host", []),
  "empty allow-list rejects all hosts"
);
check(
  !isAllowedHost("", []),
  "empty allow-list rejects empty host"
);

console.log(`Stage 2 (audit): ${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
