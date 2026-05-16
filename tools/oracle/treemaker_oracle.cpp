/*******************************************************************************
TreeMaker Rust port oracle harness.

This executable links only the vendored TreeMaker 5.0.1 model sources and emits
stable JSON lines for fixture parity tests. It deliberately uses the public C++
model API rather than reimplementing any behavior here.
*******************************************************************************/

#include <cmath>
#include <cstdlib>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

#include "tmModel.h"
#include "tmNLCO.h"

using namespace std;

namespace {

const char* kFixtures[] = {
  "tmModelTester_1.tmd5",
  "tmModelTester_2.tmd5",
  "tmModelTester_3.tmd5",
  "tmModelTester_4.tmd5",
  "tmModelTester_5.tmd5",
  "minimal_v3.tmd",
  "minimal_cp_v4.tmd4",
  "minimal_cp_v5.tmd5",
};

const size_t kFixtureCount = sizeof(kFixtures) / sizeof(kFixtures[0]);

void InitTypesOnce()
{
  if (!tmPart::TypesAreInitialized()) tmPart::InitTypes();
}

string JoinPath(const string& dir, const string& file)
{
  if (dir.empty()) return file;
  char last = dir[dir.size() - 1];
  if (last == '/' || last == '\\') return dir + file;
  return dir + "/" + file;
}

string BaseName(const string& path)
{
  string::size_type pos = path.find_last_of("/\\");
  if (pos == string::npos) return path;
  return path.substr(pos + 1);
}

string JsonEscape(const string& value)
{
  ostringstream os;
  for (string::const_iterator it = value.begin(); it != value.end(); ++it) {
    unsigned char c = static_cast<unsigned char>(*it);
    switch (c) {
      case '"': os << "\\\""; break;
      case '\\': os << "\\\\"; break;
      case '\b': os << "\\b"; break;
      case '\f': os << "\\f"; break;
      case '\n': os << "\\n"; break;
      case '\r': os << "\\r"; break;
      case '\t': os << "\\t"; break;
      default:
        if (c < 0x20) {
          os << "\\u" << hex << setw(4) << setfill('0') << int(c) << dec;
        }
        else {
          os << *it;
        }
    }
  }
  return os.str();
}

const char* BoolStr(bool value)
{
  return value ? "true" : "false";
}

const char* CpStatusName(tmTree::CPStatus status)
{
  switch (status) {
    case tmTree::HAS_FULL_CP: return "HAS_FULL_CP";
    case tmTree::EDGES_TOO_SHORT: return "EDGES_TOO_SHORT";
    case tmTree::POLYS_NOT_VALID: return "POLYS_NOT_VALID";
    case tmTree::POLYS_NOT_FILLED: return "POLYS_NOT_FILLED";
    case tmTree::POLYS_MULTIPLE_IBPS: return "POLYS_MULTIPLE_IBPS";
    case tmTree::VERTICES_LACK_DEPTH: return "VERTICES_LACK_DEPTH";
    case tmTree::FACETS_NOT_VALID: return "FACETS_NOT_VALID";
    case tmTree::NOT_LOCAL_ROOT_CONNECTABLE:
      return "NOT_LOCAL_ROOT_CONNECTABLE";
  }
  return "UNKNOWN";
}

void ReadTree(tmTree& tree, const string& path)
{
  ifstream fin(path.c_str());
  if (!fin.good()) {
    throw runtime_error("unable to open " + path);
  }
  tree.GetSelf(fin);
}

size_t CountLeafNodes(tmTree& tree)
{
  const tmDpptrArray<tmNode>& nodes = tree.GetNodes();
  size_t count = 0;
  for (size_t i = 0; i < nodes.size(); ++i) {
    if (nodes[i]->IsLeafNode()) ++count;
  }
  return count;
}

size_t CountLeafPaths(tmTree& tree)
{
  const tmDpptrArray<tmPath>& paths = tree.GetPaths();
  size_t count = 0;
  for (size_t i = 0; i < paths.size(); ++i) {
    if (paths[i]->IsLeafPath()) ++count;
  }
  return count;
}

double MaxEdgeStrain(const tmDpptrArray<tmEdge>& edges)
{
  double maxStrain = 0.0;
  for (size_t i = 0; i < edges.size(); ++i) {
    if (i == 0 || edges[i]->GetStrain() > maxStrain) {
      maxStrain = edges[i]->GetStrain();
    }
  }
  return maxStrain;
}

double WeightedRmsStrainPercent(const tmDpptrArray<tmEdge>& edges)
{
  if (edges.empty()) return 0.0;
  double ss = 0.0;
  for (size_t i = 0; i < edges.size(); ++i) {
    tmEdge* edge = edges[i];
    ss += edge->GetStiffness() * pow(edge->GetStrain(), 2);
  }
  ss /= edges.size();
  return 100.0 * sqrt(ss);
}

void EmitSummary(const string& path)
{
  tmTree tree;
  ReadTree(tree, path);

  tmArray<tmEdge*> badEdges;
  tmArray<tmPoly*> badPolys;
  tmArray<tmVertex*> badVertices;
  tmArray<tmCrease*> badCreases;
  tmArray<tmFacet*> badFacets;
  tmTree::CPStatus cpStatus =
    tree.GetCPStatus(badEdges, badPolys, badVertices, badCreases, badFacets);

  cout << "{"
       << "\"case\":\"summary\","
       << "\"file\":\"" << JsonEscape(BaseName(path)) << "\","
       << "\"paper_width\":" << tree.GetPaperWidth() << ","
       << "\"paper_height\":" << tree.GetPaperHeight() << ","
       << "\"scale\":" << tree.GetScale() << ","
       << "\"has_symmetry\":" << BoolStr(tree.HasSymmetry()) << ","
       << "\"is_feasible\":" << BoolStr(tree.IsFeasible()) << ","
       << "\"is_polygon_valid\":" << BoolStr(tree.IsPolygonValid()) << ","
       << "\"is_polygon_filled\":" << BoolStr(tree.IsPolygonFilled()) << ","
       << "\"is_vertex_depth_valid\":"
       << BoolStr(tree.IsVertexDepthValid()) << ","
       << "\"is_facet_data_valid\":" << BoolStr(tree.IsFacetDataValid())
       << ","
       << "\"is_local_root_connectable\":"
       << BoolStr(tree.IsLocalRootConnectable()) << ","
       << "\"nodes\":" << tree.GetNumNodes() << ","
       << "\"owned_nodes\":" << tree.GetOwnedNodes().size() << ","
       << "\"leaf_nodes\":" << CountLeafNodes(tree) << ","
       << "\"edges\":" << tree.GetNumEdges() << ","
       << "\"owned_edges\":" << tree.GetOwnedEdges().size() << ","
       << "\"paths\":" << tree.GetNumPaths() << ","
       << "\"leaf_paths\":" << CountLeafPaths(tree) << ","
       << "\"polys\":" << tree.GetNumPolys() << ","
       << "\"vertices\":" << tree.GetNumVertices() << ","
       << "\"creases\":" << tree.GetNumCreases() << ","
       << "\"facets\":" << tree.GetFacets().size() << ","
       << "\"conditions\":" << tree.GetNumConditions() << ","
       << "\"cp_status\":\"" << CpStatusName(cpStatus) << "\","
       << "\"bad_edges\":" << badEdges.size() << ","
       << "\"bad_polys\":" << badPolys.size() << ","
       << "\"bad_vertices\":" << badVertices.size() << ","
       << "\"bad_creases\":" << badCreases.size() << ","
       << "\"bad_facets\":" << badFacets.size()
       << "}" << endl;
}

void EmitOptimize(const string& path, const string& kind)
{
  tmTree tree;
  ReadTree(tree, path);

  bool converged = true;
  int reason = 0;
  string error;
  size_t movingNodeCount = 0;
  size_t stretchyEdgeCount = 0;

  try {
    if (kind == "scale") {
      tmNLCO_alm nlco;
      tmScaleOptimizer optimizer(&tree, &nlco);
      optimizer.Initialize();
      optimizer.Optimize();
    }
    else if (kind == "edge") {
      tmDpptrArray<tmNode> movingNodes = tree.GetOwnedNodes();
      tmDpptrArray<tmEdge> stretchyEdges = tree.GetOwnedEdges();
      movingNodeCount = movingNodes.size();
      stretchyEdgeCount = stretchyEdges.size();

      tmNLCO_alm nlco;
      tmEdgeOptimizer optimizer(&tree, &nlco);
      optimizer.Initialize(movingNodes, stretchyEdges);
      movingNodeCount = movingNodes.size();
      stretchyEdgeCount = stretchyEdges.size();
      optimizer.Optimize();
    }
    else if (kind == "strain") {
      tmDpptrArray<tmNode> movingNodes = tree.GetOwnedNodes();
      tmDpptrArray<tmEdge> stretchyEdges = tree.GetOwnedEdges();

      tmNLCO_alm nlco;
      tmStrainOptimizer optimizer(&tree, &nlco);
      optimizer.Initialize(movingNodes, stretchyEdges);
      movingNodeCount = movingNodes.size();
      stretchyEdgeCount = stretchyEdges.size();
      optimizer.Optimize();
    }
    else {
      throw runtime_error("unknown optimization kind " + kind);
    }
  }
  catch (tmNLCO::EX_BAD_CONVERGENCE ex) {
    converged = false;
    reason = ex.GetReason();
  }
  catch (tmScaleOptimizer::EX_BAD_SCALE) {
    converged = false;
    reason = -1001;
    error = "bad_scale";
  }
  catch (tmEdgeOptimizer::EX_NO_MOVING_NODES) {
    converged = false;
    reason = -1002;
    error = "no_moving_nodes";
  }
  catch (tmEdgeOptimizer::EX_NO_MOVING_EDGES) {
    converged = false;
    reason = -1003;
    error = "no_moving_edges";
  }
  catch (tmStrainOptimizer::EX_NO_MOVING_NODES_OR_EDGES) {
    converged = false;
    reason = -1004;
    error = "no_moving_nodes_or_edges";
  }

  const tmDpptrArray<tmEdge>& edges = tree.GetOwnedEdges();

  cout << "{"
       << "\"case\":\"optimize\","
       << "\"kind\":\"" << JsonEscape(kind) << "\","
       << "\"file\":\"" << JsonEscape(BaseName(path)) << "\","
       << "\"converged\":" << BoolStr(converged) << ","
       << "\"reason\":" << reason << ","
       << "\"error\":\"" << JsonEscape(error) << "\","
       << "\"scale\":" << tree.GetScale() << ","
       << "\"is_feasible\":" << BoolStr(tree.IsFeasible()) << ","
       << "\"moving_nodes\":" << movingNodeCount << ","
       << "\"stretchy_edges\":" << stretchyEdgeCount << ","
       << "\"max_edge_strain\":" << MaxEdgeStrain(edges) << ","
       << "\"weighted_rms_strain_percent\":"
       << WeightedRmsStrainPercent(edges) << ","
       << "\"nodes\":" << tree.GetNumNodes() << ","
       << "\"edges\":" << tree.GetNumEdges() << ","
       << "\"paths\":" << tree.GetNumPaths() << ","
       << "\"conditions\":" << tree.GetNumConditions()
       << "}" << endl;
}

void Usage(ostream& os)
{
  os << "Usage:\n"
     << "  treemaker-oracle summary <file>\n"
     << "  treemaker-oracle optimize <scale|edge|strain> <file>\n"
     << "  treemaker-oracle run-fixtures [--fixture-dir <dir>]\n";
}

int RunFixtures(int argc, char** argv)
{
  string fixtureDir = "tests/fixtures";
  for (int i = 2; i < argc; ++i) {
    string arg = argv[i];
    if (arg == "--fixture-dir" && i + 1 < argc) {
      fixtureDir = argv[++i];
    }
    else {
      throw runtime_error("unknown run-fixtures argument " + arg);
    }
  }

  for (size_t i = 0; i < kFixtureCount; ++i) {
    EmitSummary(JoinPath(fixtureDir, kFixtures[i]));
  }

  EmitOptimize(JoinPath(fixtureDir, kFixtures[0]), "scale");
  EmitOptimize(JoinPath(fixtureDir, kFixtures[1]), "scale");
  EmitOptimize(JoinPath(fixtureDir, kFixtures[2]), "scale");
  EmitOptimize(JoinPath(fixtureDir, kFixtures[3]), "edge");
  EmitOptimize(JoinPath(fixtureDir, kFixtures[4]), "strain");
  return 0;
}

}  // namespace

