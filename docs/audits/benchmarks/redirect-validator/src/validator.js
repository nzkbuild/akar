// HTTP redirect validator — deliberately has known defects.
//
// DEFECT 1 (functional): allows open redirect to any host when isAllowedHost
// receives an empty string (edge case in `safeRedirect`).
//
// DEFECT 2 (security): `redirectUrl` does not sanitize CRLF, enabling header
// injection when the URL is reflected in a `Location` header (Stage 2 concern).

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
