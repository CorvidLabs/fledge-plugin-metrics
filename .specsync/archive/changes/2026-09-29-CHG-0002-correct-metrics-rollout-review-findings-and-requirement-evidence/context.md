---
change: CHG-0002-correct-metrics-rollout-review-findings-and-requirement-evidence
artifact: context
---

# Context

Review found generated agent commands that could lose user input, governance files omitted from meaningful-path enforcement, and an acceptance criterion that overstated LOC integration coverage. This change corrects those migration artifacts without changing Rust product code. The existing `specsync scaffold` command is retained because SpecSync 5.0.1 documents and implements companion-file generation.
