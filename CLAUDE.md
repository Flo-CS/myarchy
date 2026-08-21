# Conventions

## Comments

Write almost none. The code is the source of truth.

A comment is only justified when the fact **cannot be expressed in code**: external behaviour that contradicts what the code looks like, a non-obvious invariant, or genuinely dense logic.

One line.

Never write a comment to record what you worked out, why a change was made, or what used to be broken.

No preambles above functions, files or sections. If a function needs one, the name is wrong, rename it instead.

## Tests

Few, and high in the call stack. Test the operation a command triggers, not the helpers under it.

A test is only justified when it **can fail on a real breakage**. One that restates the implementation only fails when the code changes on purpose — delete it.

One scenario per test. Four assertions about one action are one test, not four.

Assert an invariant across a range of inputs rather than writing a test per input.

Use real data: actual device names, modes, command output.

Never write a test for coverage. Before trusting one, break the code it covers and watch it fail.
