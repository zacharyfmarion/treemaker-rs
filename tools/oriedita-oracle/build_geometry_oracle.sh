#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
oracle_root="$repo_root/tools/oriedita-oracle"
source_root="${ORIEDITA_SOURCE:-$repo_root/third_party/oriedita}"
origami_source="$source_root/origami/src/main/java"
build_root="$oracle_root/build"
classes_root="$build_root/classes"

if [[ ! -d "$origami_source" ]]; then
  echo "Oriedita source not found at $origami_source" >&2
  echo "Set ORIEDITA_SOURCE to the pinned Oriedita checkout." >&2
  exit 1
fi

rm -rf "$classes_root"
mkdir -p "$classes_root"

javac \
  -d "$classes_root" \
  -sourcepath "$oracle_root/stubs:$oracle_root/src:$origami_source" \
  "$oracle_root/stubs/origami/crease_pattern/FoldLineSet.java" \
  "$origami_source/origami/Epsilon.java" \
  "$origami_source/origami/crease_pattern/CustomLineTypes.java" \
  "$origami_source/origami/crease_pattern/element/Point.java" \
  "$origami_source/origami/crease_pattern/element/LineColor.java" \
  "$origami_source/origami/crease_pattern/element/LineSegment.java" \
  "$origami_source/origami/crease_pattern/element/StraightLine.java" \
  "$origami_source/origami/crease_pattern/element/Circle.java" \
  "$origami_source/origami/crease_pattern/OritaCalc.java" \
  "$oracle_root/src/OrieditaGeometryOracle.java"

cat > "$build_root/oriedita-geometry-oracle" <<EOF
#!/usr/bin/env bash
set -euo pipefail
java -cp "$classes_root" OrieditaGeometryOracle "\$@"
EOF
chmod +x "$build_root/oriedita-geometry-oracle"

echo "$build_root/oriedita-geometry-oracle"
