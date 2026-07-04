# Risk Levels

## Low
Read files, list directories, git status, run tests, run build, inspect logs.

Must: Proceed without confirmation.

## Medium
Install lockfile-pinned deps, run known scripts, generate new files, rename files.

Must: Proceed but note the action taken.

## High
Add a new dependency, run migrations, delete files, modify config files, make network calls.

Must: Surface the action and expected impact before or immediately after. Prefer reversible path.

## Critical
Force push, mass delete, print secrets, curl|bash pipe execution, rewrite auth/payment/security logic.

Must: Stop, explain risk, wait for explicit user confirmation.
Never: Proceed with a critical action autonomously.

## Rules

Should:
- Default to the higher level when uncertain
- Treat any action touching credentials, payments, or access control as critical

Never:
- Downgrade a risk level to avoid asking
- Chain high/critical actions without surfacing them
