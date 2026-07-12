// HTTP redirect validator — deliberately contains known defects.
//
// FIX TARGET: This file is the baseline for a redirect-validation benchmark.
//
// Defect A (functional): `isAllowedHost` uses `.every()` instead of `.some()`,
//   so a host must end with ALL allow-list entries to pass. "example.com"
//   is rejected because it does not end with "sub.example.com". Stage 1
//   tests assert the CORRECT behavior and will FAIL until this is fixed.
//
// Defect B (security): `isAllowedHost` has no guard for an empty allow-list.
//   `[].every()` and `[].some()` are both vacuously true, so any host passes
//   when the allow-list is empty. Stage 2 tests assert the CORRECT behavior.
//
// Defect C (security): `locationHeader` does not strip CRLF characters,
//   enabling HTTP header injection. Stage 2 tests assert the CORRECT
//   behavior (CRLF stripped).

/**
 * Validate a redirect URL against an allow-list of trusted hosts.
 * Returns true if the redirect is safe.
 */
function safeRedirect(url, allowedHosts) {
  if (!url) return false;
  let host;
  try {
    host = new URL(url).hostname;
  } catch {
    return false;
  }
  if (!host) return false;
  return isAllowedHost(host, allowedHosts);
}

function isAllowedHost(host, allowedHosts) {
  // BUG: empty-string host passes the every check vacuously.
  return allowedHosts.every((allowed) => host.endsWith(allowed));
}

/**
 * Build a redirect URL. Does NOT sanitize CRLF — caller must validate first.
 */
function redirectUrl(target) {
  return "/redirect?to=" + encodeURIComponent(target);
}

/**
 * Build a Location header value. Does NOT strip CRLF.
 */
function locationHeader(target) {
  return target; // BUG: no CRLF sanitization
}

module.exports = { safeRedirect, isAllowedHost, redirectUrl, locationHeader };
