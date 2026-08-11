# Dev → QA: P15-E2-T1

**Task:** Core SDNN-or-RMSSD + iOS Auto-sync  
**Role:** Dev  
**Date:** 2026-08-11  
**Branch:** `phase/15-companion-hrv-autonomy`

## Done
### Core
- `pipeline::normalize_hrv` accepts `rmssd_ms` OR `sdnn_ms`.
- `feature-engine` `mean_hrv_ms` (prefer rmssd → sdnn) used by Focus / Recovery; Stress prefers single ms map.
- Tests: `hrv_sdnn_only_is_kept`, `sdnn_only_emits_recovery`, `sdnn_only_fills_hrv_focus_slot`.

### iOS
- `HrvSample.swift`, `ObservationQueue.swift`, `HealthKitSyncCoordinator.swift`
- Auto-sync toggle, observers, background delivery entitlement, flush queue
- Manual send latest HR+HRV; calm when HRV missing

## Verify
```bash
cargo test -p pipeline normalize
cargo test -p feature-engine --lib
# optional:
cd apps/companion/ios && … swiftc check (see README)
```

## AC checklist
- [x] SDNN-only Observations normalize
- [x] Features emit on SDNN-only HRV
- [x] Queue + flush; no busy-loop
- [x] Background delivery entitlement
- [x] Manual send remains
