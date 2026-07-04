# Lessons

<!-- Add a new entry each time something goes wrong or a non-obvious insight is found.
     Keep entries append-only — never delete, only supersede. -->

---

## Entry template

- **date**: 2026-01-01
- **scope**: project | global
- **confidence**: low | medium | high
- **source**: postmortem | observation | review
- **expires**: never | 2027-01-01
- **summary**: One sentence describing what was learned.
- **prevention**: Concrete action that prevents recurrence.

---

## Example

- **date**: 2026-01-01
- **scope**: project
- **confidence**: high
- **source**: postmortem
- **expires**: never
- **summary**: OpenOptions::append on Windows requires the file to be opened with write(true) as well.
- **prevention**: Always pair `.append(true).write(true)` in OpenOptions on any platform target.
