#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$ROOT/third_party/treemaker-5.0.1/Source"
BUILD="$ROOT/tools/oracle/build"
OUT="$BUILD/treemaker-oracle"

mkdir -p "$BUILD"

c++ -std=gnu++98 -O2 \
  -I"$SRC" \
  -I"$SRC/tmModel" \
  -I"$SRC/tmModel/tmPtrClasses" \
  -I"$SRC/tmModel/tmOptimizers" \
  -I"$SRC/tmModel/tmSolvers" \
  -I"$SRC/tmModel/tmNLCO" \
  -I"$SRC/tmModel/tmTreeClasses" \
  "$ROOT/tools/oracle/treemaker_oracle.cpp" \
  "$SRC/tmHeader.cpp" \
  "$SRC/tmPrec.cpp" \
  "$SRC/tmModel/tmPtrClasses/tmDpptrTarget.cpp" \
  "$SRC/tmModel/tmNLCO/tmNLCO_alm.cpp" \
  "$SRC/tmModel/tmNLCO/tmNLCO.cpp" \
  "$SRC/tmModel/tmOptimizers/tmConstraintFns.cpp" \
  "$SRC/tmModel/tmOptimizers/tmEdgeOptimizer.cpp" \
  "$SRC/tmModel/tmOptimizers/tmOptimizer.cpp" \
  "$SRC/tmModel/tmOptimizers/tmScaleOptimizer.cpp" \
  "$SRC/tmModel/tmOptimizers/tmStrainOptimizer.cpp" \
  "$SRC/tmModel/tmSolvers/tmStubFinder.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmCluster.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmCondition.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionEdgeLengthFixed.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionEdgesSameStrain.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodeCombo.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodeFixed.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodeOnCorner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodeOnEdge.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodesCollinear.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodesPaired.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionNodeSymmetric.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionPathActive.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionPathAngleFixed.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionPathAngleQuant.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmConditionPathCombo.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmCrease.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmCreaseOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmEdge.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmEdgeOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmFacet.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmFacetOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmNode.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmNodeOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmPart.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmPath.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmPathOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmPoint.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmPoly.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmPolyOwner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmTreeCleaner.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmTree.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmTree_FacetOrder.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmTree_IO.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmTree_TestTrees.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmVertex.cpp" \
  "$SRC/tmModel/tmTreeClasses/tmVertexOwner.cpp" \
  -o "$OUT"

echo "$OUT"
