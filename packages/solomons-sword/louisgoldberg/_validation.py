"""Runtime checks for the facts supplied to the trust calculations."""


def validate_boolean_facts(**facts: object) -> None:
    """Leave unknown facts for each calculation to handle; refuse other types."""
    for name, value in facts.items():
        if value is not None and type(value) is not bool:
            raise ValueError(f"{name} must be True, False or None")
