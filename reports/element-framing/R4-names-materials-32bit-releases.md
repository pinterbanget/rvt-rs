# R4: names and materials for Revit 2016 to 2022

**Date:** 2026-10-06
**Branch:** `phase-r/r1-identity` (fork), commit `7564336`
**Status:** positive.
- Names agree with the 2024 copy of each model on 98 to 99.7 % of elements.
- The material differences that remain are, with two exceptions, the 2024 export's own.

**Answer key:** as in R1, and scored with `tools/re/cross_release_vs_reference.py`. For materials, the 2024 export turned out not to be a reliable key; see section 3.
**Probes:**
- `examples/probe_r4_native_objects.rs`, which prints an element's native graph objects or finds a value in them;
- `examples/probe_r4_curtain_materials.rs`.

## 1. The native object graph on the 32-bit releases

`native_document` decodes an element's serialized objects by its file's own schema. R3 made it read 2016 to 2022. It decodes the same objects there as on 2024, with `m_id` where 2024 writes `m_id64`:

| element | class | what it holds |
|---|---|---|
| mullion 423134 | `FamilyInstance` | `m_masterSymbolId` 232770; `InstanceInfo.m_symbolId` 462990, a per-instance symbol with no name |
| type 232770 | `SysMullionFamSym` | `SymbolInfo.m_name` "64 x 128 rectangular", `m_familyId` 8482, `m_renderStyleId` 232859 |
| family 8482 | `Family` | `m_name` "Rectangular Mullion" |
| material 232859 | `Material` | `m_name` "Metal - Paint Finish - Dark Gray, Matte" |
| floor 176804 | `Floor` | `m_floorAttributesId` 339, `SymbolInfo.m_name` "Generic 150mm" |
| railing 566588 | `BaseRailing` | `m_stairsRailingAttrId` 490129, `SymbolInfo.m_name` "SH_1100mm" |
| column type 140902 | `FamilySymbol` | `m_geomTag2MaterialId` [(25, 140876)], identical on 2022 and 2024 |

## 2. Names

An element without a type name is named from the native graph:
- its type, from the field its own object names it in (`m_masterSymbolId`, `m_typeId`, or a field ending in `AttributesId` or `AttrId`, and `InstanceInfo.m_symbolId` only where none is set);
- the type's `SymbolInfo.m_name`;
- its family's `Family.m_name`, through `m_familyId`. A system type names no family and keeps RE-63's.

| model | releases | names, before | names, after |
|---|---|---|---|
| rac_basic | 2016 / 2022 | 47.8 / 50.4 % | 98.3 / 98.7 % |
| rac_advanced | 2016 / 2022 | 16.3 / 16.2 % | 99.7 / 98.8 % |
| rst_basic | 2016 / 2022 | 92.7 / 99.1 % | 92.7 / 99.1 % |

What remains is a handful of building pads, stair runs and stringers, and wall sweeps, which 2024 names by rules of their own (`Assembled Stair:Stair:<id> Run 1`, a sweep's bare id). On `rst_basic` 2016, 29 family names differ because Autodesk renamed those families after 2016 ("CHS-Circular Hollow Sections (AS 1163)" against "... (AS 1163_C250L0)").

## 3. Materials

An element with a type and no material from RE-113's map takes its type's materials from the native graph:
- a curtain mullion's or panel's type: its `m_renderStyleId`, the ElementId RE-166 reads 135 bytes into a 2024 `SysMullionFamSym`;
- any other type: the materials of its `m_geomTag2MaterialId`, RE-82's map.

Each material is named by its `Material.m_name`.

Against the 2024 export, element by element (2022; 2016 alike):

| model | agree | we lack a named material 2024 has | we have one where 2024 has none or `<Unnamed>` | both named, differ |
|---|---|---|---|---|
| rac_basic | 389 of 464 | 0 | 73 | 2 |
| rst_basic | 244 of 454 | 0 | 39 | 0 |
| rac_advanced | 425 of 5,657 | 23 | 3 | 5,174 |

The differences are the 2024 export's:
- **rac_advanced, "Wood - Laminated".**
  - All 5,174 differing elements have it in their 2024 materials.
  - 4,648 are curtain mullions and panels the 2024 export gives "Wood - Laminated" alone. Its own RE-166 reader (`scan_curtain_type_materials` on the 2024 file) gives their types "Metal - Aluminium, Black-Anodized" and "Metal - Aluminium", as this route does.
  - The other 526 have exactly our materials once "Wood - Laminated" is dropped.
  - A later step of the 2024 route overrides RE-166's material there.
- **rac_basic and rst_basic, our extra materials.**
  - The materials we add where 2024 has none are their types' own: the "Glass" of 44 system panels, the aluminium of light fixtures, a porcelain bowl, a chrome faucet, the concrete of footings and pile caps.
  - On `rst_basic`, 27 wood open-web joists get "Metal Stud Layer" and "Softwood, Lumber", where 2024 writes `<Unnamed>`.
- **The 23 we lack.**
  - 22 railings and a covering on `rac_advanced`.
  - 2024 gives the "Guardrail - Pipe" railings "Site - Sand". Their rails' `m_materialId` is -1, by category, so that material is itself doubtful.

## Not done

- **Stair-part and wall-sweep names**, which follow 2024's naming rules rather than the type.
- **A railing's material by category** (object styles); `scan_category_material_2023` still uses 2023's constants.
