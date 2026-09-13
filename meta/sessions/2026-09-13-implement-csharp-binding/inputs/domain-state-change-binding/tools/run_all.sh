#!/bin/sh
# Every check this binding ships. Positive fixtures must pass; negative
# fixtures must fail — an instrument that has never returned an adverse result
# has untested discriminating power.
set -e
cd "$(dirname "$0")/.."
echo
echo "  POSITIVE"
python3 tools/validate.py examples/place-order.determinations.yaml >/dev/null && echo "    schema        5/5 ordering records valid"
python3 tools/validate.py examples/fulfilment.determinations.yaml >/dev/null && echo "    schema        2/2 fulfilment records valid"
python3 tools/prove_prohibitions.py >/dev/null && echo "    prohibitions  7/7 forbidden shapes rejected"
python3 tools/check_resolution.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml >/dev/null && echo "    resolution    ordering resolved"
python3 tools/check_resolution.py examples/fulfilment.eventmodel.yaml examples/fulfilment.determinations.yaml >/dev/null && echo "    resolution    fulfilment resolved"
python3 tools/check_composition.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml examples/fulfilment.eventmodel.yaml examples/fulfilment.determinations.yaml >/dev/null && echo "    composition   seam agrees"
python3 tools/check_conformance.py conformance/manifest.yaml >/dev/null && echo "    criterion     11/11"
echo
echo "  NEGATIVE (each must fail)"
if python3 tools/check_resolution.py examples/broken.eventmodel.yaml examples/place-order.determinations.yaml >/dev/null 2>&1; then
  echo "    resolution    UNEXPECTED PASS"; exit 1
else echo "    resolution    dangling references caught"; fi
if python3 tools/check_composition.py examples/ordering.eventmodel.yaml examples/place-order.determinations.yaml examples/seam-defect/fulfilment.eventmodel.yaml examples/seam-defect/fulfilment.determinations.yaml >/dev/null 2>&1; then
  echo "    composition   UNEXPECTED PASS"; exit 1
else echo "    composition   seam defect caught"; fi
echo
