#!/usr/bin/env python3
"""Score an rvt-rs export of one release against an export of the same model
saved in another release (the cross-release answer key, RE-140).

Autodesk re-saves its sample projects in every release, and ElementIds survive
the upgrade. An admitted reader's export of the 2024 copy therefore labels
every element of the 2016 to 2023 copies: its IFC class, name, storey,
materials and world box. This scorer joins the two exports on the element
Tag (the ElementId) and reports the agreement per field.

It is agreement with an admitted reader of the same model, not recovery
against Revit's own export, and an element the model gained or lost between
releases counts against coverage either way.

Usage:
    cross_release_vs_reference.py REFERENCE.ifc TEST.ifc [--json OUT.json] [--tol 0.01]
"""
from __future__ import annotations

import argparse
import json
import multiprocessing
import sys
from collections import Counter

import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.util.element as ue

SKIP_CLASSES = ("IfcFeatureElement",)  # openings are scored through their fillings


def elements(f):
    out = {}
    for e in f.by_type("IfcElement"):
        if any(e.is_a(c) for c in SKIP_CLASSES):
            continue
        if e.Tag and e.Tag.isdigit():
            out[int(e.Tag)] = e
    # IFC4 gives a space no Tag; rooms join on their GlobalId, which rvt-rs
    # derives from the Revit UniqueId and so survives an upgrade.
    for s in f.by_type("IfcSpace"):
        out["space:" + s.GlobalId] = s
    return out


def storey(e):
    c = ue.get_container(e)
    while c is not None and not c.is_a("IfcBuildingStorey"):
        c = ue.get_aggregate(c)
    return c.Name if c is not None else None


def materials(e):
    try:
        return tuple(sorted(m.Name or "" for m in ue.get_materials(e)))
    except Exception:
        return ()


def body_source(e):
    """How the element is drawn: its `BodySource`, or, where its plan outline
    is its sketch's (a floor's, roof's or ceiling's), its `ProfileSource`."""
    pset = ue.get_psets(e).get("RvtElementRecordGeometry") or {}
    profile = pset.get("ProfileSource") or ""
    return profile if "sketch" in profile else pset.get("BodySource")


# Body sources that are the element record's box, cut or trimmed: placed and
# sized, but not the element's own shape.
BOX_SOURCES = ("bbox", "box", "join_trimmed", "join_cut", "column_cut")


def is_real_shape(source):
    return bool(source) and not any(marker in source for marker in BOX_SOURCES)


def world_boxes(f, wanted):
    s = ifcopenshell.geom.settings()
    s.set("use-world-coords", True)
    s.set("disable-opening-subtractions", True)
    boxes = {}
    products = [e for e in wanted.values()]
    if not products:
        return boxes
    it = ifcopenshell.geom.iterator(s, f, multiprocessing.cpu_count(), include=products)
    if it.initialize():
        while True:
            sh = it.get()
            v = sh.geometry.verts
            if v:
                xs, ys, zs = v[0::3], v[1::3], v[2::3]
                boxes[sh.id] = (min(xs), min(ys), min(zs), max(xs), max(ys), max(zs))
            if not it.next():
                break
    return boxes


def name_without_id(name):
    if not name:
        return name
    head, sep, tail = name.rpartition(":")
    return head if sep and tail.isdigit() else name


def score(ref_path, test_path, tol):
    ref, test = ifcopenshell.open(ref_path), ifcopenshell.open(test_path)
    r, t = elements(ref), elements(test)
    shared = sorted(set(r) & set(t), key=str)
    rb, tb = world_boxes(ref, {k: r[k] for k in shared}), world_boxes(test, {k: t[k] for k in shared})
    fields = Counter()
    mismatches = {k: [] for k in ("class", "global_id", "name", "storey", "materials", "box")}
    for k in shared:
        a, b = r[k], t[k]
        checks = {
            "class": (a.is_a(), b.is_a()),
            "global_id": (a.GlobalId, b.GlobalId),
            "name": (name_without_id(a.Name), name_without_id(b.Name)),
            "storey": (storey(a), storey(b)),
            "materials": (materials(a), materials(b)),
        }
        for field, (x, y) in checks.items():
            if x == y:
                fields[field] += 1
            else:
                mismatches[field].append({"id": k, "reference": x, "test": y})
        ba, bb = rb.get(a.id()), tb.get(b.id())
        if ba and bb:
            close = max(abs(p - q) for p, q in zip(ba, bb)) <= tol
            fields["box_scored"] += 1
            fields["box"] += close
            # Where both draw the element the same way, the boxes measure
            # the decoder; elsewhere they also measure the choice of body.
            if body_source(a) == body_source(b):
                fields["same_source_scored"] += 1
                fields["same_source_box"] += close
            if not close:
                mismatches["box"].append({"id": k, "class": a.is_a(), "reference": ba, "test": bb,
                                          "reference_source": body_source(a), "test_source": body_source(b)})
    real = lambda els, ids: sum(1 for k in ids if is_real_shape(body_source(els[k])))
    summary = {
        "reference": ref_path,
        "test": test_path,
        "reference_elements": len(r),
        "test_elements": len(t),
        "shared": len(shared),
        "coverage": len(shared) / len(r) if r else 0.0,
        "only_in_reference": len(set(r) - set(t)),
        "only_in_test": len(set(t) - set(r)),
        "only_in_reference_by_class": dict(Counter(r[k].is_a() for k in set(r) - set(t)).most_common()),
        "only_in_test_by_class": dict(Counter(t[k].is_a() for k in set(t) - set(r)).most_common()),
        "agreement": {
            f: (fields[f] / len(shared) if shared else 0.0)
            for f in ("class", "global_id", "name", "storey", "materials")
        },
        "box_agreement": fields["box"] / fields["box_scored"] if fields["box_scored"] else 0.0,
        "same_source_box_agreement": (
            fields["same_source_box"] / fields["same_source_scored"] if fields["same_source_scored"] else 0.0
        ),
        "same_source_scored": fields["same_source_scored"],
        "reference_storeys": sorted({s.Name for s in ref.by_type("IfcBuildingStorey")}),
        "test_storeys": sorted({s.Name for s in test.by_type("IfcBuildingStorey")}),
        "real_shape_share": {
            "reference": real(r, shared) / len(shared) if shared else 0.0,
            "test": real(t, shared) / len(shared) if shared else 0.0,
        },
    }
    return summary, mismatches


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("reference")
    ap.add_argument("test")
    ap.add_argument("--json", help="write summary and mismatches here")
    ap.add_argument("--tol", type=float, default=0.01, help="box tolerance in model units (default 0.01)")
    args = ap.parse_args()
    summary, mismatches = score(args.reference, args.test, args.tol)
    a = summary["agreement"]
    print(
        f"coverage {summary['coverage']:.1%} ({summary['shared']}/{summary['reference_elements']}, "
        f"+{summary['only_in_test']} extra) | class {a['class']:.1%} name {a['name']:.1%} "
        f"gid {a['global_id']:.1%} storey {a['storey']:.1%} materials {a['materials']:.1%} "
        f"box {summary['box_agreement']:.1%} (same body {summary['same_source_box_agreement']:.1%} "
        f"of {summary['same_source_scored']}) | "
        f"real shape {summary['real_shape_share']['test']:.1%} vs {summary['real_shape_share']['reference']:.1%}"
    )
    if args.json:
        with open(args.json, "w") as fh:
            json.dump({"summary": summary, "mismatches": mismatches}, fh, indent=1, default=str)
    return 0


if __name__ == "__main__":
    sys.exit(main())
