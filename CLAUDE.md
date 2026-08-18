# Conventions

## Comments

Write almost none. The code is the source of truth.

A comment is only justified when the fact **cannot be expressed in code**: external behaviour that
contradicts what the code looks like, a non-obvious invariant, or genuinely dense logic. One line.

```bash
# `keyword` is a no-op on Lua configs and `eval` answers ok for rejected rules
mon_eval() { ... }
```

Never write a comment to record what you worked out, why a change was made, or what used to be
broken. That goes in the commit message or `wiki/`, not the source.

No preambles above functions, files or sections. If a function needs one, the name is wrong —
rename it instead.
