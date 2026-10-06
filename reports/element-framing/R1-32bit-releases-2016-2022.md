# R1: Revit 2016 to 2022 read with the 32-bit record envelope

**Date:** 2026-10-06
**Branch:** `phase-r/r1-identity` (fork), commits `80eada8`, `45a3e50`
**Status:** positive. The 2016 to 2022 releases are read as the 2023 path reads 2023, with three
corrections to that path that 2023 also gains.
**Answer key:** Autodesk's `rac_basic`, `rst_basic` and `rac_advanced` sample projects, re-saved by
Autodesk in every release (pinned in `research/autodesk-sample-projects.tsv`). Each release's export
is joined on ElementId with the export of the 2024 copy of the same model; ElementIds and the
UniqueId-derived GlobalIds survive the upgrade. This is agreement with an admitted reader of the same
model, not recovery against Revit's own export.
**Scorer:** `tools/re/cross_release_vs_reference.py`.
**Probes:**
- `examples/probe_r1_schema_tags.rs`
- `examples/probe_r1_wall_layers.rs`
- `examples/probe_r1_levels.rs`
- `examples/probe_r1_placement_kinds.rs`
- `examples/probe_r1_level_800333.rs`

## Findings

1. **Tags from the file's own schema.** Several readers search for a header that embeds a class tag.
   - The element-data header's `u16` is the tag of `CellList`.
   - A material object carries the tag of `Material`, and its name ends before `PhysicalParamSet`'s.
   - Read from each release's `Formats/Latest`, these reproduce every hardcoded 2023 to 2026 constant.
   - They supply 2016 to 2022's: `CellList` `0x0255` to `0x028d`, `Material` `0x088b` to `0x09af`.
2. **Layer-count frame tags.** These are field tags, not class tags, so they were measured instead.
   - `rac_basic`'s `SIP 202mm` wall type (428955) frames the same six layers in every release, at the place 2023 frames them with `0x106f`.
   - The tags, 2016 to 2022: `0x0e06`, `0x0e5e`, `0x0eeb`, `0x0f37`, `0x0f76`, `0x0f79`, `0x1003`.
3. **The container reference** of a 32-bit record is a `u32` 16 bytes after the category (22 before the marker), followed by `ff ff ff ff`.
   - The `u64` read at 2024's place relative to the marker is an all-`0xFF` run, so no 32-bit record was a container member.
   - The new field agrees with 2024's container on every shared record:

     | file | shared records | new field agrees | old read misses |
     |---|---|---|---|
     | rac_basic 2016 | 1,020 | all | 9 |
     | rac_basic 2022 | 1,021 | all | 9 |
     | rac_advanced 2018 | 7,412 | all | 67 |
     | rac_advanced 2022 | 7,621 | all | 67 |
4. **Placed Levels on 2016 to 2018** carry placement kind `0xffffef6f`.
   - 2019 and later write `0xffffef7f`.
   - No element record of the 24 files carries `0xffffef6f`.
5. **The Building Story frame** on a 32-bit release is `ff ff ff ff 00 01`, an `f64`, a `u32` ElementId (the Level type, 305 on `rac_basic`) and the setting byte.
   - 2024 writes the `u64` forms.
   - Every Level's setting now equals 2024's on all three models.

## Agreement with the 2024 copy, per ElementId

| model | releases | coverage | class | GlobalId | name | storey | materials | box |
|---|---|---|---|---|---|---|---|---|
| rac_basic | 2016 to 2022 | 100 % (464) | 98.1 | 100 | 44.4 to 47.0 | 81.7 | 61.6 | 94.9 to 95.6 |
| rac_basic | 2023 | 100 % | 98.1 | 100 | 47.4 | 81.7 | 61.9 | 95.6 |
| rst_basic | 2016 to 2018 | 99.8 % | 100 | 100 | 92.7 to 99.1 | 95.1 | 54.5 | 90.7 to 93.6 |
| rst_basic | 2019 to 2023 | 100 % (454) | 100 | 100 | 99.1 | 95.2 | 54.6 | 93.6 |
| rac_advanced | 2016 to 2020 | 98.8 % | 99.4 | 100 | 15.7 to 15.8 | 98.0 | 6.6 | 95.1 |
| rac_advanced | 2021 to 2023 | 100 % (5,657) | 99.4 | 100 | 15.6 | 97.8 | 7.5 | 97.9 |

Before this work, nothing was exported on 2016 to 2022.

On 2023, before findings 3 to 5, storeys agreed at:
- 15.5 % on `rac_basic` (no Levels read);
- 90 % on `rac_advanced`;
- 91.9 % on `rst_basic`.

Exports of Einhoven (2023) and of the 2024 files are byte-identical apart from timestamps.

## Not done, by size of the gap

- **Shapes.**
  - The 32-bit path draws every element as its record box.
  - `rst_basic`'s beams, columns and slabs are real shapes on 2024 (82 % of shared elements) and boxes here (9 %).
- **Names on `rac_advanced`** (15.6 % on 2023 as well).
  - Curtain-wall members and panels, and families whose type the name entries do not reach.
- **Materials on `rac_advanced`** (7.5 %).
  - The category material (`scan_category_material_2023`) still uses 2023's constants.
- **Curtain walls.** Nine curtain walls on `rac_basic` are written as `IfcWall`.
- **Second prologue (RE-30) on 2016 to 2018.**
  - Not examined separately.
  - The records it would hold appear in the coverage above: 98.8 to 100 %.
