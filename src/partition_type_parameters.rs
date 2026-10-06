//! Text parameters of element types, stored under their BuiltInParameter
//! (#35, RE-77).
//!
//! An element's parameter values sit in value blocks that open with the
//! element's ElementId followed by 56 bytes of `0xff` and 3 zero bytes
//! (Discussion #112). Within a type's block, some text parameters are
//! stored as `i64 BuiltInParameter · u32 n · UTF-16 × n`. An entry belongs to
//! the block it follows, up to the next block, and the block's ElementId
//! must be declared in `Global/ElemTable`. The partition record chain
//! (RE-35) cannot attribute these: it covers only each partition's leading
//! records, while most type records sit in the loaded families' documents
//! after it (4% of Snowdon Towers' inflated partitions are in the chain).
//!
//! Measured on Autodesk's Snowdon Towers (Revit 2024) against the VIM export
//! of its 2027 edition, per type that both hold (see
//! `reports/element-framing/RE-77-type-text-parameters.md`). Instance
//! parameters such as Mark and Comments are not stored this way and are not
//! read. A type whose blocks carry two different values for one parameter
//! gets neither (fail closed).
//!
//! A type's length parameters sit in the same blocks as `f64 feet · ff × 8 ·
//! i64 parameter` (RE-93): a BuiltInParameter's negative id, or the declared
//! ElementId of a family parameter. [`type_window_openings`] and
//! [`type_door_openings`] read the values a window's and a door's opening
//! are built from, and [`type_i_sections`] a steel framing type's I section
//! (RE-103).

use crate::RevitFile;
use crate::partition_element_records::bbox_marker;
use std::collections::{BTreeMap, BTreeSet};

/// The type parameters read, by BuiltInParameter id, with the name Revit
/// shows for each.
pub const TYPE_TEXT_PARAMETERS: &[(i64, &str)] = &[
    (-1001405, "Type Mark"),
    (-1010103, "Description"),
    (-1001206, "Fire Rating"),
];

/// Longest value read, in UTF-16 units.
const MAX_VALUE_UNITS: usize = 400;

/// The bytes that follow a `u32` ElementId to open its value block on a
/// 32-bit release (2016 to 2023): half the run, as a 2023 Level's name block
/// halves its own (RE-107). Measured on Autodesk's `rst_basic` 2022, whose
/// section types (1042289 and the rest) open their blocks so.
const VALUE_BLOCK_MARK_32: [u8; 31] = {
    let mut mark = [0xffu8; 31];
    mark[28] = 0;
    mark[29] = 0;
    mark[30] = 0;
    mark
};

/// The bytes that follow an ElementId to open its value block.
const VALUE_BLOCK_MARK: [u8; 59] = {
    let mut mark = [0xffu8; 59];
    mark[56] = 0;
    mark[57] = 0;
    mark[58] = 0;
    mark
};

/// Every type's text parameters in one inflated partition, by the declared
/// ElementId of the value block each entry follows.
pub fn scan_partition(
    buf: &[u8],
    declared_ids: &BTreeSet<u32>,
) -> BTreeMap<u32, BTreeMap<&'static str, String>> {
    let blocks = value_blocks(buf, declared_ids);
    let mut values: BTreeMap<(u32, &'static str), Option<String>> = BTreeMap::new();
    for (parameter, name) in TYPE_TEXT_PARAMETERS {
        for at in memchr::memmem::find_iter(buf, &parameter.to_le_bytes()) {
            let Some(text) = text_at(buf, at + 8) else {
                continue;
            };
            let index = blocks.partition_point(|(start, _)| *start < at);
            let Some(&(_, Some(owner))) = index.checked_sub(1).and_then(|i| blocks.get(i)) else {
                continue;
            };
            values
                .entry((owner, name))
                .and_modify(|seen| {
                    if seen.as_deref() != Some(text.as_str()) {
                        *seen = None;
                    }
                })
                .or_insert(Some(text));
        }
    }
    let mut out: BTreeMap<u32, BTreeMap<&'static str, String>> = BTreeMap::new();
    for ((owner, name), value) in values {
        if let Some(value) = value {
            out.entry(owner).or_default().insert(name, value);
        }
    }
    out
}

