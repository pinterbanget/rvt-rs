# R3: shapes for Revit 2016 to 2022

**Date:** 2026-10-06
**Branch:** `phase-r/r1-identity` (fork), commits `a1a3e2c`, `ece482f`, `f4175e3`, `45547bb`
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

4. **Turned family instances and curtain members.** `partition_instance_transforms` reads the 32-bit data header and `u32` id.
   - The 31 turned instances of `rac_basic` read the same transform on 2016, 2022 and 2024 (`examples/probe_r3_transforms.rs`).
   - The 32-bit route now attaches each record's Revit class (RE-76), which `attach_instance_axes` keys on, and curtain members' axes (RE-106).
   - On 2016 and 2022, `rac_basic` draws its 31 turned instances turned, and `rac_advanced` its 148 oriented curtain members and 3 turned instances, as 2024 does.
5. **I sections from type parameters.**
   - On a 32-bit release a type's value block opens with a `u32` ElementId, 28 × `0xff` and 3 zero bytes, and a length entry is `f64 · ff × 4 · i32 parameter`.
   - `rst_basic` 2022's section types hold the same Width, Height and thicknesses as their 2024 copies (43, 43, 37 and 37 entries on each).
   - Its 17 I-section columns are drawn on 2017 to 2022.
   - Revit 2016 writes none of the structural section BuiltInParameters; they first appear in 2017. So 2016's columns keep their box.

6. **Every wall as 2024 draws it** (`f4175e3`).
   - **Join lists.** On a 32-bit release a wall's join lists put a `u32` and 13 zero bytes before the count (at +29 from the opening word), and hold `u32 · u32 ElementId · u32` entries. Wall 198694 of `rac_basic` 2016 and 2022 holds the same lists as its 2024 copy. Read with the 32-bit header, butt joins (RE-70, RE-127) resolve.
   - **Wall types.** A 32-bit wall's type is no longer picked from among the Levels and materials its record names. Level 1, which no window names, kept two walls from their type, and a cladding material that only walls name took a third's.
   - **Curtain walls.** `record_references` reads a 32-bit record's `u32` list, and the 32-bit route attaches curtain walls (RE-46).
   - **Result.** On 2016 and 2022 every wall and curtain wall that both exports hold agrees with 2024 in class, body source and box: 47 of 47 on `rac_basic`, 146 of 146 on `rac_advanced`.

7. **Revit 2016's steel sections.** 2016 writes no section BuiltInParameters.
   - Its section types hold the same sizes as family parameters whose `ParamElemFamily` captions are `bf`, `d`, `tw` and `tf`.
   - `native_i_sections` reads them from the native graph. In place of the centroid test it requires the type's own solid to be doubly symmetric (`native_section_doubly_symmetric`).
   - On `rst_basic` 2016 it picks exactly the two types 2017's centroid test reads as I sections. The 17 columns are drawn as on 2017 to 2024, each within 0.01 ft of 2024's box.
   - `rst_basic` 2016's real shapes are 80.4 % against 2024's 80.6 %, and storeys 100 %.

## Default export against the 2024 copy

"Real shape" counts a body that is not the record's box, cut or trimmed: walls along their centreline, beams along their axis, saved meshes. Record boxes trimmed at joins or cut at columns count as boxes.

As of `f4175e3`:

| model | releases | class | real shape (test / 2024) | box | same-body box |
|---|---|---|---|---|---|
| rac_basic | 2016 to 2017 | 100 % | 5.4 / 5.4 % | 99.3 to 99.6 % | 99.3 to 99.6 % of 452 |
| rac_basic | 2018 to 2023 | 100 % | 5.4 / 5.4 % | 100 % | 100 % of 452 |
| rst_basic | 2016 | 100 % | 80.4 / 80.6 % | 96.5 % | 97.1 % of 445 |
| rst_basic | 2017 to 2023 | 100 % | 80.2 to 80.4 / 80.4 to 80.6 % | 99.3 % | 100 % of 445 to 446 |
| rac_advanced | 2016 to 2020 | 100 % | 2.6 / 2.6 % | 97.1 % | 97.1 % of 5,554 |
| rac_advanced | 2021 to 2023 | 100 % | 2.3 to 2.6 / 2.7 % | 99.6 to 99.9 % | 99.9 to 100 % of 5,598 to 5,613 |

Storeys agree on 99.4 to 100 % (`rst_basic` 2016: 95.4 %).

Before R3, real shapes were 0 % on every 32-bit release, and box agreement 93.6 to 97.9 %.

On `rac_advanced`, same-body agreement is lower before 2021. RE-140 found the same split (89.6 against 99.8 % box agreement), most likely Autodesk's own edits to the model in 2021.

Exports of 2024 files are byte-identical apart from timestamps, with or without `--saved-meshes`.

## With `--saved-meshes`

As of `ece482f`, before the wall work of `f4175e3`:

| model | releases | real shape (test / 2024) | box | same-body box |
|---|---|---|---|---|
| rac_basic | 2016 to 2023 | 76.5 / 74.6 % | 97.8 to 98.2 % | 99.1 to 99.5 % of 432 |
| rst_basic | 2016 | 93.4 / 94.3 % | 96.7 % | 96.9 % of 423 |
| rst_basic | 2017 to 2023 | 95.6 / 94.3 % | 99.6 % | 100 % of 440 to 441 |
| rac_advanced | 2016 to 2020 | 92.4 / 91.4 % | 96.5 % | 96.9 % of 5,435 |
| rac_advanced | 2021 to 2023 | 92.2 to 92.4 / 91.2 % | 99.1 to 99.3 % | 99.9 % of 5,479 to 5,494 |

The 32-bit exports draw slightly more elements from saved meshes than 2024's do. Where 2024 has a decoder of its own for an element, it uses that before the mesh; the 32-bit route lacks some of those decoders.

## Not done

- **Slabs, roofs and stairs** are boxes on both sides on these models; the sketch readers were not opened to the 32-bit route.
