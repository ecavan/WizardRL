"""Self-play reinforcement learning for Wizard.

The game itself runs in Rust (`wizard_rl._engine`, built from crates/wizard-py); this package
holds the network and the training loop.
"""

from ._engine import (  # noqa: F401
    ACT_BID,
    ACT_CARD,
    ACT_TRUMP,
    ACTIONS,
    FEATURES,
    GAME,
    HAND,
    PHASE,
    ROUND_FEATURES,
    WizardEnv,
    action_name,
    bid_scenario,
    rust_forward,
)