/// Every type's text parameters in the file, by type ElementId. Empty for a
/// release whose element records are not decoded.
pub fn type_text_parameters(
    rf: &mut RevitFile,
    revit_version: u32,
) -> BTreeMap<u32, BTreeMap<&'static str, String>> {
    if bbox_marker(revit_version).is_none() {
        return BTreeMap::new();
    }
    let Ok(records) = crate::elem_table::parse_records(rf) else {
        return BTreeMap::new();
    };
    let declared = crate::elem_table::declared_ids(&records);
    let mut out: BTreeMap<u32, BTreeMap<&'static str, String>> = BTreeMap::new();
    for stream in rf.partition_stream_names() {
        let Ok(inflated) = rf.inflated_partition(&stream) else {
            continue;
        };
        for (owner, parameters) in scan_partition(inflated.bytes(), &declared) {
            out.entry(owner).or_default().extend(parameters);
        }
    }
    out
}

/// A `u32`-counted UTF-16 string at `at` of printable characters.
fn text_at(buf: &[u8], at: usize) -> Option<String> {
    let len = u32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?) as usize;
    if !(1..=MAX_VALUE_UNITS).contains(&len) {
        return None;
    }
    let bytes = buf.get(at + 4..at + 4 + len * 2)?;
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let text = String::from_utf16(&units).ok()?;
    text.chars().all(|c| !c.is_control()).then_some(text)
}

/// A window type's Width and Height, by BuiltInParameter (RE-93).
pub const WINDOW_WIDTH_PARAMETER: i64 = -1001301;
/// See [`WINDOW_WIDTH_PARAMETER`].
pub const WINDOW_HEIGHT_PARAMETER: i64 = -1001300;
/// The name of the window family parameter holding a type's default sill
/// height (RE-93). It is a family parameter, not a BuiltInParameter, so it
/// is found by its name, as Revit's English templates name it.
pub const DEFAULT_SILL_HEIGHT_NAME: &str = "Default Sill Height";
/// From a family parameter's ElementId (after `01 00 00 00`) to its name,
/// `u32 n · UTF-16 × n`, which a `u32`-counted `revit.local.family:` id
/// follows (RE-93).
pub const PARAMETER_NAME_OFFSET: usize = 0x56;
const FAMILY_PARAMETER_ID_PREFIX: &str = "revit.local.family:";
/// Largest length value read, feet.
const MAX_LENGTH_FEET: f64 = 1000.0;

/// The declared family parameters in `buf` named `name` (RE-93).
pub fn family_parameters_named(
    buf: &[u8],
    declared_ids: &BTreeSet<u32>,
    name: &str,
) -> BTreeSet<u32> {
    let units: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let Ok(count) = u32::try_from(name.encode_utf16().count()) else {
        return BTreeSet::new();
    };
    let mut pattern = count.to_le_bytes().to_vec();
    pattern.extend_from_slice(&units);
    memchr::memmem::find_iter(buf, &pattern)
        .filter(|&at| {
            text_at(buf, at + pattern.len())
                .is_some_and(|id| id.starts_with(FAMILY_PARAMETER_ID_PREFIX))
        })
        .filter_map(|at| {
            let id_at = at.checked_sub(PARAMETER_NAME_OFFSET)?;
            if buf.get(id_at.checked_sub(4)?..id_at)? != [1, 0, 0, 0] {
                return None;
            }
            let id = u64::from_le_bytes(buf.get(id_at..id_at + 8)?.try_into().ok()?);
            u32::try_from(id)
                .ok()
                .filter(|id| declared_ids.contains(id))
        })
        .collect()
}

/// The length values in `buf` of each parameter in `parameters`, by the
/// declared ElementId of the value block each entry follows. A block that
/// holds two values for one parameter gets neither.
pub fn scan_partition_lengths(
    buf: &[u8],
    declared_ids: &BTreeSet<u32>,
    parameters: &BTreeSet<i64>,
) -> BTreeMap<u32, BTreeMap<i64, f64>> {
    scan_partition_lengths_sized(buf, declared_ids, parameters, false)
}

