#!/usr/bin/env python3
"""
Validate Product Framework artifacts.

Usage:
  python validate.py                       # validate the bundled examples
  python validate.py path/to/product.ttl   # SHACL-validate a product graph
  python validate.py path/to/file.yaml --as layout|task-type|work-unit|delivery

Requires: rdflib, pyshacl, jsonschema, pyyaml
"""
import sys, json, argparse, pathlib
HERE = pathlib.Path(__file__).parent

JSON_SCHEMAS = {
    "layout":       HERE / "json" / "layout-model.schema.json",
    "task-type":    HERE / "json" / "task-type-definition.schema.json",
    "work-unit":    HERE / "json" / "work-unit.schema.json",
    "delivery":     HERE / "json" / "delivery.schema.json",
    "how-contract": HERE / "json" / "how-contract.schema.json",
}
ONTOLOGY = HERE / "ontology" / "product-framework.ttl"
SHAPE_FILES = sorted((HERE / "shapes").glob("*.ttl"))


def _load_shapes():
    import rdflib
    g = rdflib.Graph()
    for f in SHAPE_FILES:
        g.parse(f, format="turtle")
    return g


def validate_graph(ttl_path):
    import rdflib
    from pyshacl import validate
    data = rdflib.Graph(); data.parse(ttl_path, format="turtle")
    shapes = _load_shapes()
    ont = rdflib.Graph(); ont.parse(ONTOLOGY, format="turtle")
    conforms, _, text = validate(data, shacl_graph=shapes, ont_graph=ont,
                                 inference="rdfs", advanced=True)
    print(f"[graph] {ttl_path}: {'CONFORMS' if conforms else 'NON-CONFORMANT'}")
    if not conforms:
        print(text)
    return conforms


def validate_json(path, kind):
    import yaml
    from jsonschema import Draft202012Validator
    schema = json.load(open(JSON_SCHEMAS[kind]))
    inst = yaml.safe_load(open(path))
    errs = sorted(Draft202012Validator(schema).iter_errors(inst), key=lambda e: str(e.path))
    ok = not errs
    print(f"[{kind}] {path}: {'VALID' if ok else 'INVALID'}")
    for e in errs:
        print(f"   - {list(e.path)}: {e.message}")
    return ok


def validate_examples():
    ex = HERE / "examples"
    results = []
    results.append(validate_graph(ex / "todo-product.ttl"))
    results.append(validate_json(ex / "layout-model.example.yaml", "layout"))
    results.append(validate_json(ex / "how-contract.example.yaml", "how-contract"))
    results.append(validate_json(ex / "task-type-definition.example.yaml", "task-type"))
    results.append(validate_json(ex / "work-unit.example.yaml", "work-unit"))
    results.append(validate_json(ex / "delivery.example.yaml", "delivery"))
    return all(results)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("path", nargs="?", help="file to validate; omit to run bundled examples")
    ap.add_argument("--as", dest="kind", choices=list(JSON_SCHEMAS),
                    help="treat the file as this artifact kind (for .yaml/.json)")
    args = ap.parse_args()

    if not args.path:
        ok = validate_examples()
    elif args.kind:
        ok = validate_json(args.path, args.kind)
    elif str(args.path).endswith(".ttl"):
        ok = validate_graph(args.path)
    else:
        sys.exit("For YAML/JSON, pass --as layout|task-type|work-unit|delivery")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
