"""questkit: tools for writing, testing and debugging the fork's quests. Run as `python3 tools/questkit <command>`;
`python3 tools/questkit --help` lists the commands. See docs/src/fork/questkit.md."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from questkit.cli import main  # noqa: E402

if __name__ == "__main__":
    sys.exit(main())