/// [`scan_partition_lengths`] on a release whose ids are `u32`s (`narrow`,
/// 2016 to 2023): an entry is `f64 feet · ff × 4 · i32 parameter` in a
/// 32-bit value block (`u32` ElementId, 28 × `0xff`, 3 zero bytes). Measured on `rst_basic` 2022, whose section
/// types hold the same Width, Height and thicknesses as their 2024 copies
/// (43, 43, 37 and 37 entries on each).
pub fn scan_partition_lengths_sized(
    buf: &[u8],
    declared_ids: &BTreeSet<u32>,
    parameters: &BTreeSet<i64>,
    narrow: bool,
) -> BTreeMap<u32, BTreeMap<i64, f64>> {
    let blocks = if narrow {
        value_blocks_32(buf, declared_ids)
    } else {
        value_blocks(buf, declared_ids)
    };
    let gap = if narrow { 4 } else { 8 };
    let mut values: BTreeMap<(u32, i64), Option<f64>> = BTreeMap::new();
    for &parameter in parameters {
        let needle = if narrow {
            let Ok(narrow_id) = i32::try_from(parameter) else {
                continue;
            };
            narrow_id.to_le_bytes().to_vec()
        } else {
            parameter.to_le_bytes().to_vec()
        };
        for at in memchr::memmem::find_iter(buf, &needle) {
            let Some(value) = at
                .checked_sub(8 + gap)
                .and_then(|start| buf.get(start..at))
                .filter(|entry| entry[8..].iter().all(|&b| b == 0xff))
                .map(|entry| f64::from_le_bytes(entry[..8].try_into().expect("8 bytes")))
                .filter(|value| value.is_finite() && value.abs() <= MAX_LENGTH_FEET)
            else {
                continue;
            };
            let index = blocks.partition_point(|(start, _)| *start < at);
            let Some(&(_, Some(owner))) = index.checked_sub(1).and_then(|i| blocks.get(i)) else {
                continue;
            };
            values
                .entry((owner, parameter))
                .and_modify(|seen| {
                    if *seen != Some(value) {
                        *seen = None;
                    }
                })
                .or_insert(Some(value));
        }
    }
    let mut out: BTreeMap<u32, BTreeMap<i64, f64>> = BTreeMap::new();
    for ((owner, parameter), value) in values {
        if let Some(value) = value {
            out.entry(owner).or_default().insert(parameter, value);
        }
    }
    out
}

/// What a window's opening is built from, from its type (RE-93), feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowTypeOpening {
    /// The type's Width.
    pub width_feet: f64,
    /// The type's Height.
    pub height_feet: f64,
    /// The type's Default Sill Height.
    pub default_sill_feet: f64,
}

/// The opening of each type in `types` whose Width, Height and Default Sill
/// Height are read, and agree across every partition (RE-93). Empty for a
/// release whose element records are not decoded.
pub fn type_window_openings(
    rf: &mut RevitFile,
    revit_version: u32,
    types: &BTreeSet<u32>,
) -> BTreeMap<u32, WindowTypeOpening> {
    if bbox_marker(revit_version).is_none() || types.is_empty() {
        return BTreeMap::new();
    }
    let Ok(records) = crate::elem_table::parse_records(rf) else {
        return BTreeMap::new();
    };
    let declared = crate::elem_table::declared_ids(&records);
    let streams = rf.partition_stream_names();
    let mut sills: BTreeSet<i64> = BTreeSet::new();
    for stream in &streams {
        if let Ok(inflated) = rf.inflated_partition(stream) {
            sills.extend(
                family_parameters_named(inflated.bytes(), &declared, DEFAULT_SILL_HEIGHT_NAME)
                    .into_iter()
                    .map(i64::from),
            );
        }
    }
    let mut parameters = sills.clone();
    parameters.extend([WINDOW_WIDTH_PARAMETER, WINDOW_HEIGHT_PARAMETER]);
    type_lengths(rf, &declared, types, &parameters)
        .into_iter()
        .filter_map(|(owner, held)| {
            let value = |parameter: i64| held.get(&parameter).copied();
            let mut sill = sills.iter().filter_map(|&parameter| value(parameter));
            let default_sill_feet = sill.next()?;
            if sill.any(|other| other != default_sill_feet) {
                return None;
            }
            let opening = WindowTypeOpening {
                width_feet: value(WINDOW_WIDTH_PARAMETER)?,
                height_feet: value(WINDOW_HEIGHT_PARAMETER)?,
                default_sill_feet,
            };
            (opening.width_feet > 0.0
                && opening.height_feet > 0.0
                && opening.default_sill_feet >= 0.0)
                .then_some((owner, opening))
        })
        .collect()
}

