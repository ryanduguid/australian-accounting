# v0.1.4

Changes since `v0.1.3`:

- pin `ato-benchmark-compare==0.1.4`. The pinned 0.1.3 shipped `__version__ = "0.1.2"`, so `engine_version` on every benchmark response understated the engine that produced the figures. The engine is now single-sourced upstream and reports 0.1.4;
- read `aus_accounting_mcp.__version__` from the installed distribution metadata instead of repeating the number in the package, so the module attribute cannot drift from `pyproject.toml` the way the engine's did; and
- add a test asserting each quoted `engine_version` equals the installed version of the engine that produced it.

No tool behaviour, statutory logic or refusal changed. Division 7A is still refused and SBR payloads are still synthetic.

Not advice. Outputs are preparation aids for a qualified professional, not compliance determinations.
