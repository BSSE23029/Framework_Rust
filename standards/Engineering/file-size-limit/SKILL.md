---
name: file-size-limit
description: The Dart file size rule in this repo — 300 lines as the target, 500 as the hard ceiling — and how to split a file that has outgrown it without changing behaviour. Use when a file is over 300 lines, when a hook reports the ceiling was crossed, when adding code to a file that is already close to it, or when deciding where a new file belongs in the folder tree.
---

# Dart file size: 300 soft, 500 hard

Two numbers, both **physical lines** — what `wc -l` prints. Not "300 excluding
comments", not "300 of real code". Countable in one command, with no argument about
what counts.

- **300 is the target.** Crossing it is a prompt to look for a seam, not a failure.
- **500 is the ceiling.** Nothing lives above it.

Applies to every `.dart` file under `lib/` and `test/`.

    .claude/skills/file-size-limit/check.sh                          # every file
    .claude/skills/file-size-limit/check.sh lib/x.dart lib/y.dart    # just these
    .claude/skills/file-size-limit/check.sh --hard                   # breaches only

`OVER` lines are the 300 nudge and do not fail the run; `HARD` lines are the 500
breach and exit 1. A `PostToolUse` hook in `.claude/settings.json` runs it on every
file Write/Edit touches and speaks up on the hard ceiling, so a breach is reported the
moment it happens rather than at review time.

## Why two numbers

One hard number at 300 would be a rule this codebase argues with. The doc comments
here are dense on purpose — a `WHY:` block explaining which thread a WebView blocks is
routinely longer than the method under it — and a single 300-line cap turns *"this
file explains itself well"* into *"this file must be cut in three"*. The pressure
lands on the comments, which are the one thing that must not give.

So: 300 says look, 500 says act. Between them, judgement. A 380-line file that is one
coherent idea with long explanations stays. A 380-line file holding three unrelated
widgets gets split, and would have been worth splitting at 200.

Do not pad a 40-line file, and do not split a 280-line file that reads as one idea.
The rule exists because a file you cannot hold in your head is a file you edit by
guessing. Under it, use whatever size the idea actually is.

## Exempt

Only these, and the checker already knows them:

- **Generated code** — `*.pb.dart`, `*.pbjson.dart`, `*.pbgrpc.dart`, `*.pbenum.dart`,
  `*.g.dart`, `*.freezed.dart`, `*.mocks.dart`, and anything under a `proto/` directory.
  These are rewritten by a generator that has never heard of this rule; splitting them
  is undone by the next `protoc` run.
- **`_rn_legacy/`** — the React Native source kept as a porting reference. Never
  modified at all, per CLAUDE.md.

Nothing else is exempt. "It's cohesive", "it's mostly comments", and "it's a big
switch" are not exemptions — they are descriptions of files that split cleanly.

## Comments are not the fat

This codebase's doc comments carry the *why*: the RN bug a port avoids, the platform
that forced a branch, the thread a WebView blocks. They are the most expensive thing
in the file to reconstruct and the first thing a careless split throws away.

**When a declaration moves, its doc comment moves with it, verbatim.** A file that is
long because of comment volume is split into more files. It is never brought under a
limit by deleting or shortening a comment. Losing a `// WHY:` to save eight lines is a
net loss, and one this repo will feel again the next time that code is touched. This
is exactly the case the 300–500 gap exists to absorb: use the room before you reach
for the comment.

Keep the one-line RN-origin comment (`/// … ported from src/screens/Foo.js.`) at the
top of whichever file inherits the port's entry point, and add one to each new file
naming the same origin and the slice it holds.

## Splitting without changing behaviour

A split is a **pure move**. Same code, new address. If a diff shows a changed
condition, a renamed field, a dropped `mounted` check, a reordered `await`, or a
`const` that was not there before, it is no longer a split and it needs to be
reviewed as the behaviour change it is.

Work in this order:

1. **Read the whole file first.** Splitting from a grep is how a listener stops being
   disposed.
2. **Find the seams** — a widget that only reads its own arguments, a group of pure
   functions, a state machine's transition table, a set of model classes, one tab of a
   screen. Prefer seams that already have a name in the file.
3. **Move a whole declaration at a time**, with its doc comment and its private
   helpers.
4. **Keep the original path working.** Other files and tests import it by path; the
   entry point stays where it was.
5. **Run `flutter analyze` (must be clean) and the tests that cover the file.** Then
   the whole suite before you call it done.

### Which Dart mechanism

**Prefer separate libraries with imports.** A real file with its own imports and a
public API is testable on its own and states its dependencies.

    lib/features/foo/foo_screen.dart          // entry point, still imported by name
    lib/features/foo/widgets/foo_header.dart
    lib/features/foo/widgets/foo_list.dart

**Use `part` / `part of` when — and only when — private members have to stay shared.**
A 1,200-line `State` class whose methods all touch `_files`, `_terminal` and
`setState` cannot be cut into libraries without making a dozen private fields public,
and making them public to satisfy a line count is a worse outcome than the long file
was. `part` moves the text without moving the privacy boundary.

    // lib/features/ide/ide_screen.dart
    part 'ide_screen_editor.dart';
    part 'ide_screen_toolbar.dart';

    // lib/features/ide/ide_screen_editor.dart
    part of 'ide_screen.dart';

Reach for `part` second, not first. When a chunk of a big `State` turns out to touch
only two fields, it wants to be a class with two constructor arguments, not a `part`.

**Never** widen visibility to make a split compile — no `_foo` → `foo` rename, no
`@visibleForTesting` added to dodge a line count. If the only way out is widening, use
`part`.

### Which folder

Folders and subfolders are the point, not a side effect. The tree is what makes forty
small files navigable where one big file was not.

Match the existing feature-first layout (see CLAUDE.md) and extend it downward:

    lib/features/<feature>/
      <feature>_screen.dart          entry point
      <feature>_controller.dart      state / providers
      widgets/                       widgets used only by this feature
      models/                        data types used only by this feature
      <subsystem>/                   a coherent piece with its own vocabulary
                                     (ide/debug/, ide/emulator/, ide/lab/)

- A widget used by one feature goes in that feature's `widgets/`. Only genuinely
  cross-feature widgets belong in `lib/shared/widgets/`.
- When a subsystem grows past ~6 files, give it a subfolder rather than a longer
  prefix on every filename. `ide/emulator/emulator_dock.dart`, not
  `ide/ide_emulator_dock.dart`.
- Tests mirror `lib/` under `test/`, and split the same way:
  `test/features/ide/debug/…` for `lib/features/ide/debug/…`.

### Barrel files

A barrel (`export` only) is the way to keep one import working after a split:

    // lib/features/ide/debug/debug_models.dart
    export 'models/stack_frame.dart';
    export 'models/breakpoint.dart';

Use it when the file was imported by name from several places. Do not add barrels
nobody imports — an unused barrel is a file that exists to be maintained.

## Writing new code

The ceiling applies to the file you are about to create, not just the ones already
here. Two habits keep it from ever mattering:

- When a file passes ~250 lines, the next addition goes in a new file. Deciding at 250
  is a choice about where code belongs; deciding at 310 is a chore.
- A new screen starts as a folder, not a file. `features/foo/foo_screen.dart` plus
  `features/foo/widgets/` from the first commit costs nothing and means the third
  widget has an obvious home.

## Verifying a split

Before calling one done:

    flutter analyze                                   # must print "No issues found!"
    flutter test test/path/to/the_matching_test.dart  # the file's own coverage
    .claude/skills/file-size-limit/check.sh --hard    # nothing over 500

And read the diff as a reviewer would. `git diff` on a pure move should be almost
entirely deletions from one file and identical additions in another. Anything else in
there is either a bug you introduced or a change you owe an explanation for.