/// A door type's Rough Width, by BuiltInParameter (RE-94).
pub const DOOR_ROUGH_WIDTH_PARAMETER: i64 = -1001305;
/// A door type's Rough Height, by BuiltInParameter (RE-94).
pub const DOOR_ROUGH_HEIGHT_PARAMETER: i64 = -1001304;

/// A door type's rough opening (RE-94), feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoorTypeOpening {
    /// The type's Rough Width.
    pub rough_width_feet: f64,
    /// The type's Rough Height.
    pub rough_height_feet: f64,
}

/// The rough opening of each type in `types` whose Rough Width and Rough
/// Height are read, and agree across every partition (RE-94). Empty for a
/// release whose element records are not decoded.
pub fn type_door_openings(
    rf: &mut RevitFile,
    revit_version: u32,
    types: &BTreeSet<u32>,
) -> BTreeMap<u32, DoorTypeOpening> {
    if bbox_marker(revit_version).is_none() || types.is_empty() {
        return BTreeMap::new();
    }
    let Ok(records) = crate::elem_table::parse_records(rf) else {
        return BTreeMap::new();
    };
    let declared = crate::elem_table::declared_ids(&records);
    let parameters = BTreeSet::from([DOOR_ROUGH_WIDTH_PARAMETER, DOOR_ROUGH_HEIGHT_PARAMETER]);
    type_lengths(rf, &declared, types, &parameters)
        .into_iter()
        .filter_map(|(owner, held)| {
            let opening = DoorTypeOpening {
                rough_width_feet: *held.get(&DOOR_ROUGH_WIDTH_PARAMETER)?,
                rough_height_feet: *held.get(&DOOR_ROUGH_HEIGHT_PARAMETER)?,
            };
            (opening.rough_width_feet > 0.0 && opening.rough_height_feet > 0.0)
                .then_some((owner, opening))
        })
        .collect()
}

/// A structural section type's Width, by BuiltInParameter (RE-103).
pub const SECTION_WIDTH_PARAMETER: i64 = -1_005_502;
/// A structural section type's Height (RE-103).
pub const SECTION_HEIGHT_PARAMETER: i64 = -1_005_503;
/// An I-shaped section type's web thickness (RE-103).
pub const SECTION_WEB_THICKNESS_PARAMETER: i64 = -1_005_525;
/// An I-shaped section type's flange thickness (RE-103).
pub const SECTION_FLANGE_THICKNESS_PARAMETER: i64 = -1_005_524;
/// A section type's centroid, from its left edge (RE-103).
pub const SECTION_CENTROID_HORIZONTAL_PARAMETER: i64 = -1_005_508;
/// A section type's centroid, from its bottom edge (RE-103).
pub const SECTION_CENTROID_VERTICAL_PARAMETER: i64 = -1_005_509;

/// How closely an I section's centroid must sit at its centre, feet.
pub const SECTION_CENTROID_TOLERANCE_FEET: f64 = 1e-4;

/// An I-shaped structural section (RE-103), feet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ISection {
    /// The flanges' width.
    pub width_feet: f64,
    /// The section's overall depth.
    pub depth_feet: f64,
    /// The web's thickness.
    pub web_feet: f64,
    /// Each flange's thickness.
    pub flange_feet: f64,
}