int main(int argc, char** argv)
{
  cout.setf(ios_base::fixed);
  cout.precision(10);
  InitTypesOnce();

  try {
    if (argc < 2) {
      Usage(cerr);
      return 2;
    }

    string command = argv[1];
    if (command == "summary") {
      if (argc != 3) {
        Usage(cerr);
        return 2;
      }
      EmitSummary(argv[2]);
      return 0;
    }

    if (command == "optimize") {
      if (argc != 4) {
        Usage(cerr);
        return 2;
      }
      EmitOptimize(argv[3], argv[2]);
      return 0;
    }

    if (command == "run-fixtures") {
      return RunFixtures(argc, argv);
    }

    Usage(cerr);
    return 2;
  }
  catch (const exception& ex) {
    cerr << "treemaker-oracle: " << ex.what() << endl;
    return 1;
  }
  catch (tmPart::EX_IO_BAD_TOKEN ex) {
    cerr << "treemaker-oracle: bad token " << ex.mToken << endl;
    return 1;
  }
  catch (tmTree::EX_IO_UNRECOGNIZED_CONDITION ex) {
    cerr << "treemaker-oracle: unrecognized conditions " << ex.mNumMissed
         << endl;
    return 1;
  }
  catch (...) {
    cerr << "treemaker-oracle: unknown exception" << endl;
    return 1;
  }
}
