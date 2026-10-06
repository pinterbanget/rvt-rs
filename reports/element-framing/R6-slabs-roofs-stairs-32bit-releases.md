# R6: floors, roofs, ceilings and stairs for Revit 2016 to 2022

**Date:** 2026-10-06
**Branch:** `phase-r/r1-identity` (fork)
**Status:** positive.
- On 2016 to 2023, floors, building pads, roofs and ceilings are drawn from their sketches, and straight stair runs from their treads and risers, as 2024's decoders draw them.
- Every outline and run both exports draw agrees with the 2024 copy, except where Autodesk edited the model after 2020.
- Real shapes are level with the 2024 copy on `rac_basic` and `rst_basic`, and above it on `rac_advanced` from 2021.

**Answer key:** as in R1. Autodesk's three sample projects of each release are joined on ElementId with the 2024 copy's export. Where the 32-bit route draws more than 2024 does, the extra shapes are checked against Revit's own saved graphics of the 2024 file.
**Scorer:** `tools/re/cross_release_vs_reference.py`. It now counts a body over a sketched outline (`ProfileSource` `..._sketch_lines`) as a real shape. Before, floors' outlines counted as record boxes on both sides, because their `BodySource` stays the record box.
**Probes:**
- `examples/probe_r6_sketch_owner.rs`, `probe_r6_no_outline.rs`, `probe_r6_frames.rs`, `probe_r6_record_refs.rs`
- `examples/probe_r6_sketch_curves.rs`, `probe_r6_copies.rs`
- `examples/probe_r6_native_fields.rs`, `probe_r6_graph_status.rs`, `probe_r6_refusals.rs`
- `examples/probe_r6_run_types.rs`, `probe_r6_element_fields.rs`, `probe_r6_mesh_extent.rs`

## Findings

1. **A sketch line's owner.**
   - On a 32-bit record, as on 2024 (RE-25), the second counted list after the box ends with the element whose sketch the line is.
   - Joined with the 2024 copies, it names 2024's owner on every shared outlined sketch line of the 13 files probed: 2,723 records.
2. **Records framed without an outline.**
   - Where an outlined record's marker holds the tag of `Outline` (`0x0129` on 2022), some records hold `00 00`. Their ElementId is 50 bytes before the marker, not 52, and their two lists follow the marker with no box between.
   - Category, container, placement kind and class tag sit where an outlined record has them.
   - The four files probed frame 6,670 to 8,441 records each this way, and none of their ids is also framed with an outline.
   - Among them are sketch lines: 41 of `rac_basic`'s, 465 of `rac_advanced`'s and 260 of `rst_basic`'s on 2022. Their 2024 copies are outlined; the upgrade wrote their boxes.
   - Floor 495352 of `rac_basic` has one outlined line and three without an outline.
3. **Stale boxes.** A 32-bit sketch line's record box can lag its curve:
   - floor 647908's bottom edge on `rst_basic` 2022 lies 0.26 mm off its box;
   - roof 724430's four lines on `rac_basic` 2022 lie 4.6 ft above theirs.

   Their curves are their 2024 copies', and so are the 2024 copies' boxes. The 2024 solver tests each line's ends against its box, so on 32-bit releases every sketch line is boxed by its own curve.
4. **Curves and sketch lists from the native graph.**
   - A sketch line's `CurveElem` holds its curve as one `GLine` or one `GArc`. Its `SketchMembership.m_groupId` names its sketch (`VarSketch`).
   - The sketch's `m_userId` names the sketched element, and its `m_curveObjIdxMap` lists the sketch's curves. This is the list RE-97 reads from a 2024 Sketch's bytes. It matters for edited sketches: `rac_basic` floor 523064 has 3 lines naming it and 8 in its sketch's list.
   - The native reader takes the copy of an element that the element table names. A byte scan meets every copy, and drops one whose copies disagree. On `rac_advanced` 2021, sketch line 147478 is stored in Partitions/7 and /8 with different ends (Autodesk moved that wall in 2021), so the byte reader dropped it and its ceiling.
5. **Revit 2016 and 2017's `VarParam`.**
   - These releases declare `VarParam.m_pSubst`, a pointer with schema modifier 4 and no class reference. 2018 drops the field.
   - The native decoder refused every `VarSketch` holding one. A null one (a zero `u32` token) is now read as a null pointer; a non-null one is still refused.
   - No file of the corpus holds a non-null one. `rac_advanced` 2016 decodes all 17,099 element graphs (`probe_r6_refusals`).
6. **Stairs from the native graph.**
   - RE-47 and RE-52 read 2024's stairs at fixed byte offsets. The native graph holds the same values by name:
     - a stair's `StairsElement`: `m_actualRiserHeight`, `m_actualTreadDepth`, `m_actualNumberOfRisers`;
     - a run's `StairsRun`: `m_stairsId`, riser indices, `m_endWithRiser` (RE-92), and its boundary and riser lines as `GLine`s, in the order RE-52's `run_sketch` reads them from 2024's bytes;
     - a run type's `StairsRunType`: `m_structuralDepth`, `m_treadThickness`, `m_riserThickness`, `m_nosingLength`, `m_hasMonolithicSupport`, `m_hasTread`, `m_hasRiser`, `m_riserIsSlanted`, `m_riserTreadConnection`.
   - On the 2024 copies of the three samples and Snowdon Towers, all 20 run types' native fields equal what RE-52 reads from their bytes. `m_riserTreadConnection` 1 is RE-52's tread under the riser.
   - A run's type is the one `StairsRunType` its record's reference list names, as on 2024. A run's riser count follows RE-47's rule: a stair's only run has the stair's; several runs have their riser-index differences where those add up.