/// The I section of each type in `types` (RE-103): a type whose Width,
/// Height, web thickness and flange thickness are read and whose centroid
/// sits at its centre both ways, as only a doubly symmetric section's does.
/// A channel or a tee stores the same four values with its centroid off
/// centre, and an angle with neither thickness, so neither is read as an I.
pub fn type_i_sections(
    rf: &mut RevitFile,
    revit_version: u32,
    types: &BTreeSet<u32>,
) -> BTreeMap<u32, ISection> {
    let release_read = bbox_marker(revit_version).is_some()
        || crate::partition_element_records_2023::reads_release(revit_version);
    if !release_read || types.is_empty() {
        return BTreeMap::new();
    }
    let Ok(records) = crate::elem_table::parse_records(rf) else {
        return BTreeMap::new();
    };
    let declared = crate::elem_table::declared_ids(&records);
    let parameters = BTreeSet::from([
        SECTION_WIDTH_PARAMETER,
        SECTION_HEIGHT_PARAMETER,
        SECTION_WEB_THICKNESS_PARAMETER,
        SECTION_FLANGE_THICKNESS_PARAMETER,
        SECTION_CENTROID_HORIZONTAL_PARAMETER,
        SECTION_CENTROID_VERTICAL_PARAMETER,
    ]);
    type_lengths(rf, &declared, types, &parameters)
        .into_iter()
        .filter_map(|(owner, held)| {
            let value = |parameter: i64| held.get(&parameter).copied();
            let section = ISection {
                width_feet: value(SECTION_WIDTH_PARAMETER)?,
                depth_feet: value(SECTION_HEIGHT_PARAMETER)?,
                web_feet: value(SECTION_WEB_THICKNESS_PARAMETER)?,
                flange_feet: value(SECTION_FLANGE_THICKNESS_PARAMETER)?,
            };
            let centred = |centroid: f64, extent: f64| {
                (centroid - extent / 2.0).abs() <= SECTION_CENTROID_TOLERANCE_FEET
            };
            (section.web_feet > 0.0
                && section.flange_feet > 0.0
                && section.web_feet < section.width_feet
                && 2.0 * section.flange_feet < section.depth_feet
                && centred(
                    value(SECTION_CENTROID_HORIZONTAL_PARAMETER)?,
                    section.width_feet,
                )
                && centred(
                    value(SECTION_CENTROID_VERTICAL_PARAMETER)?,
                    section.depth_feet,
                ))
            .then_some((owner, section))
        })
        .collect()
}

/// The values of `parameters` held by each type in `types`, where every
/// partition's value agrees.
fn type_lengths(
    rf: &mut RevitFile,
    declared: &BTreeSet<u32>,
    types: &BTreeSet<u32>,
    parameters: &BTreeSet<i64>,
) -> BTreeMap<u32, BTreeMap<i64, f64>> {
    let narrow = rf
        .basic_file_info()
        .is_ok_and(|b| crate::partition_element_records_2023::reads_release(b.version));
    let mut values: BTreeMap<u32, BTreeMap<i64, Option<f64>>> = BTreeMap::new();
    for stream in rf.partition_stream_names() {
        let Ok(inflated) = rf.inflated_partition(&stream) else {
            continue;
        };
        for (owner, found) in
            scan_partition_lengths_sized(inflated.bytes(), declared, parameters, narrow)
        {
            if !types.contains(&owner) {
                continue;
            }
            let held = values.entry(owner).or_default();
            for (parameter, value) in found {
                held.entry(parameter)
                    .and_modify(|seen| {
                        if *seen != Some(value) {
                            *seen = None;
                        }
                    })
                    .or_insert(Some(value));
            }
        }
    }
    values
        .into_iter()
        .map(|(owner, held)| {
            let agreed = held
                .into_iter()
                .filter_map(|(parameter, value)| Some((parameter, value?)))
                .collect();
            (owner, agreed)
        })
        .collect()
}

/// Value block starts in `buf`: the mark, not preceded by a further 0xff (a
/// longer run of 0xff is padding, not a block), with a declared ElementId
/// before it, the block's owner. A mark with 0 before it also starts a
/// block, owned by element 0 only when the table declares one: it ends the
/// block before it either way. (Until #152 every 2024 and later file
/// declared a false id 0, read from the bytes after its ElemTable, and the
/// blocks were cut there; without the cut, RE1 Architecture's curtain wall
/// type took the next block's `Type Mark`.)
/// [`value_blocks`] on a 32-bit release: a `u32` ElementId and
/// [`VALUE_BLOCK_MARK_32`].
fn value_blocks_32(buf: &[u8], declared_ids: &BTreeSet<u32>) -> Vec<(usize, Option<u32>)> {
    memchr::memmem::find_iter(buf, &VALUE_BLOCK_MARK_32)
        .filter(|&at| at >= 4 && buf[at - 1] != 0xff)
        .filter_map(|at| {
            let id = u32::from_le_bytes(buf[at - 4..at].try_into().ok()?);
            if declared_ids.contains(&id) {
                Some((at, Some(id)))
            } else {
                (id == 0).then_some((at, None))
            }
        })
        .collect()
}

