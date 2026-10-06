# R3: shapes for Revit 2016 to 2022

**Date:** 2026-10-06
**Branch:** `phase-r/r1-identity` (fork), commit `a1a3e2c`
**Status:** positive. On 2016 to 2022, the 32-bit route now draws real shapes from three decoders the 2024 route already had. The share of real shapes is level with the 2024 copy, except for column I-sections.
**Answer key:** as in R1. Autodesk's three sample projects of each release are joined on ElementId with the 2024 copy's export. Scored with `tools/re/cross_release_vs_reference.py`, which now also reports box agreement where both exports draw an element the same way ("same-body").
**Probes:**
- `examples/probe_r3_saved_scene.rs`
- `examples/probe_r3_instance_info.rs`
- `examples/probe_r3_group_headers.rs`
- `examples/probe_r3_wall_lines.rs`

## Findings

1. **Saved graphics (`--saved-meshes`).** The native record framing (`native_document`) reads every 32-bit release with 4-byte ids. Two release differences:
   - **Group body bytes.** Revit 2016 and 2017 count each record's trailing `u32` length in a group's declared body bytes. Every group of their samples reads out exactly, with declared bytes exceeding the bodies by 4 per record (848 records: 131,004 against 127,612). 2018 and later count the bodies alone.
   - **`m_GRepId`.** `InstanceInfo` has no `m_GRepId` field before 2023: the 2022 schema writes only `m_Trf`, `m_cda` and `m_symbolId`. A missing field is read as the default representation. With it required to be 0, every external symbol of 2022 was refused.

   With both, `rac_basic` draws the same 349 of 394 box bodies from saved meshes on every release from 2016 to 2023, the same 738,019 triangles on each. Before, 2022 drew 37 and 2016 and 2017 none.
2. **Beam axes (RE-49).** The bounded line in a framing element's data is the same record behind the 32-bit element-data header.
   - On the 32-bit route the axis is attached before RE-122's column cuts, which now apply only to a beam without a line.
   - `rst_basic` 2016 to 2022 draw 345 beams along their line, as 2024 does; before, 339 were boxes and 31 column-cut boxes.
3. **Wall centrelines.** The 32-bit route kept walls' record boxes (RE-114); it now draws them as 2024 does.
   - Revit 2021 inserted the u32 "word" between a wall's location line and its flip, in the release that brought slanted and tapered walls. On all 56 walls of `rac_basic` the 2020 and 2021 bytes match four bytes apart.
   - 2016 to 2020 are read without the word, every wall vertical (word 1). Read as 2021's, 11 of the 56 orientations failed to parse and the others read word 0.
   - `rac_basic` 2016 to 2022 now draw 18 walls from their centreline (2024: 25). Of the 40 walls drawn as 2024 draws them, 38 agree with 2024's box within 0.01 ft.

## Default export against the 2024 copy

"Real shape" counts a body that is not the record's box, cut or trimmed: walls along their centreline, beams along their axis, saved meshes. Record boxes trimmed at joins or cut at columns count as boxes.

| model | releases | real shape (test / 2024) | box | same-body box |
|---|---|---|---|---|
| rac_basic | 2016 to 2017 | 3.9 / 5.4 % | 97.6 to 97.8 % | 98.8 to 99.0 % of 414 |
| rac_basic | 2018 to 2023 | 3.9 / 5.4 % | 98.2 % | 99.5 % of 414 |
| rst_basic | 2016 to 2023 | 76.4 to 76.6 / 80.4 to 80.6 % | 96.5 to 99.3 % | 97.0 to 100 % of 428 to 429 |
| rac_advanced | 2016 to 2020 | 2.2 / 2.6 % | 96.5 % | 96.9 % of 5,370 |
| rac_advanced | 2021 to 2023 | 2.2 / 2.7 % | 99.1 to 99.4 % | 99.9 % of 5,414 to 5,429 |

Before R3, real shapes were 0 % on every 32-bit release, and box agreement 93.6 to 97.9 %.

On `rac_advanced`, same-body agreement is lower before 2021. RE-140 found the same split (89.6 against 99.8 % box agreement), most likely Autodesk's own edits to the model in 2021.

Exports of 2024 files are byte-identical apart from timestamps, with or without `--saved-meshes`.

## With `--saved-meshes`

*(To be filled from `rvt-corpus/out-r3-sm2`.)*

## Not done

- **Column I-sections** (`rst_basic` 2024: 17 columns). `attach_beam_sections` reads the type's section from 2024's type records, which the 32-bit releases do not write in that form.
- **Turned and oriented family-instance boxes** (`rac_basic` 2024: 31; `rac_advanced`: 148). `scan_instance_transforms` reads a 64-bit data header. With `--saved-meshes` most of these are meshes anyway.
- **Seven walls 2024 draws from their centreline stay record boxes**, and two of the 40 compared walls differ.
- **Slabs, roofs and stairs** are boxes on both sides on these models; the sketch readers were not opened to the 32-bit route.
