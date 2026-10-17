"""Install the fast-check worker's locked npm dependencies.

Uses the Node and npm of the nodejs-wheel-binaries pin in the dev extra, never a
global installation. `npm ci` installs exactly package-lock.json, verifies every
integrity hash and, with --ignore-scripts, runs no package's install scripts.
Run it after installing the dev extra and again whenever the lock changes.
"""

import sys
from pathlib import Path

import nodejs_wheel

sys.exit(nodejs_wheel.npm(["ci", "--ignore-scripts"], cwd=Path(__file__).parent))
