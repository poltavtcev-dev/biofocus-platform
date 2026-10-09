# Wearables v2 fixtures (synthetic)

Invented HealthKit samples and the Observations ADR-030 expects after mapping.
These files are **not** a copy of anyone's Apple Health export.

`data-summary.md` was not in the export folder. The brief's aggregate is the reason
these cases exist: a date-ascending sync never reaches recent days, Xiaomi apps
do not write HRV or sleep stages, and a deletion must stay a marker.

| File | What it locks |
| :--- | :--- |
| `apple_watch.json` | Watch kinds, HRV `method: sdnn`, sleep stages, SpO2, respiratory rate, wrist temperature, workout |
| `xiaomi_mi_fitness.json` | Band via Mi Fitness: HR, steps, energy, coarse sleep. No HRV, no stages |
| `zepp_life.json` | Older band via Zepp Life: HR, steps, `in_bed` / `asleep` only. No HRV |
| `late_write_and_deletion.json` | Sample whose start is **before** a date anchor, plus `source_deletion` aimed at a Watch sample |
| `legacy_payloads.json` | Shapes already stored (`hrv` without `method`, sleep `asleep`, HR without `src`) |

Bundle → `src.kind` (also in ADR-030):

| bundle_id | kind |
| :--- | :--- |
| `com.xiaomi.wearable` | `xiaomi_mi_fitness` |
| `com.huami.watch`, `com.xiaomi.hm.health` | `zepp_life` |
| Apple Health + `product_type` `Watch…` | `apple_watch` |
| Apple Health + `product_type` `iPhone…` | `iphone` |
| `was_user_entered: true` | `manual` |

No `source.name` and no `HKDevice.name` anywhere in these files.
