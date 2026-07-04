# Design Quality

Frontend work activates the design module. Follow these rules whenever touching UI.

## Source of Truth

Must:
- Use DESIGN_DNA.md as the primary style reference if it exists
- Reuse existing components before creating new ones
- Follow established spacing, typography, and color tokens

## Quality Checks

Must verify before marking UI work done:
- Empty state handled (no blank voids)
- Loading state handled (no layout shift)
- Error state handled (user-visible, not console-only)
- Responsive layout holds at mobile and desktop breakpoints
- Visual hierarchy is clear without relying on color alone
- Accessible: keyboard navigable, sufficient contrast, meaningful alt text

Should:
- Check spacing consistency against existing patterns
- Verify typography scale matches the design system

## Anti-Patterns

Never:
- Add random gradients or box shadows not in the design system
- Use generic card layouts not established in the project
- Build fake UI (placeholder content passed off as real states)
- Ignore empty/loading/error states

## No DESIGN_DNA Case

- Small UI change: match the existing visual style exactly
- Major new UI surface: create a lightweight DESIGN_DNA.md first, then build
