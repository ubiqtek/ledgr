# Table of Contents

- [1. Record architecture decisions](0001-record-architecture-decisions.md)
- [2. Use OFX as the primary Barclays statement import format](0002-use-ofx-for-barclays-statement-import.md)
- [3. Single crate, package `ledgr`](0003-single-crate-package-ledgr.md)
- [4. Use XDG conventions for `ledgr`'s local files, not platform-native dirs](0004-xdg-conventions-for-local-files.md)
- [5. Independent spend and income ledgers, derived from raw transactions](0005-independent-spend-and-income-ledgers.md)
- [6. No account-type gate on spend derivation — scan every account uniformly](0006-no-account-type-gate-on-spend-derivation.md)
- [7. Model assets and liabilities as accounts with (manual or imported) balance snapshots](0007-assets-and-liabilities-as-accounts-with-balance-snapshots.md)
- [8. Reference household accounts — config-only, never imported, no balance data](0008-reference-household-accounts.md)
- [9. Persisted ledgers, built once at import — the UI only ever queries them](0009-persisted-ledgers-built-at-import.md)