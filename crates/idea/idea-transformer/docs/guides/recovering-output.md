# Recover an interrupted installation

An interrupted native install may leave a complete old tree, a complete new tree, or retained
stage/backup state. Keep those bytes until ownership has been established. Recovery chooses
among validated trees; it does not infer authority from a path name or a PID.

## Inspect before recovery

For a root named `generated`, inspect its fixed same-parent siblings:

```text
generated/
.generated.stackpress-lock
.generated.stackpress-stage/
.generated.stackpress-backup/
.generated.stackpress-journal
```

The lock records a writer PID for investigation only. Establish that no writer remains before
explicitly removing a stale process-death lock. Do not remove an active writer's lock or delete
a backup just to make the next run pass. Normal check/install calls refuse retained recovery state.

## Invoke the recovery API

Once inspection and any stale-lock removal are complete, call `OutputWorkspace::recover` with
the same destination. This function can be used by an app-owned maintenance command:

```rust
use stackpress_idea_transformer::{OutputWorkspace, Result};
use std::path::Path;

fn recover_output(root: &Path) -> Result<()> {
    OutputWorkspace::new(root)?.recover()
}
```

The function acquires its own cooperative lock and requires a recognized journal. A healthy
workspace with no journal returns an error; this is not an unconditional cleanup command.
All retained candidates are validated before any recovery rename or purge.

## Interpret the result

Recovery keeps a valid installed root. If the destination is absent, it restores the complete
backup; on a first install with no predecessor, it can install the complete stage. Recognized
cleanup intent requires an installed valid root and can resume partial backup deletion while
rejecting unknown files. A partial backup never becomes the destination.

A complete stage left before recognized intent, malformed/unknown journals, missing recovery
trees, manual files, edited owned bytes, symlinks, and special files require operator attention.
The API does not guess whether an unrecognized stage was authorized. Random temporary siblings
left by process death also remain outside automatic recovery.

After recovery, rerun generation from source and check freshness. A restored old tree can be
validly owned and still stale for the new plan. There is no crash-atomic or power-loss guarantee,
and all writers must cooperate with exclusive ownership. See
[OutputWorkspace](../api/output-workspace.md) for returns and error behavior.
