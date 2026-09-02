"""CLI entrypoint for aus-accounting-mcp."""

from aus_accounting_mcp.server import run_stdio


def main() -> None:
    run_stdio()


if __name__ == "__main__":
    main()
