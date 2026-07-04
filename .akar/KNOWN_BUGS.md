# Known Bugs

<!-- Track bugs that are acknowledged but not yet fixed.
     Update Status to "fixed <date>" when resolved; never delete rows. -->

| Date | Bug | Pattern | Prevention | Status |
|------|-----|---------|------------|--------|
| 2026-01-01 | Example: rotate_if_needed race on Windows | File rename while another process holds a read handle | Use a tmp-swap pattern or retry loop | open |