7. **Doors in curtain walls** (a fix to R3).
   - The 32-bit route bound doors and windows to their host before it knew which walls are curtain walls. Five doors of `rac_basic` and 17 of `rac_advanced` were cut into curtain walls, which Revit's export never does (#439).
   - They are now bound after curtain walls are attached, as on 2024. Doors and windows filling an opening: 26 of 26 and 107 of 107, as 2024.

## Results against the 2024 copy

### Outlines

Elements both exports draw from their sketch, and how many agree in volume (0.1 %) and world box (3 mm):

| model | releases | floors | roofs | ceilings | agree |
|---|---|---|---|---|---|
| rac_basic | 2016 to 2023 | 10 of 10 | 1 of 1 | 2 of 2 | all |
| rst_basic | 2016 to 2023 | 5 of 5 | - | - | all |
| rac_advanced | 2021 to 2023 | 17 of 17 | 4 of 4 | 65 of 65 | all |
| rac_advanced | 2016 to 2020 | 17 of 17 | 4 of 4 | 65 of 65 | 72 of 86 |

On `rac_advanced` 2016 to 2020, the 14 that differ are two floors and twelve ceilings along one wall. Each reaches 7.5 mm further north than on 2021 and later.
- The 2016 sketch line 147478 starts at y = 100.0297 ft, and the 2021 and 2024 one at 100.0050 ft.
- This is the model edit R3 found (same-body box agreement 97 % before 2021).

Before R6 the 32-bit route drew no outline; 2024 draws these 104.

### Stair runs

| model | releases | runs both draw | agree | runs only the 32-bit route draws |
|---|---|---|---|---|
| rst_basic | 2016 to 2023 | 1 | 1 | 0 |
| rac_advanced | 2021 to 2023 | 5 | 5 | 5 |
| rac_basic | 2016 to 2023 | 0 | - | 0 |

- **rac_advanced, the five extra runs.** These are the lower flights, which start at elevation 0.
  - 2024's RE-92 byte reader takes run 354390 to neither begin nor end with a riser, so it expects 14 riser lines where the sketch has 13, and leaves the run a box (`examples/probe_r6_run_lines.rs`).
  - Its native `m_beginWithRiser` and `m_endWithRiser` are both true, on 2022 and on 2024.
  - Revit's own saved meshes on the 2024 file have the same world box as the 32-bit route's bodies, and volumes within 2.6 % (0.2350 against 0.2291 m³).
- **rac_basic.** Its runs stay boxes on both sides:
  - one is an L-shaped run, which RE-52's `run_sketch` does not draw;
  - two are monolithic runs without slanted risers, a construction RE-92 did not measure.
- **rac_advanced 2016 to 2020.** These have no runs to compare: the model's stairs before 2021 are other elements (172987 and on), with no separate runs.

### Wall profile openings

The six openings 2024 cuts in `rac_basic`'s walls from their edited elevation profiles (B55) are cut on 2016 and 2022 with the same boxes. The 32-bit route cuts one more, a 10 cm strip at the west end of wall 977133.
- 2024's byte reader finds no sketch list for that wall; the native graph does.
- Revit's saved mesh of the wall starts at x = 23.169 ft (7.062 m), where the cut wall starts. The uncut one would start at 6.961 m.

### Default export

Real shapes now count sketched outlines (see Scorer).

| model | releases | real shape (test / 2024) | box | same-body box |
|---|---|---|---|---|
| rac_basic | 2016 to 2017 | 8.2 / 8.2 % | 99.3 to 99.6 % | 99.3 to 99.6 % of 452 |
| rac_basic | 2018 to 2023 | 8.2 / 8.2 % | 100 % | 100 % of 452 |
| rst_basic | 2016 | 81.7 / 81.7 % | 96.7 % | 97.1 % of 446 |
| rst_basic | 2017 to 2023 | 81.5 to 81.7 / 81.5 to 81.7 % | 99.6 % | 100 % of 446 to 447 |
| rac_advanced | 2016 to 2020 | 4.3 / 4.2 % | 97.1 % | 97.1 % of 5,548 |
| rac_advanced | 2021 | 4.1 / 4.2 % | 99.6 % | 99.9 % of 5,592 |
| rac_advanced | 2022 to 2023 | 4.4 / 4.2 % | 99.9 % | 100 % of 5,607 |

The 32-bit route draws more of `rac_advanced` than 2024:
- the five stair runs (2021 to 2023);
- six W250X49.1 columns as I-sections (R5's `native_i_sections`), where 2024 draws boxes. Revit's saved meshes give the same world boxes, and volumes within 4 % (the root fillets are not drawn).

On 2021 it draws less: 15 walls stay boxes (see Not done).

Exports of 2024 files are byte-identical apart from timestamps: Autodesk's three samples, Snowdon Towers Structural and Plumbing, and Core Interior with and without `--saved-meshes`.

## Not done

- **Walls of `rac_advanced` 2021.**
  - 15 walls stay join-trimmed boxes. Revit 2021 stores them in Partitions/7 (their place before the 2021 edit) and in /8, and the byte readers of their location line and orientation drop an element whose copies disagree.
  - The fix is the one finding 4 makes for sketch lines: read the copy the element table names, or let the newest partition's copy win. That changes what the 2024 route reads, so it needs its own check against the 2024 answer keys.
- **Roof slopes (RE-56) and shaft openings (RE-99)** are not opened to the 32-bit route. None of the three samples has a shed roof or a shaft that 2024 draws.
- **L-shaped and winder runs, and monolithic runs without slanted risers**, which 2024 does not draw either.