fn value_blocks(buf: &[u8], declared_ids: &BTreeSet<u32>) -> Vec<(usize, Option<u32>)> {
    memchr::memmem::find_iter(buf, &VALUE_BLOCK_MARK)
        .filter(|&at| at >= 8 && buf[at - 1] != 0xff)
        .filter_map(|at| {
            let id = u64::from_le_bytes(buf[at - 8..at].try_into().ok()?);
            let id = u32::try_from(id).ok()?;
            if declared_ids.contains(&id) {
                Some((at, Some(id)))
            } else {
                (id == 0).then_some((at, None))
            }
        })
        .collect()
}

/// Family-parameter captions of a steel section's width, depth, web and
/// flange thickness, as a steel family names its own parameters where Revit
/// has no section BuiltInParameters (Revit 2016; they first appear in 2017):
/// `bf`, `d`, `tw` and `tf` on Autodesk's `rst_basic` 2016, whose
/// `200UB25.4` type holds the values its 2017 copy holds under
/// [`SECTION_WIDTH_PARAMETER`] and the rest.
const SECTION_CAPTIONS: [(&str, usize); 4] = [("bf", 0), ("d", 1), ("tw", 2), ("tf", 3)];

/// Whether a type's own solid (its native graph's `Plane` faces) is
/// symmetric about its centre across the extrusion both ways: every side
/// face, projected onto the section plane, has its mirror image across each
/// axis. An I section's are; a channel's, tee's or angle's are not.
/// `None` where the graph holds no side faces along one axis.
pub fn native_section_doubly_symmetric(
    graph: &crate::native_parameters::ObjectGraph,
) -> Option<bool> {
    let vec3 = |value: &serde_json::Value| -> Option<[f64; 3]> {
        let a = value.as_array()?;
        Some([
            a.first()?.as_f64()?,
            a.get(1)?.as_f64()?,
            a.get(2)?.as_f64()?,
        ])
    };
    // Each face: origin, u axis, v axis, u range.
    let mut faces = Vec::new();
    for object in graph.objects.iter().filter(|o| o.class_name == "Plane") {
        let f = &object.fields;
        let (Some(o), Some(x), Some(y)) =
            (vec3(&f["m_origin"]), vec3(&f["m_xVec"]), vec3(&f["m_yVec"]))
        else {
            continue;
        };
        let corners = f["m_Envelope"]["m_corners"].as_array()?;
        let u0 = corners.first()?.get(0)?.as_f64()?;
        let u1 = corners.get(1)?.get(0)?.as_f64()?;
        faces.push((o, x, y, u0, u1));
    }
    // The extrusion axis: the coordinate axis most faces run along in v.
    let mut votes = [0usize; 3];
    for (_, _, y, _, _) in &faces {
        if let Some(axis) = (0..3).find(|&a| y[a].abs() > 0.999) {
            votes[axis] += 1;
        }
    }
    let axis = (0..3).max_by_key(|&a| votes[a])?;
    let (i, j) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let segments: Vec<[[f64; 2]; 2]> = faces
        .iter()
        .filter(|(_, x, y, _, _)| y[axis].abs() > 0.999 && x[axis].abs() < 1e-6)
        .map(|(o, x, _, u0, u1)| {
            let at = |u: f64| [o[i] + u * x[i], o[j] + u * x[j]];
            [at(*u0), at(*u1)]
        })
        .collect();
    if segments.len() < 4 {
        return None;
    }
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in segments.iter().flatten() {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    const TOLERANCE: f64 = 1e-4;
    let same = |a: [f64; 2], b: [f64; 2]| {
        (a[0] - b[0]).abs() < TOLERANCE && (a[1] - b[1]).abs() < TOLERANCE
    };
    let has = |s: [[f64; 2]; 2]| {
        segments.iter().any(|t| {
            (same(s[0], t[0]) && same(s[1], t[1])) || (same(s[0], t[1]) && same(s[1], t[0]))
        })
    };
    let mirrored = |k: usize| {
        segments.iter().all(|s| {
            let m = |p: [f64; 2]| {
                let mut q = p;
                q[k] = 2.0 * centre[k] - p[k];
                q
            };
            has([m(s[0]), m(s[1])])
        })
    };
    Some(mirrored(0) && mirrored(1))
}

/// The I section of each type in `types` from the native object graph, for
/// a release whose section types hold their sizes as family parameters
/// rather than BuiltInParameters (Revit 2016): the values of the
/// `FamilyParams` entries whose parameter element (`ParamElemFamily`) is
/// captioned `bf`, `d`, `tw` and `tf`, where the type's own solid is
/// doubly symmetric ([`native_section_doubly_symmetric`]), as the
/// centroid test of [`type_i_sections`] requires on later releases.
pub fn native_i_sections(rf: &mut RevitFile, types: &BTreeSet<u32>) -> BTreeMap<u32, ISection> {
    let graphs = |rf: &mut RevitFile, ids: BTreeSet<u64>| {
        let mut out = BTreeMap::new();
        if ids.is_empty() {
            return out;
        }
        let options = crate::native_document::Options {
            selected_ids: ids,
            channels: BTreeSet::from([102]),
            max_graph_values: 1_000_000,
            max_graph_objects: 1_000_000,
            ..Default::default()
        };
        let _ = crate::native_document::extract(rf, &options, |record| {
            if let (Ok(id), Some(graph)) = (u32::try_from(record.identity.element_id), record.graph)
            {
                out.insert(id, graph);
            }
            Ok(())
        });
        out
    };
    let type_graphs = graphs(rf, types.iter().map(|&t| u64::from(t)).collect());
    // Each type's family-parameter values, by parameter ElementId.
    let mut values: BTreeMap<u32, BTreeMap<u32, f64>> = BTreeMap::new();
    for (&type_id, graph) in &type_graphs {
        for object in graph
            .objects
            .iter()
            .filter(|o| o.class_name == "FamilyParams")
        {
            for entry in object.fields["m_params"].as_array().into_iter().flatten() {
                let (Ok(parameter), Some(value)) = (
                    crate::native_metadata::identifier(&entry["m_paramId"]),
                    entry["m_value"].as_f64(),
                ) else {
                    continue;
                };
                if let Ok(parameter) = u32::try_from(parameter) {
                    values.entry(type_id).or_default().insert(parameter, value);
                }
            }
        }
    }
    let parameters: BTreeSet<u64> = values
        .values()
        .flat_map(|held| held.keys())
        .map(|&p| u64::from(p))
        .collect();
    let caption: BTreeMap<u32, String> = graphs(rf, parameters)
        .iter()
        .filter_map(|(&id, graph)| {
            let definition = graph
                .objects
                .iter()
                .find(|o| o.class_name == "ParamDefValue")?;
            Some((id, definition.fields["m_caption"].as_str()?.to_string()))
        })
        .collect();
    let mut out = BTreeMap::new();
    for (type_id, held) in values {
        let mut sizes = [None; 4];
        for (parameter, value) in held {
            if let Some(&(_, slot)) = caption
                .get(&parameter)
                .and_then(|c| SECTION_CAPTIONS.iter().find(|(name, _)| name == c))
            {
                sizes[slot] = Some(value);
            }
        }
        let [
            Some(width_feet),
            Some(depth_feet),
            Some(web_feet),
            Some(flange_feet),
        ] = sizes
        else {
            continue;
        };
        let section = ISection {
            width_feet,
            depth_feet,
            web_feet,
            flange_feet,
        };
        let symmetric = type_graphs
            .get(&type_id)
            .and_then(native_section_doubly_symmetric)
            == Some(true);
        if symmetric
            && section.web_feet > 0.0
            && section.flange_feet > 0.0
            && section.web_feet < section.width_feet
            && 2.0 * section.flange_feet < section.depth_feet
        {
            out.insert(type_id, section);
        }
    }
    out
}
