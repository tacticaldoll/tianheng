## Context

The gate currently runs `mktemp -d` and installs its EXIT trap on the following line. The two commands create an
ordering gap: cleanup ownership begins after resource acquisition rather than before it.

## Goals / Non-Goals

**Goals:**

- Make cleanup ownership precede signature-directory acquisition.
- Prove cleanup on a partial acquisition whose command returns failure after reporting the created path.
- Preserve every normal publish-source verdict and diagnostic.

**Non-Goals:**

- Change how the signature is extracted, reconstructed, or verified.
- Introduce a shared temporary-resource framework for unrelated scripts without evidence that their lifecycles
  are the same.

## Decisions

### Preinstall an inert EXIT cleanup

Initialize the path to empty, install a cleanup function that acts only on a non-empty path, and only then assign
the result of `mktemp -d`. Bash retains command-substitution output in the assignment even when the substitution
returns non-zero, so an allocator that reports a created directory and then fails still leaves the path available
to the already-installed EXIT trap.

This keeps ownership local to the one resource. A broader trap stack would add infrastructure without another
live consumer or a composition requirement.

### Test ordering through partial acquisition

The failure matrix supplies a PATH-local `mktemp` that creates the requested directory, prints it, and exits
non-zero. Before the change, the script exits before installing cleanup and the directory remains. After the
change, the preinstalled trap removes it. This deterministically observes the ordering guarantee without trying
to win a signal race between adjacent shell commands.

## Risks / Trade-offs

- Recursive cleanup is safe only when the path is non-empty and comes from the allocator; the cleanup function
  therefore refuses to act on the empty initial state and passes the path as one quoted operand.
- EXIT cleanup does not promise recovery from SIGKILL or machine loss; those are outside the shell's observable
  cleanup perimeter.
