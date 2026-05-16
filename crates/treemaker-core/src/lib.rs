use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod nlco;

pub type TmFloat = f64;

const DIST_TOL: TmFloat = 1.0e-4;
const MIN_EDGE_LENGTH: TmFloat = 0.01;
const DEPTH_NOT_SET: TmFloat = -999.0;
const DEGREES: TmFloat = 0.017453292519943296;

#[derive(Debug, thiserror::Error)]
pub enum TreeError {
    #[error("parse error at byte {offset}: {message}")]
    Parse { offset: usize, message: String },
    #[error("bad {kind} reference index {index}; valid range is 1..={max}")]
    BadReference {
        kind: &'static str,
        index: usize,
        max: usize,
    },
    #[error("unsupported TreeMaker document version {0}")]
    UnsupportedVersion(String),
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(&'static str),
    #[error("optimizer failed to converge: {0}")]
    OptimizerConvergence(String),
    #[error("invalid operation: {0}")]
    InvalidOperation(&'static str),
}

pub type Result<T> = std::result::Result<T, TreeError>;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: TmFloat,
    pub y: TmFloat,
}

impl Point {
    pub fn distance(self, other: Self) -> TmFloat {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerRef {
    Tree,
    Node(usize),
    Path(usize),
    Poly(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub index: usize,
    pub label: String,
    pub loc: Point,
    pub depth: TmFloat,
    pub elevation: TmFloat,
    pub is_leaf: bool,
    pub is_sub: bool,
    pub is_border: bool,
    pub is_pinned: bool,
    pub is_polygon: bool,
    pub is_junction: bool,
    pub is_conditioned: bool,
    pub owned_vertices: Vec<usize>,
    pub edges: Vec<usize>,
    pub leaf_paths: Vec<usize>,
    pub owner: OwnerRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub index: usize,
    pub label: String,
    pub length: TmFloat,
    pub strain: TmFloat,
    pub stiffness: TmFloat,
    pub is_pinned: bool,
    pub is_conditioned: bool,
    pub nodes: Vec<usize>,
}

impl Edge {
    pub fn strained_length(&self) -> TmFloat {
        self.length * (1.0 + self.strain)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub index: usize,
    pub min_tree_length: TmFloat,
    pub min_paper_length: TmFloat,
    pub act_tree_length: TmFloat,
    pub act_paper_length: TmFloat,
    pub is_leaf: bool,
    pub is_sub: bool,
    pub is_feasible: bool,
    pub is_active: bool,
    pub is_border: bool,
    pub is_polygon: bool,
    pub is_conditioned: bool,
    pub fwd_poly: Option<usize>,
    pub bkd_poly: Option<usize>,
    pub nodes: Vec<usize>,
    pub edges: Vec<usize>,
    pub outset_path: Option<usize>,
    pub front_reduction: TmFloat,
    pub back_reduction: TmFloat,
    pub min_depth: TmFloat,
    pub min_depth_dist: TmFloat,
    pub owned_vertices: Vec<usize>,
    pub owned_creases: Vec<usize>,
    pub owner: OwnerRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPoly {
    pub index: usize,
    pub centroid: Point,
    pub is_sub_poly: bool,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawVertex {
    pub index: usize,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCrease {
    pub index: usize,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFacet {
    pub index: usize,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub index: usize,
    pub is_feasible: bool,
    pub kind: ConditionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionKind {
    NodeCombo {
        node: usize,
        to_symmetry_line: bool,
        to_paper_edge: bool,
        to_paper_corner: bool,
        x_fixed: bool,
        x_fix_value: TmFloat,
        y_fixed: bool,
        y_fix_value: TmFloat,
    },
    NodeFixed {
        node: usize,
        x_fixed: bool,
        y_fixed: bool,
        x_fix_value: TmFloat,
        y_fix_value: TmFloat,
    },
    NodeOnCorner {
        node: usize,
    },
    NodeOnEdge {
        node: usize,
    },
    NodeSymmetric {
        node: usize,
    },
    NodesPaired {
        node1: usize,
        node2: usize,
    },
    NodesCollinear {
        node1: usize,
        node2: usize,
        node3: usize,
    },
    EdgeLengthFixed {
        edge: usize,
    },
    EdgesSameStrain {
        edge1: usize,
        edge2: usize,
    },
    PathCombo {
        node1: usize,
        node2: usize,
        is_angle_fixed: bool,
        angle: TmFloat,
        is_angle_quant: bool,
        quant: usize,
        quant_offset: TmFloat,
    },
    PathActive {
        node1: usize,
        node2: usize,
    },
    PathAngleFixed {
        node1: usize,
        node2: usize,
        angle: TmFloat,
    },
    PathAngleQuant {
        node1: usize,
        node2: usize,
        quant: usize,
        quant_offset: TmFloat,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub source_version: String,
    pub paper_width: TmFloat,
    pub paper_height: TmFloat,
    pub scale: TmFloat,
    pub has_symmetry: bool,
    pub sym_loc: Point,
    pub sym_angle: TmFloat,
    pub is_feasible: bool,
    pub is_polygon_valid: bool,
    pub is_polygon_filled: bool,
    pub is_vertex_depth_valid: bool,
    pub is_facet_data_valid: bool,
    pub is_local_root_connectable: bool,
    pub needs_cleanup: bool,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub paths: Vec<Path>,
    pub polys: Vec<RawPoly>,
    pub vertices: Vec<RawVertex>,
    pub creases: Vec<RawCrease>,
    pub facets: Vec<RawFacet>,
    pub conditions: Vec<Condition>,
    pub owned_nodes: Vec<usize>,
    pub owned_edges: Vec<usize>,
    pub owned_paths: Vec<usize>,
    pub owned_polys: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CPStatus {
    HasFullCp,
    EdgesTooShort,
    PolysNotValid,
    PolysNotFilled,
    PolysMultipleIbps,
    VerticesLackDepth,
    FacetsNotValid,
    NotLocalRootConnectable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeSummary {
    pub source_version: String,
    pub paper_width: TmFloat,
    pub paper_height: TmFloat,
    pub scale: TmFloat,
    pub has_symmetry: bool,
    pub is_feasible: bool,
    pub cp_status: CPStatus,
    pub nodes: usize,
    pub edges: usize,
    pub paths: usize,
    pub polys: usize,
    pub vertices: usize,
    pub creases: usize,
    pub facets: usize,
    pub conditions: usize,
    pub leaf_nodes: usize,
    pub leaf_paths: usize,
    pub conditioned_nodes: usize,
    pub conditioned_edges: usize,
    pub conditioned_paths: usize,
    pub conditions_by_tag: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimizationReport {
    pub kind: OptimizationKind,
    pub converged: bool,
    pub old_scale: TmFloat,
    pub new_scale: TmFloat,
    pub is_feasible: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationKind {
    Scale,
    Edge,
    Strain,
}

impl Tree {
    pub fn from_tmd_str(input: &str) -> Result<Self> {
        let mut reader = Reader::new(input);
        reader.expect_tag("tree")?;
        let version = reader.read_token("version")?;
        let tree = match version.as_str() {
            "4.0" => {
                let mut tree = Self::read_v4(&mut reader, version)?;
                tree.validate()?;
                tree.cleanup_lengths_and_feasibility();
                tree
            }
            "5.0" => {
                let tree = Self::read_v5(&mut reader, version)?;
                tree.validate()?;
                tree
            }
            "3.0" => return Err(TreeError::UnsupportedVersion(version)),
            _ => return Err(TreeError::UnsupportedVersion(version)),
        };
        Ok(tree)
    }

    pub fn to_tmd5_string(&self) -> String {
        let mut out = Writer::new(10, "\n");
        out.s("tree");
        out.s("5.0");
        out.f(self.paper_width);
        out.f(self.paper_height);
        out.f(self.scale);
        out.b(self.has_symmetry);
        out.point(self.sym_loc);
        out.f(self.sym_angle);
        out.b(self.is_feasible);
        out.b(self.is_polygon_valid);
        out.b(self.is_polygon_filled);
        out.b(self.is_vertex_depth_valid);
        out.b(self.is_facet_data_valid);
        out.b(self.is_local_root_connectable);
        out.b(self.needs_cleanup);
        out.u(self.nodes.len());
        out.u(self.edges.len());
        out.u(self.paths.len());
        out.u(self.polys.len());
        out.u(self.vertices.len());
        out.u(self.creases.len());
        out.u(self.facets.len());
        out.u(self.conditions.len());

        for node in &self.nodes {
            out.node_v5(node);
        }
        for edge in &self.edges {
            out.edge(edge);
        }
        for path in &self.paths {
            out.path_v5(path);
        }
        for poly in &self.polys {
            out.raw_part("poly", poly.index, &poly.fields);
        }
        for vertex in &self.vertices {
            out.raw_part("vrtx", vertex.index, &vertex.fields);
        }
        for crease in &self.creases {
            out.raw_part("crse", crease.index, &crease.fields);
        }
        for facet in &self.facets {
            out.raw_part("fact", facet.index, &facet.fields);
        }
        for condition in &self.conditions {
            out.condition_v5(condition);
        }
        out.array(&self.owned_nodes);
        out.array(&self.owned_edges);
        out.array(&self.owned_paths);
        out.array(&self.owned_polys);
        out.finish()
    }

    pub fn export_v4_string(&self) -> String {
        let mut out = Writer::new(6, "\r");
        out.s("tree");
        out.s("4.0");
        out.f(self.paper_width);
        out.f(self.paper_height);
        out.f(self.scale);
        out.b(self.has_symmetry);
        out.point(self.sym_loc);
        out.f(self.sym_angle);
        out.u(self.nodes.len());
        out.u(self.edges.len());
        out.u(self.paths.len());
        out.u(0);
        out.u(0);
        out.u(0);
        out.u(self.conditions.len());
        for node in &self.nodes {
            out.node_v4(node);
        }
        for edge in &self.edges {
            out.edge(edge);
        }
        for path in &self.paths {
            out.path_v4(path);
        }
        for condition in &self.conditions {
            out.condition_v4(condition);
        }
        out.array(&self.owned_nodes);
        out.array(&self.owned_edges);
        out.array(&self.owned_paths);
        out.u(0);
        out.finish()
    }

    pub fn summary(&self) -> TreeSummary {
        let mut conditions_by_tag = BTreeMap::new();
        for condition in &self.conditions {
            *conditions_by_tag
                .entry(condition.kind.tag().to_string())
                .or_insert(0) += 1;
        }
        TreeSummary {
            source_version: self.source_version.clone(),
            paper_width: self.paper_width,
            paper_height: self.paper_height,
            scale: self.scale,
            has_symmetry: self.has_symmetry,
            is_feasible: self.is_feasible,
            cp_status: self.cp_status(),
            nodes: self.nodes.len(),
            edges: self.edges.len(),
            paths: self.paths.len(),
            polys: self.polys.len(),
            vertices: self.vertices.len(),
            creases: self.creases.len(),
            facets: self.facets.len(),
            conditions: self.conditions.len(),
            leaf_nodes: self.nodes.iter().filter(|n| n.is_leaf).count(),
            leaf_paths: self.paths.iter().filter(|p| p.is_leaf).count(),
            conditioned_nodes: self.nodes.iter().filter(|n| n.is_conditioned).count(),
            conditioned_edges: self.edges.iter().filter(|e| e.is_conditioned).count(),
            conditioned_paths: self.paths.iter().filter(|p| p.is_conditioned).count(),
            conditions_by_tag,
        }
    }

    pub fn is_feasible(&self) -> bool {
        self.is_feasible
    }

    pub fn cp_status(&self) -> CPStatus {
        if self
            .edges
            .iter()
            .any(|edge| edge.strained_length() < MIN_EDGE_LENGTH)
        {
            return CPStatus::EdgesTooShort;
        }
        if !self.is_polygon_valid {
            return CPStatus::PolysNotValid;
        }
        if !self.is_polygon_filled {
            return CPStatus::PolysNotFilled;
        }
        if !self.is_vertex_depth_valid {
            return CPStatus::VerticesLackDepth;
        }
        if !self.is_facet_data_valid {
            return CPStatus::FacetsNotValid;
        }
        if !self.is_local_root_connectable {
            return CPStatus::NotLocalRootConnectable;
        }
        CPStatus::HasFullCp
    }

    pub fn optimize_scale(&mut self) -> Result<OptimizationReport> {
        let old_scale = self.scale;
        let leaf_nodes = self.leaf_nodes_in_owned_order();
        let num_vars = 1 + 2 * leaf_nodes.len();

        let mut state = vec![0.0; num_vars];
        state[0] = self.scale;
        let mut node_offsets = vec![None; self.nodes.len() + 1];
        for (i, node_id) in leaf_nodes.iter().copied().enumerate() {
            let offset = 1 + 2 * i;
            node_offsets[node_id] = Some(offset);
            let loc = self.nodes[node_id - 1].loc;
            state[offset] = loc.x;
            state[offset + 1] = loc.y;
        }

        let mut optimizer = nlco::NlcoAlm::new(num_vars);
        let mut lower_bounds = vec![0.0; num_vars];
        let mut upper_bounds = vec![0.0; num_vars];
        upper_bounds[0] = 2.0;
        for i in 0..leaf_nodes.len() {
            lower_bounds[1 + 2 * i] = 0.0;
            lower_bounds[2 + 2 * i] = 0.0;
            upper_bounds[1 + 2 * i] = self.paper_width;
            upper_bounds[2 + 2 * i] = self.paper_height;
        }
        optimizer.set_bounds(lower_bounds, upper_bounds);
        optimizer.set_objective(Box::new(ScaleObjective));
        optimizer.add_linear_inequality(Box::new(nlco::OneVarFn::new(0, -1.0, 0.1 * self.scale)));

        for path_id in &self.owned_paths {
            let path = &self.paths[*path_id - 1];
            if !path.is_leaf || self.has_path_active_base_condition(path) {
                continue;
            }
            self.add_scale_path_constraint(
                &mut optimizer,
                &node_offsets,
                path.nodes[0],
                *path.nodes.last().unwrap(),
                path.min_tree_length,
                false,
            );
        }

        for condition in &self.conditions {
            self.add_scale_condition_constraints(&mut optimizer, &node_offsets, &condition.kind);
        }

        let inform = optimizer.minimize(&mut state);
        if inform != 0 {
            return Err(TreeError::OptimizerConvergence(format!(
                "ALM returned result code {inform}"
            )));
        }

        self.scale = state[0];
        for (i, node_id) in leaf_nodes.into_iter().enumerate() {
            let offset = 1 + 2 * i;
            self.nodes[node_id - 1].loc = Point {
                x: state[offset],
                y: state[offset + 1],
            };
        }
        self.cleanup_lengths_and_feasibility();

        Ok(OptimizationReport {
            kind: OptimizationKind::Scale,
            converged: true,
            old_scale,
            new_scale: self.scale,
            is_feasible: self.is_feasible,
            message: "ALM scale optimization converged".to_string(),
        })
    }

    pub fn optimize_edges(&mut self) -> Result<OptimizationReport> {
        let old_scale = self.scale;
        let moving_nodes = self.moving_nodes_for_edge_optimizer();
        let stretchy_edges = self.stretchy_edges_for_edge_optimizer();
        if moving_nodes.is_empty() {
            return Err(TreeError::InvalidOperation(
                "edge optimization has no moving leaf nodes",
            ));
        }
        if stretchy_edges.is_empty() {
            return Err(TreeError::InvalidOperation(
                "edge optimization has no stretchy edges",
            ));
        }

        let num_vars = 1 + 2 * moving_nodes.len();
        let mut state = vec![0.0; num_vars];
        let mut node_offsets = vec![None; self.nodes.len() + 1];
        for (i, node_id) in moving_nodes.iter().copied().enumerate() {
            let offset = 1 + 2 * i;
            node_offsets[node_id] = Some(offset);
            let loc = self.nodes[node_id - 1].loc;
            state[offset] = loc.x;
            state[offset + 1] = loc.y;
        }
        let stretchy_lookup = self.edge_lookup(&stretchy_edges);

        let mut optimizer = nlco::NlcoAlm::new(num_vars);
        let mut lower_bounds = vec![0.0; num_vars];
        let mut upper_bounds = vec![0.0; num_vars];
        lower_bounds[0] = -0.999;
        upper_bounds[0] = 10.0;
        for i in 0..moving_nodes.len() {
            upper_bounds[1 + 2 * i] = self.paper_width;
            upper_bounds[2 + 2 * i] = self.paper_height;
        }
        optimizer.set_bounds(lower_bounds, upper_bounds);
        optimizer.set_objective(Box::new(ScaleObjective));

        for path_id in &self.owned_paths {
            let path = &self.paths[*path_id - 1];
            if !path.is_leaf || self.has_path_active_base_condition(path) {
                continue;
            }
            self.add_edge_path_constraint(
                &mut optimizer,
                &node_offsets,
                &stretchy_lookup,
                path,
                false,
            );
        }
        for condition in &self.conditions {
            self.add_edge_condition_constraints(
                &mut optimizer,
                &node_offsets,
                &stretchy_lookup,
                &condition.kind,
            );
        }

        let inform = optimizer.minimize(&mut state);
        if inform != 0 {
            return Err(TreeError::OptimizerConvergence(format!(
                "ALM returned result code {inform}"
            )));
        }

        for (i, node_id) in moving_nodes.into_iter().enumerate() {
            let offset = 1 + 2 * i;
            self.nodes[node_id - 1].loc = Point {
                x: state[offset],
                y: state[offset + 1],
            };
        }
        for edge_id in stretchy_edges {
            self.edges[edge_id - 1].strain = state[0];
        }
        self.cleanup_lengths_and_feasibility();

        Ok(OptimizationReport {
            kind: OptimizationKind::Edge,
            converged: true,
            old_scale,
            new_scale: self.scale,
            is_feasible: self.is_feasible,
            message: "ALM edge strain optimization converged".to_string(),
        })
    }

    pub fn optimize_strain(&mut self) -> Result<OptimizationReport> {
        let old_scale = self.scale;
        let moving_nodes = self.moving_nodes_for_strain_optimizer();
        let stretchy_edges = self.owned_edges.clone();
        if moving_nodes.is_empty() && stretchy_edges.is_empty() {
            return Err(TreeError::InvalidOperation(
                "strain optimization has no moving nodes or edges",
            ));
        }

        let edge_offset = 2 * moving_nodes.len();
        let num_vars = edge_offset + stretchy_edges.len();
        let mut state = vec![0.0; num_vars];
        let mut node_offsets = vec![None; self.nodes.len() + 1];
        let mut edge_offsets = vec![None; self.edges.len() + 1];

        for (i, node_id) in moving_nodes.iter().copied().enumerate() {
            let offset = 2 * i;
            node_offsets[node_id] = Some(offset);
            let loc = self.nodes[node_id - 1].loc;
            state[offset] = loc.x;
            state[offset + 1] = loc.y;
        }
        let mut stiffness = Vec::with_capacity(stretchy_edges.len());
        for (i, edge_id) in stretchy_edges.iter().copied().enumerate() {
            let offset = edge_offset + i;
            edge_offsets[edge_id] = Some(offset);
            state[offset] = self.edges[edge_id - 1].strain;
            let edge_stiffness = self.edges[edge_id - 1].stiffness;
            stiffness.push(if edge_stiffness <= 0.0 {
                1.0
            } else {
                edge_stiffness
            });
        }

        let mut optimizer = nlco::NlcoAlm::new(num_vars);
        let mut lower_bounds = vec![0.0; num_vars];
        let mut upper_bounds = vec![0.0; num_vars];
        for i in 0..moving_nodes.len() {
            upper_bounds[2 * i] = self.paper_width;
            upper_bounds[2 * i + 1] = self.paper_height;
        }
        for i in edge_offset..num_vars {
            lower_bounds[i] = -0.999;
            upper_bounds[i] = 2.0;
        }
        optimizer.set_bounds(lower_bounds, upper_bounds);
        optimizer.set_objective(Box::new(StrainObjective {
            edge_offset,
            stiffness,
        }));

        for path_id in &self.owned_paths {
            let path = &self.paths[*path_id - 1];
            if !path.is_leaf || self.has_path_active_base_condition(path) {
                continue;
            }
            self.add_strain_path_constraint(
                &mut optimizer,
                &node_offsets,
                &edge_offsets,
                path,
                false,
            );
        }
        for condition in &self.conditions {
            self.add_strain_condition_constraints(
                &mut optimizer,
                &node_offsets,
                &edge_offsets,
                &condition.kind,
            );
        }

        let inform = optimizer.minimize(&mut state);
        if inform != 0 {
            return Err(TreeError::OptimizerConvergence(format!(
                "ALM returned result code {inform}"
            )));
        }

        for (i, node_id) in moving_nodes.into_iter().enumerate() {
            let offset = 2 * i;
            self.nodes[node_id - 1].loc = Point {
                x: state[offset],
                y: state[offset + 1],
            };
        }
        for (i, edge_id) in stretchy_edges.into_iter().enumerate() {
            self.edges[edge_id - 1].strain = state[edge_offset + i];
        }
        self.cleanup_lengths_and_feasibility();

        Ok(OptimizationReport {
            kind: OptimizationKind::Strain,
            converged: true,
            old_scale,
            new_scale: self.scale,
            is_feasible: self.is_feasible,
            message: "ALM strain optimization converged".to_string(),
        })
    }

    pub fn build_tree_polys(&mut self) -> Result<()> {
        Err(TreeError::UnsupportedOperation(
            "tree polygon construction is not ported yet",
        ))
    }

    pub fn build_polys_and_crease_pattern(&mut self) -> Result<()> {
        Err(TreeError::UnsupportedOperation(
            "crease-pattern generation is not ported yet",
        ))
    }

    fn read_v4(reader: &mut Reader<'_>, source_version: String) -> Result<Self> {
        let paper_width = reader.read_f64("paper width")?;
        let paper_height = reader.read_f64("paper height")?;
        let scale = reader.read_f64("scale")?;
        let has_symmetry = reader.read_bool("has symmetry")?;
        let sym_loc = reader.read_point("symmetry location")?;
        let sym_angle = reader.read_f64("symmetry angle")?;
        let num_nodes = reader.read_usize("node count")?;
        let num_edges = reader.read_usize("edge count")?;
        let num_paths = reader.read_usize("path count")?;
        let num_polys = reader.read_usize("poly count")?;
        let num_vertices = reader.read_usize("vertex count")?;
        let num_creases = reader.read_usize("crease count")?;
        let num_conditions = reader.read_usize("condition count")?;
        if num_polys != 0 || num_vertices != 0 || num_creases != 0 {
            return Err(TreeError::UnsupportedOperation(
                "v4 legacy crease-pattern payload parsing must be ported before loading CP-bearing v4 files",
            ));
        }

        let mut nodes = Vec::with_capacity(num_nodes);
        let mut edges = Vec::with_capacity(num_edges);
        let mut paths = Vec::with_capacity(num_paths);
        let mut polys = Vec::with_capacity(num_polys);
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut creases = Vec::with_capacity(num_creases);

        for _ in 0..num_nodes {
            nodes.push(reader.read_node_v4()?);
        }
        for _ in 0..num_edges {
            edges.push(reader.read_edge(true)?);
        }
        for _ in 0..num_paths {
            paths.push(reader.read_path_v4()?);
        }
        for _ in 0..num_polys {
            polys.push(reader.read_poly_v4()?);
        }
        for _ in 0..num_vertices {
            vertices.push(reader.read_vertex_v4()?);
        }
        for _ in 0..num_creases {
            creases.push(reader.read_crease_v4()?);
        }

        let mut conditions = Vec::with_capacity(num_conditions);
        for i in 0..num_conditions {
            conditions.push(reader.read_condition_v4(i + 1)?);
        }

        let owned_nodes = reader.read_index_array("owned nodes")?;
        let owned_edges = reader.read_index_array("owned edges")?;
        let owned_paths = reader.read_index_array("owned paths")?;
        let owned_polys = reader.read_index_array("owned polys")?;

        Ok(Self {
            source_version,
            paper_width,
            paper_height,
            scale,
            has_symmetry,
            sym_loc,
            sym_angle,
            is_feasible: false,
            is_polygon_valid: false,
            is_polygon_filled: false,
            is_vertex_depth_valid: false,
            is_facet_data_valid: false,
            is_local_root_connectable: false,
            needs_cleanup: false,
            nodes,
            edges,
            paths,
            polys,
            vertices,
            creases,
            facets: Vec::new(),
            conditions,
            owned_nodes,
            owned_edges,
            owned_paths,
            owned_polys,
        })
    }

    fn read_v5(reader: &mut Reader<'_>, source_version: String) -> Result<Self> {
        let paper_width = reader.read_f64("paper width")?;
        let paper_height = reader.read_f64("paper height")?;
        let scale = reader.read_f64("scale")?;
        let has_symmetry = reader.read_bool("has symmetry")?;
        let sym_loc = reader.read_point("symmetry location")?;
        let sym_angle = reader.read_f64("symmetry angle")?;
        let is_feasible = reader.read_bool("feasible flag")?;
        let is_polygon_valid = reader.read_bool("polygon valid flag")?;
        let is_polygon_filled = reader.read_bool("polygon filled flag")?;
        let is_vertex_depth_valid = reader.read_bool("vertex depth valid flag")?;
        let is_facet_data_valid = reader.read_bool("facet data valid flag")?;
        let is_local_root_connectable = reader.read_bool("local root connectable flag")?;
        let needs_cleanup = reader.read_bool("needs cleanup flag")?;
        let num_nodes = reader.read_usize("node count")?;
        let num_edges = reader.read_usize("edge count")?;
        let num_paths = reader.read_usize("path count")?;
        let num_polys = reader.read_usize("poly count")?;
        let num_vertices = reader.read_usize("vertex count")?;
        let num_creases = reader.read_usize("crease count")?;
        let num_facets = reader.read_usize("facet count")?;
        let num_conditions = reader.read_usize("condition count")?;
        if num_polys != 0 || num_vertices != 0 || num_creases != 0 || num_facets != 0 {
            return Err(TreeError::UnsupportedOperation(
                "typed v5 crease-pattern payload parsing is not ported yet",
            ));
        }

        let mut nodes = Vec::with_capacity(num_nodes);
        let mut edges = Vec::with_capacity(num_edges);
        let mut paths = Vec::with_capacity(num_paths);
        let mut polys = Vec::with_capacity(num_polys);
        let mut vertices = Vec::with_capacity(num_vertices);
        let mut creases = Vec::with_capacity(num_creases);
        let mut facets = Vec::with_capacity(num_facets);

        for _ in 0..num_nodes {
            nodes.push(reader.read_node_v5()?);
        }
        for _ in 0..num_edges {
            edges.push(reader.read_edge(false)?);
        }
        for _ in 0..num_paths {
            paths.push(reader.read_path_v5()?);
        }
        for _ in 0..num_polys {
            polys.push(reader.read_poly_v5()?);
        }
        for _ in 0..num_vertices {
            vertices.push(reader.read_vertex_v5()?);
        }
        for _ in 0..num_creases {
            creases.push(reader.read_crease_v5()?);
        }
        for _ in 0..num_facets {
            facets.push(reader.read_facet_v5()?);
        }

        let mut conditions = Vec::with_capacity(num_conditions);
        for _ in 0..num_conditions {
            conditions.push(reader.read_condition_v5()?);
        }

        let owned_nodes = reader.read_index_array("owned nodes")?;
        let owned_edges = reader.read_index_array("owned edges")?;
        let owned_paths = reader.read_index_array("owned paths")?;
        let owned_polys = reader.read_index_array("owned polys")?;

        Ok(Self {
            source_version,
            paper_width,
            paper_height,
            scale,
            has_symmetry,
            sym_loc,
            sym_angle,
            is_feasible,
            is_polygon_valid,
            is_polygon_filled,
            is_vertex_depth_valid,
            is_facet_data_valid,
            is_local_root_connectable,
            needs_cleanup,
            nodes,
            edges,
            paths,
            polys,
            vertices,
            creases,
            facets,
            conditions,
            owned_nodes,
            owned_edges,
            owned_paths,
            owned_polys,
        })
    }

    fn validate(&self) -> Result<()> {
        for id in &self.owned_nodes {
            self.check_ref("node", *id, self.nodes.len())?;
        }
        for id in &self.owned_edges {
            self.check_ref("edge", *id, self.edges.len())?;
        }
        for id in &self.owned_paths {
            self.check_ref("path", *id, self.paths.len())?;
        }
        for id in &self.owned_polys {
            self.check_ref("poly", *id, self.polys.len())?;
        }
        for node in &self.nodes {
            for id in &node.edges {
                self.check_ref("edge", *id, self.edges.len())?;
            }
            for id in &node.leaf_paths {
                self.check_ref("path", *id, self.paths.len())?;
            }
            for id in &node.owned_vertices {
                self.check_ref("vertex", *id, self.vertices.len())?;
            }
            self.check_owner(&node.owner)?;
        }
        for edge in &self.edges {
            for id in &edge.nodes {
                self.check_ref("node", *id, self.nodes.len())?;
            }
        }
        for path in &self.paths {
            for id in &path.nodes {
                self.check_ref("node", *id, self.nodes.len())?;
            }
            for id in &path.edges {
                self.check_ref("edge", *id, self.edges.len())?;
            }
            if let Some(id) = path.fwd_poly {
                self.check_ref("poly", id, self.polys.len())?;
            }
            if let Some(id) = path.bkd_poly {
                self.check_ref("poly", id, self.polys.len())?;
            }
            if let Some(id) = path.outset_path {
                self.check_ref("path", id, self.paths.len())?;
            }
            for id in &path.owned_vertices {
                self.check_ref("vertex", *id, self.vertices.len())?;
            }
            for id in &path.owned_creases {
                self.check_ref("crease", *id, self.creases.len())?;
            }
            self.check_owner(&path.owner)?;
        }
        for condition in &self.conditions {
            condition.kind.validate_refs(self)?;
        }
        Ok(())
    }

    fn check_ref(&self, kind: &'static str, index: usize, max: usize) -> Result<()> {
        if index == 0 || index > max {
            return Err(TreeError::BadReference { kind, index, max });
        }
        Ok(())
    }

    fn check_owner(&self, owner: &OwnerRef) -> Result<()> {
        match *owner {
            OwnerRef::Tree => Ok(()),
            OwnerRef::Node(id) => self.check_ref("node", id, self.nodes.len()),
            OwnerRef::Path(id) => self.check_ref("path", id, self.paths.len()),
            OwnerRef::Poly(id) => self.check_ref("poly", id, self.polys.len()),
        }
    }

    fn find_leaf_path_between(&self, node1: usize, node2: usize) -> Option<&Path> {
        self.paths.iter().find(|path| {
            path.is_leaf
                && matches!(
                    path.nodes.first().copied().zip(path.nodes.last().copied()),
                    Some((a, b)) if (a == node1 && b == node2) || (a == node2 && b == node1)
                )
        })
    }

    fn leaf_nodes_in_owned_order(&self) -> Vec<usize> {
        self.owned_nodes
            .iter()
            .copied()
            .filter(|id| self.nodes[id - 1].is_leaf)
            .collect()
    }

    fn has_path_active_base_condition(&self, path: &Path) -> bool {
        let Some((node1, node2)) = path.nodes.first().copied().zip(path.nodes.last().copied())
        else {
            return false;
        };
        self.conditions
            .iter()
            .any(|condition| match condition.kind {
                ConditionKind::PathActive { node1: a, node2: b }
                | ConditionKind::PathAngleFixed {
                    node1: a, node2: b, ..
                }
                | ConditionKind::PathAngleQuant {
                    node1: a, node2: b, ..
                } => (a == node1 && b == node2) || (a == node2 && b == node1),
                _ => false,
            })
    }

    fn scale_offset(node_offsets: &[Option<usize>], node: usize) -> Option<usize> {
        node_offsets.get(node).copied().flatten()
    }

    fn add_scale_path_constraint(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        node1: usize,
        node2: usize,
        min_tree_length: TmFloat,
        equality: bool,
    ) {
        let (Some(ix), Some(jx)) = (
            Self::scale_offset(node_offsets, node1),
            Self::scale_offset(node_offsets, node2),
        ) else {
            return;
        };
        let constraint = Box::new(nlco::PathFn1::new(ix, ix + 1, jx, jx + 1, min_tree_length));
        if equality {
            optimizer.add_nonlinear_equality(constraint);
        } else {
            optimizer.add_nonlinear_inequality(constraint);
        }
    }

    fn add_scale_path_active_constraint(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        node1: usize,
        node2: usize,
    ) {
        if let Some(path) = self.find_leaf_path_between(node1, node2) {
            self.add_scale_path_constraint(
                optimizer,
                node_offsets,
                node1,
                node2,
                path.min_tree_length,
                true,
            );
        }
    }

    fn add_scale_condition_constraints(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        kind: &ConditionKind,
    ) {
        match *kind {
            ConditionKind::NodeCombo {
                node,
                to_symmetry_line,
                to_paper_edge,
                to_paper_corner,
                x_fixed,
                x_fix_value,
                y_fixed,
                y_fix_value,
            } => {
                let Some(ix) = Self::scale_offset(node_offsets, node) else {
                    return;
                };
                let iy = ix + 1;
                if self.has_symmetry && to_symmetry_line {
                    optimizer.add_linear_equality(Box::new(nlco::StickToLineFn::new(
                        ix,
                        iy,
                        self.sym_loc.x,
                        self.sym_loc.y,
                        self.sym_angle,
                    )));
                }
                if to_paper_edge {
                    optimizer.add_nonlinear_equality(Box::new(nlco::StickToEdgeFn::new(
                        ix,
                        iy,
                        self.paper_width,
                        self.paper_height,
                    )));
                }
                if to_paper_corner {
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        ix,
                        self.paper_width,
                    )));
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        iy,
                        self.paper_height,
                    )));
                }
                if x_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        ix,
                        -1.0,
                        x_fix_value,
                    )));
                }
                if y_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        iy,
                        -1.0,
                        y_fix_value,
                    )));
                }
            }
            ConditionKind::NodeFixed {
                node,
                x_fixed,
                y_fixed,
                x_fix_value,
                y_fix_value,
            } => {
                let Some(ix) = Self::scale_offset(node_offsets, node) else {
                    return;
                };
                let iy = ix + 1;
                if x_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        ix,
                        -1.0,
                        x_fix_value,
                    )));
                }
                if y_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        iy,
                        -1.0,
                        y_fix_value,
                    )));
                }
            }
            ConditionKind::NodeOnCorner { node } => {
                if let Some(ix) = Self::scale_offset(node_offsets, node) {
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        ix,
                        self.paper_width,
                    )));
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        ix + 1,
                        self.paper_height,
                    )));
                }
            }
            ConditionKind::NodeOnEdge { node } => {
                if let Some(ix) = Self::scale_offset(node_offsets, node) {
                    optimizer.add_nonlinear_equality(Box::new(nlco::StickToEdgeFn::new(
                        ix,
                        ix + 1,
                        self.paper_width,
                        self.paper_height,
                    )));
                }
            }
            ConditionKind::NodeSymmetric { node } => {
                if !self.has_symmetry {
                    return;
                }
                if let Some(ix) = Self::scale_offset(node_offsets, node) {
                    optimizer.add_linear_equality(Box::new(nlco::StickToLineFn::new(
                        ix,
                        ix + 1,
                        self.sym_loc.x,
                        self.sym_loc.y,
                        self.sym_angle,
                    )));
                }
            }
            ConditionKind::NodesPaired { node1, node2 } => {
                if !self.has_symmetry {
                    return;
                }
                let (Some(ix), Some(jx)) = (
                    Self::scale_offset(node_offsets, node1),
                    Self::scale_offset(node_offsets, node2),
                ) else {
                    return;
                };
                optimizer.add_linear_equality(Box::new(nlco::PairFn1A::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    self.sym_angle,
                )));
                optimizer.add_linear_equality(Box::new(nlco::PairFn1B::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    self.sym_loc.x,
                    self.sym_loc.y,
                    self.sym_angle,
                )));
            }
            ConditionKind::NodesCollinear {
                node1,
                node2,
                node3,
            } => {
                let (Some(ix), Some(jx), Some(kx)) = (
                    Self::scale_offset(node_offsets, node1),
                    Self::scale_offset(node_offsets, node2),
                    Self::scale_offset(node_offsets, node3),
                ) else {
                    return;
                };
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn1::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    kx,
                    kx + 1,
                )));
            }
            ConditionKind::EdgeLengthFixed { .. } | ConditionKind::EdgesSameStrain { .. } => {}
            ConditionKind::PathCombo {
                node1,
                node2,
                is_angle_fixed,
                angle,
                is_angle_quant,
                quant,
                quant_offset,
            } => {
                let (Some(ix), Some(jx)) = (
                    Self::scale_offset(node_offsets, node1),
                    Self::scale_offset(node_offsets, node2),
                ) else {
                    return;
                };
                self.add_scale_path_active_constraint(optimizer, node_offsets, node1, node2);
                if is_angle_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::PathAngleFn1::new(
                        ix,
                        ix + 1,
                        jx,
                        jx + 1,
                        angle,
                    )));
                }
                if is_angle_quant {
                    optimizer.add_nonlinear_equality(Box::new(nlco::QuantizeAngleFn1::new(
                        ix,
                        ix + 1,
                        jx,
                        jx + 1,
                        quant,
                        quant_offset,
                    )));
                }
            }
            ConditionKind::PathActive { node1, node2 } => {
                self.add_scale_path_active_constraint(optimizer, node_offsets, node1, node2);
            }
            ConditionKind::PathAngleFixed {
                node1,
                node2,
                angle,
            } => {
                self.add_scale_path_active_constraint(optimizer, node_offsets, node1, node2);
                let (Some(ix), Some(jx)) = (
                    Self::scale_offset(node_offsets, node1),
                    Self::scale_offset(node_offsets, node2),
                ) else {
                    return;
                };
                optimizer.add_linear_equality(Box::new(nlco::PathAngleFn1::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    angle,
                )));
            }
            ConditionKind::PathAngleQuant {
                node1,
                node2,
                quant,
                quant_offset,
            } => {
                self.add_scale_path_active_constraint(optimizer, node_offsets, node1, node2);
                let (Some(ix), Some(jx)) = (
                    Self::scale_offset(node_offsets, node1),
                    Self::scale_offset(node_offsets, node2),
                ) else {
                    return;
                };
                optimizer.add_nonlinear_equality(Box::new(nlco::QuantizeAngleFn1::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    quant,
                    quant_offset,
                )));
            }
        }
    }

    fn moving_nodes_for_edge_optimizer(&self) -> Vec<usize> {
        self.owned_nodes
            .iter()
            .copied()
            .filter(|id| {
                let node = &self.nodes[id - 1];
                node.is_leaf && !node.is_pinned
            })
            .collect()
    }

    fn edge_has_length_fixed_condition(&self, edge: usize) -> bool {
        self.conditions.iter().any(|condition| {
            matches!(condition.kind, ConditionKind::EdgeLengthFixed { edge: e } if e == edge)
        })
    }

    fn stretchy_edges_for_edge_optimizer(&self) -> Vec<usize> {
        self.owned_edges
            .iter()
            .copied()
            .filter(|id| {
                let edge = &self.edges[id - 1];
                !edge.is_pinned && !self.edge_has_length_fixed_condition(*id)
            })
            .collect()
    }

    fn edge_lookup(&self, edge_ids: &[usize]) -> Vec<bool> {
        let mut lookup = vec![false; self.edges.len() + 1];
        for id in edge_ids {
            lookup[*id] = true;
        }
        lookup
    }

    fn node_loc(&self, node: usize) -> Point {
        self.nodes[node - 1].loc
    }

    fn edge_fix_var_lengths(&self, path: &Path, stretchy_lookup: &[bool]) -> (TmFloat, TmFloat) {
        let mut lfix = 0.0;
        let mut lvar = 0.0;
        for edge_id in &path.edges {
            let edge = &self.edges[*edge_id - 1];
            let temp = edge.length * self.scale;
            if stretchy_lookup[*edge_id] {
                lfix += temp;
                lvar += temp;
            } else {
                lfix += (1.0 + edge.strain) * temp;
            }
        }
        (lfix, lvar)
    }

    fn add_edge_path_constraint(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        stretchy_lookup: &[bool],
        path: &Path,
        equality: bool,
    ) {
        let node1 = path.nodes[0];
        let node2 = *path.nodes.last().unwrap();
        let ix = Self::scale_offset(node_offsets, node1);
        let jx = Self::scale_offset(node_offsets, node2);
        let (lfix, lvar) = self.edge_fix_var_lengths(path, stretchy_lookup);
        let constraint: Option<Box<dyn nlco::DifferentiableFn>> = match (ix, jx) {
            (Some(ix), Some(jx)) => Some(Box::new(nlco::StrainPathFn1::new(
                ix,
                ix + 1,
                jx,
                jx + 1,
                lfix,
                lvar,
            ))),
            (Some(ix), None) => {
                let loc = self.node_loc(node2);
                Some(Box::new(nlco::StrainPathFn2::new(
                    ix,
                    ix + 1,
                    loc.x,
                    loc.y,
                    lfix,
                    lvar,
                )))
            }
            (None, Some(jx)) => {
                let loc = self.node_loc(node1);
                Some(Box::new(nlco::StrainPathFn2::new(
                    jx,
                    jx + 1,
                    loc.x,
                    loc.y,
                    lfix,
                    lvar,
                )))
            }
            (None, None) if lvar != 0.0 => {
                let loc1 = self.node_loc(node1);
                let loc2 = self.node_loc(node2);
                Some(Box::new(nlco::StrainPathFn3::new(
                    loc1.x, loc1.y, loc2.x, loc2.y, lfix, lvar,
                )))
            }
            (None, None) => None,
        };
        if let Some(constraint) = constraint {
            if equality {
                optimizer.add_nonlinear_equality(constraint);
            } else {
                optimizer.add_nonlinear_inequality(constraint);
            }
        }
    }

    fn add_edge_path_active_constraint(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        stretchy_lookup: &[bool],
        node1: usize,
        node2: usize,
    ) {
        if let Some(path) = self.find_leaf_path_between(node1, node2) {
            self.add_edge_path_constraint(optimizer, node_offsets, stretchy_lookup, path, true);
        }
    }

    fn add_edge_angle_constraints(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        node1: usize,
        node2: usize,
        fixed_angle: Option<TmFloat>,
        quant: Option<(usize, TmFloat)>,
    ) {
        let ix = Self::scale_offset(node_offsets, node1);
        let jx = Self::scale_offset(node_offsets, node2);
        if let Some(angle) = fixed_angle {
            match (ix, jx) {
                (Some(ix), Some(jx)) => optimizer.add_nonlinear_equality(Box::new(
                    nlco::PathAngleFn1::new(ix, ix + 1, jx, jx + 1, angle),
                )),
                (Some(ix), None) => {
                    let loc = self.node_loc(node2);
                    optimizer.add_nonlinear_equality(Box::new(nlco::PathAngleFn2::new(
                        ix,
                        ix + 1,
                        loc.x,
                        loc.y,
                        angle,
                    )));
                }
                (None, Some(jx)) => {
                    let loc = self.node_loc(node1);
                    optimizer.add_nonlinear_equality(Box::new(nlco::PathAngleFn2::new(
                        jx,
                        jx + 1,
                        loc.x,
                        loc.y,
                        angle,
                    )));
                }
                (None, None) => {}
            }
        }
        if let Some((quant, quant_offset)) = quant {
            match (ix, jx) {
                (Some(ix), Some(jx)) => optimizer.add_nonlinear_equality(Box::new(
                    nlco::QuantizeAngleFn1::new(ix, ix + 1, jx, jx + 1, quant, quant_offset),
                )),
                (Some(ix), None) => {
                    let loc = self.node_loc(node2);
                    optimizer.add_nonlinear_equality(Box::new(nlco::QuantizeAngleFn2::new(
                        ix,
                        ix + 1,
                        loc.x,
                        loc.y,
                        quant,
                        quant_offset,
                    )));
                }
                (None, Some(jx)) => {
                    let loc = self.node_loc(node1);
                    optimizer.add_nonlinear_equality(Box::new(nlco::QuantizeAngleFn2::new(
                        jx,
                        jx + 1,
                        loc.x,
                        loc.y,
                        quant,
                        quant_offset,
                    )));
                }
                (None, None) => {}
            }
        }
    }

    fn add_pair_constraints(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        node1: usize,
        node2: usize,
    ) {
        if !self.has_symmetry {
            return;
        }
        let ix = Self::scale_offset(node_offsets, node1);
        let jx = Self::scale_offset(node_offsets, node2);
        match (ix, jx) {
            (Some(ix), Some(jx)) => {
                optimizer.add_linear_equality(Box::new(nlco::PairFn1A::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    self.sym_angle,
                )));
                optimizer.add_linear_equality(Box::new(nlco::PairFn1B::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    self.sym_loc.x,
                    self.sym_loc.y,
                    self.sym_angle,
                )));
            }
            (Some(ix), None) => {
                let loc = self.node_loc(node2);
                optimizer.add_linear_equality(Box::new(nlco::PairFn2A::new(
                    ix,
                    ix + 1,
                    loc.x,
                    loc.y,
                    self.sym_angle,
                )));
                optimizer.add_linear_equality(Box::new(nlco::PairFn2B::new(
                    ix,
                    ix + 1,
                    loc.x,
                    loc.y,
                    self.sym_loc.x,
                    self.sym_loc.y,
                    self.sym_angle,
                )));
            }
            (None, Some(jx)) => {
                let loc = self.node_loc(node1);
                optimizer.add_linear_equality(Box::new(nlco::PairFn2A::new(
                    jx,
                    jx + 1,
                    loc.x,
                    loc.y,
                    self.sym_angle,
                )));
                optimizer.add_linear_equality(Box::new(nlco::PairFn2B::new(
                    jx,
                    jx + 1,
                    loc.x,
                    loc.y,
                    self.sym_loc.x,
                    self.sym_loc.y,
                    self.sym_angle,
                )));
            }
            (None, None) => {}
        }
    }

    fn add_collinear_constraints(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        node1: usize,
        node2: usize,
        node3: usize,
    ) {
        let ix = Self::scale_offset(node_offsets, node1);
        let jx = Self::scale_offset(node_offsets, node2);
        let kx = Self::scale_offset(node_offsets, node3);
        match (ix, jx, kx) {
            (Some(ix), Some(jx), Some(kx)) => {
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn1::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    kx,
                    kx + 1,
                )));
            }
            (Some(ix), Some(jx), None) => {
                let loc = self.node_loc(node3);
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn2::new(
                    ix,
                    ix + 1,
                    jx,
                    jx + 1,
                    loc.x,
                    loc.y,
                )));
            }
            (Some(ix), None, Some(kx)) => {
                let loc = self.node_loc(node2);
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn2::new(
                    ix,
                    ix + 1,
                    kx,
                    kx + 1,
                    loc.x,
                    loc.y,
                )));
            }
            (None, Some(jx), Some(kx)) => {
                let loc = self.node_loc(node1);
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn2::new(
                    jx,
                    jx + 1,
                    kx,
                    kx + 1,
                    loc.x,
                    loc.y,
                )));
            }
            (Some(ix), None, None) => {
                let loc2 = self.node_loc(node2);
                let loc3 = self.node_loc(node3);
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn3::new(
                    ix,
                    ix + 1,
                    loc2.x,
                    loc2.y,
                    loc3.x,
                    loc3.y,
                )));
            }
            (None, Some(jx), None) => {
                let loc1 = self.node_loc(node1);
                let loc3 = self.node_loc(node3);
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn3::new(
                    jx,
                    jx + 1,
                    loc1.x,
                    loc1.y,
                    loc3.x,
                    loc3.y,
                )));
            }
            (None, None, Some(kx)) => {
                let loc1 = self.node_loc(node1);
                let loc2 = self.node_loc(node2);
                optimizer.add_nonlinear_equality(Box::new(nlco::CollinearFn3::new(
                    kx,
                    kx + 1,
                    loc1.x,
                    loc1.y,
                    loc2.x,
                    loc2.y,
                )));
            }
            (None, None, None) => {}
        }
    }

    fn add_edge_condition_constraints(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        stretchy_lookup: &[bool],
        kind: &ConditionKind,
    ) {
        match *kind {
            ConditionKind::NodeCombo {
                node,
                to_symmetry_line,
                to_paper_edge,
                to_paper_corner,
                x_fixed,
                x_fix_value,
                y_fixed,
                y_fix_value,
            } => {
                let Some(ix) = Self::scale_offset(node_offsets, node) else {
                    return;
                };
                let iy = ix + 1;
                if self.has_symmetry && to_symmetry_line {
                    optimizer.add_linear_equality(Box::new(nlco::StickToLineFn::new(
                        ix,
                        iy,
                        self.sym_loc.x,
                        self.sym_loc.y,
                        self.sym_angle,
                    )));
                }
                if to_paper_edge {
                    optimizer.add_nonlinear_equality(Box::new(nlco::StickToEdgeFn::new(
                        ix,
                        iy,
                        self.paper_width,
                        self.paper_height,
                    )));
                }
                if to_paper_corner {
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        ix,
                        self.paper_width,
                    )));
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        iy,
                        self.paper_height,
                    )));
                }
                if x_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        ix,
                        -1.0,
                        x_fix_value,
                    )));
                }
                if y_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        iy,
                        -1.0,
                        y_fix_value,
                    )));
                }
            }
            ConditionKind::NodeFixed {
                node,
                x_fixed,
                y_fixed,
                x_fix_value,
                y_fix_value,
            } => {
                let Some(ix) = Self::scale_offset(node_offsets, node) else {
                    return;
                };
                if x_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        ix,
                        -1.0,
                        x_fix_value,
                    )));
                }
                if y_fixed {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(
                        ix + 1,
                        -1.0,
                        y_fix_value,
                    )));
                }
            }
            ConditionKind::NodeOnCorner { node } => {
                if let Some(ix) = Self::scale_offset(node_offsets, node) {
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        ix,
                        self.paper_width,
                    )));
                    optimizer.add_nonlinear_equality(Box::new(nlco::CornerFn::new(
                        ix + 1,
                        self.paper_height,
                    )));
                }
            }
            ConditionKind::NodeOnEdge { node } => {
                if let Some(ix) = Self::scale_offset(node_offsets, node) {
                    optimizer.add_nonlinear_equality(Box::new(nlco::StickToEdgeFn::new(
                        ix,
                        ix + 1,
                        self.paper_width,
                        self.paper_height,
                    )));
                }
            }
            ConditionKind::NodeSymmetric { node } => {
                if !self.has_symmetry {
                    return;
                }
                if let Some(ix) = Self::scale_offset(node_offsets, node) {
                    optimizer.add_linear_equality(Box::new(nlco::StickToLineFn::new(
                        ix,
                        ix + 1,
                        self.sym_loc.x,
                        self.sym_loc.y,
                        self.sym_angle,
                    )));
                }
            }
            ConditionKind::NodesPaired { node1, node2 } => {
                self.add_pair_constraints(optimizer, node_offsets, node1, node2);
            }
            ConditionKind::NodesCollinear {
                node1,
                node2,
                node3,
            } => self.add_collinear_constraints(optimizer, node_offsets, node1, node2, node3),
            ConditionKind::EdgeLengthFixed { edge } => {
                if stretchy_lookup[edge] {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(0, 1.0, 0.0)));
                }
            }
            ConditionKind::EdgesSameStrain { .. } => {}
            ConditionKind::PathCombo {
                node1,
                node2,
                is_angle_fixed,
                angle,
                is_angle_quant,
                quant,
                quant_offset,
            } => {
                self.add_edge_path_active_constraint(
                    optimizer,
                    node_offsets,
                    stretchy_lookup,
                    node1,
                    node2,
                );
                self.add_edge_angle_constraints(
                    optimizer,
                    node_offsets,
                    node1,
                    node2,
                    is_angle_fixed.then_some(angle),
                    is_angle_quant.then_some((quant, quant_offset)),
                );
            }
            ConditionKind::PathActive { node1, node2 } => {
                self.add_edge_path_active_constraint(
                    optimizer,
                    node_offsets,
                    stretchy_lookup,
                    node1,
                    node2,
                );
            }
            ConditionKind::PathAngleFixed {
                node1,
                node2,
                angle,
            } => {
                self.add_edge_path_active_constraint(
                    optimizer,
                    node_offsets,
                    stretchy_lookup,
                    node1,
                    node2,
                );
                self.add_edge_angle_constraints(
                    optimizer,
                    node_offsets,
                    node1,
                    node2,
                    Some(angle),
                    None,
                );
            }
            ConditionKind::PathAngleQuant {
                node1,
                node2,
                quant,
                quant_offset,
            } => {
                self.add_edge_path_active_constraint(
                    optimizer,
                    node_offsets,
                    stretchy_lookup,
                    node1,
                    node2,
                );
                self.add_edge_angle_constraints(
                    optimizer,
                    node_offsets,
                    node1,
                    node2,
                    None,
                    Some((quant, quant_offset)),
                );
            }
        }
    }

    fn moving_nodes_for_strain_optimizer(&self) -> Vec<usize> {
        self.owned_nodes
            .iter()
            .copied()
            .filter(|id| self.nodes[id - 1].is_leaf)
            .collect()
    }

    fn strain_fix_var_lengths(
        &self,
        path: &Path,
        edge_offsets: &[Option<usize>],
    ) -> (TmFloat, Vec<usize>, Vec<TmFloat>) {
        let mut lfix = 0.0;
        let mut vi = Vec::new();
        let mut vf = Vec::new();
        for edge_id in &path.edges {
            let edge = &self.edges[*edge_id - 1];
            if let Some(offset) = edge_offsets[*edge_id] {
                vi.push(offset);
                let scaled_length = edge.length * self.scale;
                vf.push(scaled_length);
                lfix += scaled_length;
            } else {
                lfix += edge.strained_length() * self.scale;
            }
        }
        (lfix, vi, vf)
    }

    fn add_strain_path_constraint(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        edge_offsets: &[Option<usize>],
        path: &Path,
        equality: bool,
    ) {
        let node1 = path.nodes[0];
        let node2 = *path.nodes.last().unwrap();
        let ix = Self::scale_offset(node_offsets, node1);
        let jx = Self::scale_offset(node_offsets, node2);
        let (lfix, vi, vf) = self.strain_fix_var_lengths(path, edge_offsets);
        let constraint: Option<Box<dyn nlco::DifferentiableFn>> = match (ix, jx) {
            (Some(ix), Some(jx)) => Some(Box::new(nlco::MultiStrainPathFn1::new(
                ix,
                ix + 1,
                jx,
                jx + 1,
                lfix,
                vi,
                vf,
            ))),
            (Some(ix), None) => {
                let loc = self.node_loc(node2);
                Some(Box::new(nlco::MultiStrainPathFn2::new(
                    ix,
                    ix + 1,
                    loc.x,
                    loc.y,
                    lfix,
                    vi,
                    vf,
                )))
            }
            (None, Some(jx)) => {
                let loc = self.node_loc(node1);
                Some(Box::new(nlco::MultiStrainPathFn2::new(
                    jx,
                    jx + 1,
                    loc.x,
                    loc.y,
                    lfix,
                    vi,
                    vf,
                )))
            }
            (None, None) if !vi.is_empty() => {
                let loc1 = self.node_loc(node1);
                let loc2 = self.node_loc(node2);
                Some(Box::new(nlco::MultiStrainPathFn3::new(
                    loc1.x, loc1.y, loc2.x, loc2.y, lfix, vi, vf,
                )))
            }
            (None, None) => None,
        };
        if let Some(constraint) = constraint {
            if equality {
                optimizer.add_nonlinear_equality(constraint);
            } else {
                optimizer.add_nonlinear_inequality(constraint);
            }
        }
    }

    fn add_strain_path_active_constraint(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        edge_offsets: &[Option<usize>],
        node1: usize,
        node2: usize,
    ) {
        if let Some(path) = self.find_leaf_path_between(node1, node2) {
            self.add_strain_path_constraint(optimizer, node_offsets, edge_offsets, path, true);
        }
    }

    fn add_strain_condition_constraints(
        &self,
        optimizer: &mut nlco::NlcoAlm,
        node_offsets: &[Option<usize>],
        edge_offsets: &[Option<usize>],
        kind: &ConditionKind,
    ) {
        match *kind {
            ConditionKind::EdgeLengthFixed { edge } => {
                if let Some(offset) = edge_offsets[edge] {
                    optimizer.add_linear_equality(Box::new(nlco::OneVarFn::new(offset, 1.0, 0.0)));
                }
            }
            ConditionKind::EdgesSameStrain { edge1, edge2 } => {
                if let (Some(offset1), Some(offset2)) = (edge_offsets[edge1], edge_offsets[edge2]) {
                    optimizer.add_linear_equality(Box::new(nlco::TwoVarFn::new(
                        offset1, 1.0, offset2, -1.0, 0.0,
                    )));
                }
            }
            ConditionKind::PathCombo {
                node1,
                node2,
                is_angle_fixed,
                angle,
                is_angle_quant,
                quant,
                quant_offset,
            } => {
                self.add_strain_path_active_constraint(
                    optimizer,
                    node_offsets,
                    edge_offsets,
                    node1,
                    node2,
                );
                self.add_edge_angle_constraints(
                    optimizer,
                    node_offsets,
                    node1,
                    node2,
                    is_angle_fixed.then_some(angle),
                    is_angle_quant.then_some((quant, quant_offset)),
                );
            }
            ConditionKind::PathActive { node1, node2 } => {
                self.add_strain_path_active_constraint(
                    optimizer,
                    node_offsets,
                    edge_offsets,
                    node1,
                    node2,
                );
            }
            ConditionKind::PathAngleFixed {
                node1,
                node2,
                angle,
            } => {
                self.add_strain_path_active_constraint(
                    optimizer,
                    node_offsets,
                    edge_offsets,
                    node1,
                    node2,
                );
                self.add_edge_angle_constraints(
                    optimizer,
                    node_offsets,
                    node1,
                    node2,
                    Some(angle),
                    None,
                );
            }
            ConditionKind::PathAngleQuant {
                node1,
                node2,
                quant,
                quant_offset,
            } => {
                self.add_strain_path_active_constraint(
                    optimizer,
                    node_offsets,
                    edge_offsets,
                    node1,
                    node2,
                );
                self.add_edge_angle_constraints(
                    optimizer,
                    node_offsets,
                    node1,
                    node2,
                    None,
                    Some((quant, quant_offset)),
                );
            }
            _ => self.add_edge_condition_constraints(
                optimizer,
                node_offsets,
                &self.edge_lookup(&[]),
                kind,
            ),
        }
    }

    fn cleanup_lengths_and_feasibility(&mut self) {
        let node_locs: Vec<Point> = self.nodes.iter().map(|n| n.loc).collect();
        let edge_lengths: Vec<TmFloat> = self.edges.iter().map(Edge::strained_length).collect();
        for path in &mut self.paths {
            path.min_tree_length = path
                .edges
                .iter()
                .filter_map(|id| edge_lengths.get(id.saturating_sub(1)))
                .sum();
            path.min_paper_length = path.min_tree_length * self.scale;
            if path.is_leaf && path.nodes.len() >= 2 {
                let a = node_locs[path.nodes[0] - 1];
                let b = node_locs[*path.nodes.last().unwrap() - 1];
                path.act_paper_length = a.distance(b);
                path.act_tree_length = if self.scale.abs() > DIST_TOL {
                    path.act_paper_length / self.scale
                } else {
                    0.0
                };
                path.is_feasible = path.act_paper_length >= path.min_paper_length - DIST_TOL;
                path.is_active = (path.act_paper_length - path.min_paper_length).abs() < DIST_TOL;
            } else {
                path.act_paper_length = 0.0;
                path.act_tree_length = 0.0;
                path.is_feasible = false;
                path.is_active = false;
            }
        }
        let leaf_paths_feasible = self
            .paths
            .iter()
            .filter(|path| path.is_leaf)
            .all(|path| path.is_feasible);
        let condition_feasibilities: Vec<bool> = self
            .conditions
            .iter()
            .map(|condition| condition.kind.calc_feasibility(self))
            .collect();
        for (condition, feasible) in self
            .conditions
            .iter_mut()
            .zip(condition_feasibilities.iter().copied())
        {
            condition.is_feasible = feasible;
        }
        let conditions_feasible = condition_feasibilities.into_iter().all(|feasible| feasible);
        self.is_feasible = leaf_paths_feasible && conditions_feasible;
        self.needs_cleanup = false;
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn expect_tag(&mut self, expected: &'static str) -> Result<()> {
        let tag = self.read_token("tag")?;
        if tag != expected {
            return Err(self.err(format!("expected tag {expected:?}, found {tag:?}")));
        }
        Ok(())
    }

    fn read_token(&mut self, label: &'static str) -> Result<String> {
        self.skip_leading_ws();
        if self.pos >= self.bytes.len() {
            return Err(self.err(format!("expected {label}, found end of input")));
        }
        let start = self.pos;
        while self.pos < self.bytes.len() && !self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(self.err(format!("expected {label}")));
        }
        let token = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.err(format!("{label} was not UTF-8")))?
            .to_string();
        self.consume_trailing_space();
        Ok(token)
    }

    fn read_line_field(&mut self, label: &'static str) -> Result<String> {
        if self.pos >= self.bytes.len() {
            return Err(self.err(format!("expected {label}, found end of input")));
        }
        let mut out = Vec::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            match b {
                b'\r' => {
                    if self.pos < self.bytes.len() && self.bytes[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                    break;
                }
                b'\n' => break,
                b'\\' => {
                    let Some(next) = self.bytes.get(self.pos).copied() else {
                        return Err(self.err("dangling escape in C string".to_string()));
                    };
                    self.pos += 1;
                    match next {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b'\\' => out.push(b'\\'),
                        other => {
                            return Err(self.err(format!(
                                "bad escape sequence \\{} in {label}",
                                other as char
                            )));
                        }
                    }
                }
                other => out.push(other),
            }
        }
        String::from_utf8(out).map_err(|_| self.err(format!("{label} was not UTF-8")))
    }

    fn read_raw_line(&mut self, label: &'static str) -> Result<String> {
        if self.pos >= self.bytes.len() {
            return Err(self.err(format!("expected {label}, found end of input")));
        }
        let start = self.pos;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'\r' => {
                    let end = self.pos;
                    self.pos += 1;
                    if self.pos < self.bytes.len() && self.bytes[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                    return std::str::from_utf8(&self.bytes[start..end])
                        .map(|s| s.to_string())
                        .map_err(|_| self.err(format!("{label} was not UTF-8")));
                }
                b'\n' => {
                    let end = self.pos;
                    self.pos += 1;
                    return std::str::from_utf8(&self.bytes[start..end])
                        .map(|s| s.to_string())
                        .map_err(|_| self.err(format!("{label} was not UTF-8")));
                }
                _ => self.pos += 1,
            }
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .map(|s| s.to_string())
            .map_err(|_| self.err(format!("{label} was not UTF-8")))
    }

    fn read_usize(&mut self, label: &'static str) -> Result<usize> {
        let offset = self.pos;
        let token = self.read_token(label)?;
        token.parse::<usize>().map_err(|_| TreeError::Parse {
            offset,
            message: format!("expected unsigned integer for {label}, found {token:?}"),
        })
    }

    fn read_f64(&mut self, label: &'static str) -> Result<TmFloat> {
        let offset = self.pos;
        let token = self.read_token(label)?;
        if token.starts_with("NAN") {
            return Ok(0.0);
        }
        token.parse::<TmFloat>().map_err(|_| TreeError::Parse {
            offset,
            message: format!("expected float for {label}, found {token:?}"),
        })
    }

    fn read_bool(&mut self, label: &'static str) -> Result<bool> {
        Ok(self.read_token(label)? == "true")
    }

    fn read_point(&mut self, label: &'static str) -> Result<Point> {
        Ok(Point {
            x: self.read_f64(label)?,
            y: self.read_f64(label)?,
        })
    }

    fn read_index_array(&mut self, label: &'static str) -> Result<Vec<usize>> {
        let n = self.read_usize(label)?;
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            values.push(self.read_usize(label)?);
        }
        Ok(values)
    }

    fn read_optional_index(&mut self, label: &'static str) -> Result<Option<usize>> {
        match self.read_usize(label)? {
            0 => Ok(None),
            n => Ok(Some(n)),
        }
    }

    fn read_node_v4(&mut self) -> Result<Node> {
        self.expect_tag("node")?;
        Ok(Node {
            index: self.read_usize("node index")?,
            label: self.read_line_field("node label")?,
            loc: self.read_point("node location")?,
            depth: DEPTH_NOT_SET,
            elevation: 0.0,
            is_leaf: self.read_bool("node leaf flag")?,
            is_sub: self.read_bool("node sub flag")?,
            is_border: self.read_bool("node border flag")?,
            is_pinned: self.read_bool("node pinned flag")?,
            is_polygon: self.read_bool("node polygon flag")?,
            is_junction: false,
            is_conditioned: self.read_bool("node conditioned flag")?,
            owned_vertices: self.read_index_array("node owned vertices")?,
            edges: self.read_index_array("node edges")?,
            leaf_paths: self.read_index_array("node leaf paths")?,
            owner: self.read_node_owner()?,
        })
    }

    fn read_node_v5(&mut self) -> Result<Node> {
        self.expect_tag("node")?;
        Ok(Node {
            index: self.read_usize("node index")?,
            label: self.read_line_field("node label")?,
            loc: self.read_point("node location")?,
            depth: self.read_f64("node depth")?,
            elevation: self.read_f64("node elevation")?,
            is_leaf: self.read_bool("node leaf flag")?,
            is_sub: self.read_bool("node sub flag")?,
            is_border: self.read_bool("node border flag")?,
            is_pinned: self.read_bool("node pinned flag")?,
            is_polygon: self.read_bool("node polygon flag")?,
            is_junction: self.read_bool("node junction flag")?,
            is_conditioned: self.read_bool("node conditioned flag")?,
            edges: self.read_index_array("node edges")?,
            leaf_paths: self.read_index_array("node leaf paths")?,
            owned_vertices: self.read_index_array("node owned vertices")?,
            owner: self.read_node_owner()?,
        })
    }

    fn read_edge(&mut self, repair_zero_stiffness: bool) -> Result<Edge> {
        self.expect_tag("edge")?;
        let index = self.read_usize("edge index")?;
        let label = self.read_line_field("edge label")?;
        let length = self.read_f64("edge length")?;
        let strain = self.read_f64("edge strain")?;
        let mut stiffness = self.read_f64("edge stiffness")?;
        if repair_zero_stiffness && stiffness == 0.0 {
            stiffness = 1.0;
        }
        let edge = Edge {
            index,
            label,
            length,
            strain,
            stiffness,
            is_pinned: self.read_bool("edge pinned flag")?,
            is_conditioned: self.read_bool("edge conditioned flag")?,
            nodes: self.read_index_array("edge nodes")?,
        };
        Ok(edge)
    }

    fn read_path_v4(&mut self) -> Result<Path> {
        self.expect_tag("path")?;
        Ok(Path {
            index: self.read_usize("path index")?,
            min_tree_length: self.read_f64("path min tree length")?,
            min_paper_length: self.read_f64("path min paper length")?,
            act_tree_length: 0.0,
            act_paper_length: 0.0,
            is_leaf: self.read_bool("path leaf flag")?,
            is_sub: self.read_bool("path sub flag")?,
            is_feasible: false,
            is_active: self.read_bool("path active flag")?,
            is_border: self.read_bool("path border flag")?,
            is_polygon: self.read_bool("path polygon flag")?,
            is_conditioned: self.read_bool("path conditioned flag")?,
            owned_vertices: self.read_index_array("path owned vertices")?,
            fwd_poly: self.read_optional_index("path fwd poly")?,
            bkd_poly: self.read_optional_index("path bkd poly")?,
            nodes: self.read_index_array("path nodes")?,
            edges: self.read_index_array("path edges")?,
            owner: self.read_path_owner()?,
            outset_path: None,
            front_reduction: 0.0,
            back_reduction: 0.0,
            min_depth: DEPTH_NOT_SET,
            min_depth_dist: DEPTH_NOT_SET,
            owned_creases: Vec::new(),
        })
    }

    fn read_path_v5(&mut self) -> Result<Path> {
        self.expect_tag("path")?;
        Ok(Path {
            index: self.read_usize("path index")?,
            min_tree_length: self.read_f64("path min tree length")?,
            min_paper_length: self.read_f64("path min paper length")?,
            act_tree_length: self.read_f64("path actual tree length")?,
            act_paper_length: self.read_f64("path actual paper length")?,
            is_leaf: self.read_bool("path leaf flag")?,
            is_sub: self.read_bool("path sub flag")?,
            is_feasible: self.read_bool("path feasible flag")?,
            is_active: self.read_bool("path active flag")?,
            is_border: self.read_bool("path border flag")?,
            is_polygon: self.read_bool("path polygon flag")?,
            is_conditioned: self.read_bool("path conditioned flag")?,
            fwd_poly: self.read_optional_index("path fwd poly")?,
            bkd_poly: self.read_optional_index("path bkd poly")?,
            nodes: self.read_index_array("path nodes")?,
            edges: self.read_index_array("path edges")?,
            outset_path: self.read_optional_index("path outset")?,
            front_reduction: self.read_f64("path front reduction")?,
            back_reduction: self.read_f64("path back reduction")?,
            min_depth: self.read_f64("path min depth")?,
            min_depth_dist: self.read_f64("path min depth distance")?,
            owned_vertices: self.read_index_array("path owned vertices")?,
            owned_creases: self.read_index_array("path owned creases")?,
            owner: self.read_path_owner()?,
        })
    }

    fn read_poly_v4(&mut self) -> Result<RawPoly> {
        self.expect_tag("poly")?;
        let index = self.read_usize("poly index")?;
        let centroid = self.read_point("poly centroid")?;
        let mut fields = vec![
            fmt_float(centroid.x, 10),
            fmt_float(centroid.y, 10),
            self.read_raw_line("poly node locs")?,
            self.read_raw_line("poly sub flag")?,
        ];
        for _ in 0..8 {
            fields.push(self.read_raw_line("poly field")?);
        }
        Ok(RawPoly {
            index,
            centroid,
            is_sub_poly: false,
            fields,
        })
    }

    fn read_poly_v5(&mut self) -> Result<RawPoly> {
        self.expect_tag("poly")?;
        let index = self.read_usize("poly index")?;
        let centroid = self.read_point("poly centroid")?;
        let is_sub_poly = self.read_bool("poly sub flag")?;
        let mut fields = vec![
            fmt_float(centroid.x, 10),
            fmt_float(centroid.y, 10),
            is_sub_poly.to_string(),
        ];
        for _ in 0..16 {
            fields.push(self.read_raw_line("poly field")?);
        }
        Ok(RawPoly {
            index,
            centroid,
            is_sub_poly,
            fields,
        })
    }

    fn read_vertex_v4(&mut self) -> Result<RawVertex> {
        self.expect_tag("vrtx")?;
        Ok(RawVertex {
            index: 0,
            fields: vec![
                self.read_raw_line("vertex loc x")?,
                self.read_raw_line("vertex loc y")?,
                self.read_raw_line("vertex creases")?,
                self.read_raw_line("vertex owner")?,
            ],
        })
    }

    fn read_vertex_v5(&mut self) -> Result<RawVertex> {
        self.expect_tag("vrtx")?;
        let index = self.read_usize("vertex index")?;
        let mut fields = Vec::new();
        for _ in 0..13 {
            fields.push(self.read_raw_line("vertex field")?);
        }
        Ok(RawVertex { index, fields })
    }

    fn read_crease_v4(&mut self) -> Result<RawCrease> {
        self.expect_tag("crse")?;
        Ok(RawCrease {
            index: 0,
            fields: vec![
                self.read_raw_line("crease kind")?,
                self.read_raw_line("crease vertices")?,
                self.read_raw_line("crease owner")?,
            ],
        })
    }

    fn read_crease_v5(&mut self) -> Result<RawCrease> {
        self.expect_tag("crse")?;
        let index = self.read_usize("crease index")?;
        let mut fields = Vec::new();
        for _ in 0..8 {
            fields.push(self.read_raw_line("crease field")?);
        }
        Ok(RawCrease { index, fields })
    }

    fn read_facet_v5(&mut self) -> Result<RawFacet> {
        self.expect_tag("fact")?;
        let index = self.read_usize("facet index")?;
        let mut fields = Vec::new();
        for _ in 0..11 {
            fields.push(self.read_raw_line("facet field")?);
        }
        Ok(RawFacet { index, fields })
    }

    fn read_condition_v4(&mut self, index: usize) -> Result<Condition> {
        let tag = self.read_token("condition tag")?;
        if tag.len() > 4 {
            return Err(self.err(format!("bad condition tag {tag:?}")));
        }
        let n = self.read_usize("condition line count")?;
        validate_condition_rest_len(&tag, n)?;
        let mut raw_lines = Vec::with_capacity(n);
        for _ in 0..n {
            raw_lines.push(self.read_raw_line("condition field")?);
        }
        let kind = ConditionKind::from_stream(&tag, &raw_lines)?;
        Ok(Condition {
            index,
            is_feasible: true,
            kind,
        })
    }

    fn read_condition_v5(&mut self) -> Result<Condition> {
        let tag = self.read_token("condition tag")?;
        if tag.len() > 4 {
            return Err(self.err(format!("bad condition tag {tag:?}")));
        }
        let index = self.read_usize("condition index")?;
        let is_feasible = self.read_bool("condition feasibility")?;
        let n = self.read_usize("condition line count")?;
        validate_condition_rest_len(&tag, n)?;
        let mut raw_lines = Vec::with_capacity(n);
        for _ in 0..n {
            raw_lines.push(self.read_raw_line("condition field")?);
        }
        let kind = ConditionKind::from_stream(&tag, &raw_lines)?;
        Ok(Condition {
            index,
            is_feasible,
            kind,
        })
    }

    fn read_node_owner(&mut self) -> Result<OwnerRef> {
        if self.read_usize("node owner is poly")? == 1 {
            Ok(OwnerRef::Poly(self.read_usize("node owner poly")?))
        } else {
            Ok(OwnerRef::Tree)
        }
    }

    fn read_path_owner(&mut self) -> Result<OwnerRef> {
        if self.read_usize("path owner is poly")? == 1 {
            Ok(OwnerRef::Poly(self.read_usize("path owner poly")?))
        } else {
            Ok(OwnerRef::Tree)
        }
    }

    fn skip_leading_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn consume_trailing_space(&mut self) {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos] == b' ' || self.bytes[self.pos] == b'\t')
        {
            self.pos += 1;
        }
        if self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'\n' => self.pos += 1,
                b'\r' => {
                    self.pos += 1;
                    if self.pos < self.bytes.len() && self.bytes[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn err(&self, message: String) -> TreeError {
        TreeError::Parse {
            offset: self.pos,
            message,
        }
    }
}

struct Writer {
    precision: usize,
    eol: &'static str,
    out: String,
}

impl Writer {
    fn new(precision: usize, eol: &'static str) -> Self {
        Self {
            precision,
            eol,
            out: String::new(),
        }
    }

    fn finish(self) -> String {
        self.out
    }

    fn line(&mut self, value: impl AsRef<str>) {
        self.out.push_str(value.as_ref());
        self.out.push_str(self.eol);
    }

    fn s(&mut self, value: &str) {
        self.line(value);
    }

    fn cstr(&mut self, value: &str) {
        let mut escaped = String::new();
        for ch in value.chars() {
            match ch {
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\\' => escaped.push_str("\\\\"),
                _ => escaped.push(ch),
            }
        }
        self.line(escaped);
    }

    fn u(&mut self, value: usize) {
        self.line(value.to_string());
    }

    fn f(&mut self, value: TmFloat) {
        self.line(fmt_float(value, self.precision));
    }

    fn b(&mut self, value: bool) {
        self.line(if value { "true" } else { "false" });
    }

    fn point(&mut self, value: Point) {
        self.f(value.x);
        self.f(value.y);
    }

    fn array(&mut self, values: &[usize]) {
        self.u(values.len());
        for value in values {
            self.u(*value);
        }
    }

    fn owner_node_or_tree(&mut self, owner: &OwnerRef) {
        match owner {
            OwnerRef::Poly(id) => {
                self.u(1);
                self.u(*id);
            }
            _ => self.u(0),
        }
    }

    fn node_v4(&mut self, node: &Node) {
        self.s("node");
        self.u(node.index);
        self.cstr(&node.label);
        self.point(node.loc);
        self.b(node.is_leaf);
        self.b(node.is_sub);
        self.b(node.is_border);
        self.b(node.is_pinned);
        self.b(node.is_polygon);
        self.b(node.is_conditioned);
        self.array(&node.owned_vertices);
        self.array(&node.edges);
        self.array(&node.leaf_paths);
        self.owner_node_or_tree(&node.owner);
    }

    fn node_v5(&mut self, node: &Node) {
        self.s("node");
        self.u(node.index);
        self.cstr(&node.label);
        self.point(node.loc);
        self.f(node.depth);
        self.f(node.elevation);
        self.b(node.is_leaf);
        self.b(node.is_sub);
        self.b(node.is_border);
        self.b(node.is_pinned);
        self.b(node.is_polygon);
        self.b(node.is_junction);
        self.b(node.is_conditioned);
        self.array(&node.edges);
        self.array(&node.leaf_paths);
        self.array(&node.owned_vertices);
        self.owner_node_or_tree(&node.owner);
    }

    fn edge(&mut self, edge: &Edge) {
        self.s("edge");
        self.u(edge.index);
        self.cstr(&edge.label);
        self.f(edge.length);
        self.f(edge.strain);
        self.f(edge.stiffness);
        self.b(edge.is_pinned);
        self.b(edge.is_conditioned);
        self.array(&edge.nodes);
    }

    fn path_v4(&mut self, path: &Path) {
        self.s("path");
        self.u(path.index);
        self.f(path.min_tree_length);
        self.f(path.min_paper_length);
        self.b(path.is_leaf);
        self.b(path.is_sub);
        self.b(path.is_active);
        self.b(path.is_border);
        self.b(path.is_polygon);
        self.b(path.is_conditioned);
        self.array(&path.owned_vertices);
        self.u(path.fwd_poly.unwrap_or(0));
        self.u(path.bkd_poly.unwrap_or(0));
        self.array(&path.nodes);
        self.array(&path.edges);
        self.owner_node_or_tree(&path.owner);
    }

    fn path_v5(&mut self, path: &Path) {
        self.s("path");
        self.u(path.index);
        self.f(path.min_tree_length);
        self.f(path.min_paper_length);
        self.f(path.act_tree_length);
        self.f(path.act_paper_length);
        self.b(path.is_leaf);
        self.b(path.is_sub);
        self.b(path.is_feasible);
        self.b(path.is_active);
        self.b(path.is_border);
        self.b(path.is_polygon);
        self.b(path.is_conditioned);
        self.u(path.fwd_poly.unwrap_or(0));
        self.u(path.bkd_poly.unwrap_or(0));
        self.array(&path.nodes);
        self.array(&path.edges);
        self.u(path.outset_path.unwrap_or(0));
        self.f(path.front_reduction);
        self.f(path.back_reduction);
        self.f(path.min_depth);
        self.f(path.min_depth_dist);
        self.array(&path.owned_vertices);
        self.array(&path.owned_creases);
        self.owner_node_or_tree(&path.owner);
    }

    fn raw_part(&mut self, tag: &str, index: usize, fields: &[String]) {
        self.s(tag);
        if index != 0 {
            self.u(index);
        }
        for field in fields {
            self.line(field);
        }
    }

    fn condition_v4(&mut self, condition: &Condition) {
        self.s(condition.kind.tag());
        let lines = condition.kind.stream_lines(self.precision);
        self.u(lines.len());
        for line in &lines {
            self.line(line);
        }
    }

    fn condition_v5(&mut self, condition: &Condition) {
        self.s(condition.kind.tag());
        self.u(condition.index);
        self.b(condition.is_feasible);
        let lines = condition.kind.stream_lines(self.precision);
        self.u(lines.len());
        for line in &lines {
            self.line(line);
        }
    }
}

struct ScaleObjective;

impl nlco::DifferentiableFn for ScaleObjective {
    fn func(&self, x: &[f64]) -> f64 {
        -x[0]
    }

    fn grad(&self, _x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        grad[0] = -1.0;
    }
}

struct StrainObjective {
    edge_offset: usize,
    stiffness: Vec<TmFloat>,
}

impl nlco::DifferentiableFn for StrainObjective {
    fn func(&self, x: &[f64]) -> f64 {
        let mut value = 0.0;
        for (i, x_i) in x.iter().enumerate().skip(self.edge_offset) {
            value += self.stiffness[i - self.edge_offset] * x_i.powi(2);
        }
        value
    }

    fn grad(&self, x: &[f64], grad: &mut [f64]) {
        grad.fill(0.0);
        for (i, x_i) in x.iter().enumerate().skip(self.edge_offset) {
            grad[i] = 2.0 * self.stiffness[i - self.edge_offset] * *x_i;
        }
    }
}

impl ConditionKind {
    fn from_stream(tag: &str, lines: &[String]) -> Result<Self> {
        validate_condition_rest_len(tag, lines.len())?;
        let kind = match tag {
            "CNxn" => Self::NodeCombo {
                node: parse_condition_usize(&lines[0], "condition node")?,
                to_symmetry_line: parse_condition_bool(&lines[1]),
                to_paper_edge: parse_condition_bool(&lines[2]),
                to_paper_corner: parse_condition_bool(&lines[3]),
                x_fixed: parse_condition_bool(&lines[4]),
                x_fix_value: parse_condition_f64(&lines[5], "condition x fixed value")?,
                y_fixed: parse_condition_bool(&lines[6]),
                y_fix_value: parse_condition_f64(&lines[7], "condition y fixed value")?,
            },
            "CNfn" => Self::NodeFixed {
                node: parse_condition_usize(&lines[0], "condition node")?,
                x_fixed: parse_condition_bool(&lines[1]),
                y_fixed: parse_condition_bool(&lines[2]),
                x_fix_value: parse_condition_f64(&lines[3], "condition x fixed value")?,
                y_fix_value: parse_condition_f64(&lines[4], "condition y fixed value")?,
            },
            "CNkn" => Self::NodeOnCorner {
                node: parse_condition_usize(&lines[0], "condition node")?,
            },
            "CNen" => Self::NodeOnEdge {
                node: parse_condition_usize(&lines[0], "condition node")?,
            },
            "CNsn" => Self::NodeSymmetric {
                node: parse_condition_usize(&lines[0], "condition node")?,
            },
            "CNpn" => Self::NodesPaired {
                node1: parse_condition_usize(&lines[0], "condition node 1")?,
                node2: parse_condition_usize(&lines[1], "condition node 2")?,
            },
            "CNcn" => Self::NodesCollinear {
                node1: parse_condition_usize(&lines[0], "condition node 1")?,
                node2: parse_condition_usize(&lines[1], "condition node 2")?,
                node3: parse_condition_usize(&lines[2], "condition node 3")?,
            },
            "CNfe" => Self::EdgeLengthFixed {
                edge: parse_condition_usize(&lines[0], "condition edge")?,
            },
            "CNes" => Self::EdgesSameStrain {
                edge1: parse_condition_usize(&lines[0], "condition edge 1")?,
                edge2: parse_condition_usize(&lines[1], "condition edge 2")?,
            },
            "CNap" => Self::PathActive {
                node1: parse_condition_usize(&lines[0], "condition node 1")?,
                node2: parse_condition_usize(&lines[1], "condition node 2")?,
            },
            "CNfp" => Self::PathAngleFixed {
                node1: parse_condition_usize(&lines[0], "condition node 1")?,
                node2: parse_condition_usize(&lines[1], "condition node 2")?,
                angle: parse_condition_f64(&lines[2], "condition angle")?,
            },
            "CNqp" => Self::PathAngleQuant {
                node1: parse_condition_usize(&lines[0], "condition node 1")?,
                node2: parse_condition_usize(&lines[1], "condition node 2")?,
                quant: parse_condition_usize(&lines[2], "condition quantization")?,
                quant_offset: parse_condition_f64(&lines[3], "condition quantization offset")?,
            },
            "CNxp" => Self::PathCombo {
                node1: parse_condition_usize(&lines[0], "condition node 1")?,
                node2: parse_condition_usize(&lines[1], "condition node 2")?,
                is_angle_fixed: parse_condition_bool(&lines[2]),
                angle: parse_condition_f64(&lines[3], "condition angle")?,
                is_angle_quant: parse_condition_bool(&lines[4]),
                quant: parse_condition_usize(&lines[5], "condition quantization")?,
                quant_offset: parse_condition_f64(&lines[6], "condition quantization offset")?,
            },
            _ => unreachable!("validate_condition_rest_len accepted an unrecognized tag"),
        };
        Ok(kind)
    }

    fn tag(&self) -> &'static str {
        match self {
            Self::NodeCombo { .. } => "CNxn",
            Self::NodeFixed { .. } => "CNfn",
            Self::NodeOnCorner { .. } => "CNkn",
            Self::NodeOnEdge { .. } => "CNen",
            Self::NodeSymmetric { .. } => "CNsn",
            Self::NodesPaired { .. } => "CNpn",
            Self::NodesCollinear { .. } => "CNcn",
            Self::EdgeLengthFixed { .. } => "CNfe",
            Self::EdgesSameStrain { .. } => "CNes",
            Self::PathCombo { .. } => "CNxp",
            Self::PathActive { .. } => "CNap",
            Self::PathAngleFixed { .. } => "CNfp",
            Self::PathAngleQuant { .. } => "CNqp",
        }
    }

    fn stream_lines(&self, precision: usize) -> Vec<String> {
        match self {
            Self::NodeCombo {
                node,
                to_symmetry_line,
                to_paper_edge,
                to_paper_corner,
                x_fixed,
                x_fix_value,
                y_fixed,
                y_fix_value,
            } => vec![
                node.to_string(),
                bool_stream(*to_symmetry_line),
                bool_stream(*to_paper_edge),
                bool_stream(*to_paper_corner),
                bool_stream(*x_fixed),
                fmt_float(*x_fix_value, precision),
                bool_stream(*y_fixed),
                fmt_float(*y_fix_value, precision),
            ],
            Self::NodeFixed {
                node,
                x_fixed,
                y_fixed,
                x_fix_value,
                y_fix_value,
            } => vec![
                node.to_string(),
                bool_stream(*x_fixed),
                bool_stream(*y_fixed),
                fmt_float(*x_fix_value, precision),
                fmt_float(*y_fix_value, precision),
            ],
            Self::NodeOnCorner { node }
            | Self::NodeOnEdge { node }
            | Self::NodeSymmetric { node } => vec![node.to_string()],
            Self::NodesPaired { node1, node2 } | Self::PathActive { node1, node2 } => {
                vec![node1.to_string(), node2.to_string()]
            }
            Self::NodesCollinear {
                node1,
                node2,
                node3,
            } => vec![node1.to_string(), node2.to_string(), node3.to_string()],
            Self::EdgeLengthFixed { edge } => vec![edge.to_string()],
            Self::EdgesSameStrain { edge1, edge2 } => {
                vec![edge1.to_string(), edge2.to_string()]
            }
            Self::PathCombo {
                node1,
                node2,
                is_angle_fixed,
                angle,
                is_angle_quant,
                quant,
                quant_offset,
            } => vec![
                node1.to_string(),
                node2.to_string(),
                bool_stream(*is_angle_fixed),
                fmt_float(*angle, precision),
                bool_stream(*is_angle_quant),
                quant.to_string(),
                fmt_float(*quant_offset, precision),
            ],
            Self::PathAngleFixed {
                node1,
                node2,
                angle,
            } => vec![
                node1.to_string(),
                node2.to_string(),
                fmt_float(*angle, precision),
            ],
            Self::PathAngleQuant {
                node1,
                node2,
                quant,
                quant_offset,
            } => vec![
                node1.to_string(),
                node2.to_string(),
                quant.to_string(),
                fmt_float(*quant_offset, precision),
            ],
        }
    }

    fn validate_refs(&self, tree: &Tree) -> Result<()> {
        match self {
            Self::NodeCombo { node, .. }
            | Self::NodeFixed { node, .. }
            | Self::NodeOnCorner { node }
            | Self::NodeOnEdge { node }
            | Self::NodeSymmetric { node } => tree.check_ref("node", *node, tree.nodes.len()),
            Self::NodesPaired { node1, node2 }
            | Self::PathActive { node1, node2 }
            | Self::PathAngleFixed { node1, node2, .. }
            | Self::PathAngleQuant { node1, node2, .. }
            | Self::PathCombo { node1, node2, .. } => {
                tree.check_ref("node", *node1, tree.nodes.len())?;
                tree.check_ref("node", *node2, tree.nodes.len())
            }
            Self::NodesCollinear {
                node1,
                node2,
                node3,
            } => {
                tree.check_ref("node", *node1, tree.nodes.len())?;
                tree.check_ref("node", *node2, tree.nodes.len())?;
                tree.check_ref("node", *node3, tree.nodes.len())
            }
            Self::EdgeLengthFixed { edge } => tree.check_ref("edge", *edge, tree.edges.len()),
            Self::EdgesSameStrain { edge1, edge2 } => {
                tree.check_ref("edge", *edge1, tree.edges.len())?;
                tree.check_ref("edge", *edge2, tree.edges.len())
            }
        }
    }

    fn calc_feasibility(&self, tree: &Tree) -> bool {
        match self {
            Self::NodeCombo {
                node,
                to_symmetry_line,
                to_paper_edge,
                to_paper_corner,
                x_fixed,
                x_fix_value,
                y_fixed,
                y_fix_value,
            } => {
                let Some(loc) = node_loc(tree, *node) else {
                    return false;
                };
                if tree.has_symmetry
                    && *to_symmetry_line
                    && !is_tiny(stick_to_line(loc, tree.sym_loc, tree.sym_angle))
                {
                    return false;
                }
                if *to_paper_edge
                    && !is_tiny(stick_to_edge(loc, tree.paper_width, tree.paper_height))
                {
                    return false;
                }
                if *to_paper_corner {
                    if !is_tiny(corner_coord(loc.x, tree.paper_width)) {
                        return false;
                    }
                    // TreeMaker 5.0.1's CNxn feasibility check uses paper width
                    // for the y corner coordinate; preserve that behavior.
                    if !is_tiny(corner_coord(loc.y, tree.paper_width)) {
                        return false;
                    }
                }
                if *x_fixed && !is_tiny(*x_fix_value - loc.x) {
                    return false;
                }
                !*y_fixed || is_tiny(*y_fix_value - loc.y)
            }
            Self::NodeFixed {
                node,
                x_fixed,
                y_fixed,
                x_fix_value,
                y_fix_value,
            } => {
                let Some(loc) = node_loc(tree, *node) else {
                    return false;
                };
                if *x_fixed && !is_tiny(*x_fix_value - loc.x) {
                    return false;
                }
                !*y_fixed || is_tiny(*y_fix_value - loc.y)
            }
            Self::NodeOnCorner { node } => node_loc(tree, *node).is_some_and(|loc| {
                is_tiny(corner_coord(loc.x, tree.paper_width))
                    && is_tiny(corner_coord(loc.y, tree.paper_height))
            }),
            Self::NodeOnEdge { node } => node_loc(tree, *node).is_some_and(|loc| {
                is_tiny(stick_to_edge(loc, tree.paper_width, tree.paper_height))
            }),
            Self::NodeSymmetric { node } => {
                tree.has_symmetry
                    && node_loc(tree, *node).is_some_and(|loc| {
                        is_tiny(stick_to_line(loc, tree.sym_loc, tree.sym_angle))
                    })
            }
            Self::NodesPaired { node1, node2 } => {
                if !tree.has_symmetry {
                    return false;
                }
                let (Some(a), Some(b)) = (node_loc(tree, *node1), node_loc(tree, *node2)) else {
                    return false;
                };
                is_tiny(pair_fn_1a(a, b, tree.sym_angle))
                    && is_tiny(pair_fn_1b(a, b, tree.sym_loc, tree.sym_angle))
            }
            Self::NodesCollinear {
                node1,
                node2,
                node3,
            } => {
                let (Some(a), Some(b), Some(c)) = (
                    node_loc(tree, *node1),
                    node_loc(tree, *node2),
                    node_loc(tree, *node3),
                ) else {
                    return false;
                };
                is_tiny(collinear_fn_1(a, b, c))
            }
            Self::EdgeLengthFixed { edge } => {
                edge_ref(tree, *edge).is_some_and(|edge| is_tiny(edge.strain))
            }
            Self::EdgesSameStrain { edge1, edge2 } => {
                let (Some(a), Some(b)) = (edge_ref(tree, *edge1), edge_ref(tree, *edge2)) else {
                    return false;
                };
                is_tiny(a.strain - b.strain)
            }
            Self::PathActive { node1, node2 } => tree
                .find_leaf_path_between(*node1, *node2)
                .is_some_and(|path| path.is_active),
            Self::PathAngleFixed {
                node1,
                node2,
                angle,
            } => path_active_with_nodes(tree, *node1, *node2)
                .is_some_and(|(a, b)| is_tiny(path_angle_fn_1(a, b, *angle))),
            Self::PathAngleQuant {
                node1,
                node2,
                quant,
                quant_offset,
            } => path_active_with_nodes(tree, *node1, *node2)
                .is_some_and(|(a, b)| is_tiny(quantize_angle_fn_1(a, b, *quant, *quant_offset))),
            Self::PathCombo {
                node1,
                node2,
                is_angle_fixed,
                angle,
                is_angle_quant,
                quant,
                quant_offset,
            } => {
                let Some((a, b)) = path_active_with_nodes(tree, *node1, *node2) else {
                    return false;
                };
                if *is_angle_fixed && !is_tiny(path_angle_fn_1(a, b, *angle)) {
                    return false;
                }
                !*is_angle_quant || is_tiny(quantize_angle_fn_1(a, b, *quant, *quant_offset))
            }
        }
    }
}

fn bool_stream(value: bool) -> String {
    (if value { "true" } else { "false" }).to_string()
}

fn parse_condition_usize(value: &str, label: &'static str) -> Result<usize> {
    value.parse::<usize>().map_err(|_| TreeError::Parse {
        offset: 0,
        message: format!("expected unsigned integer for {label}, found {value:?}"),
    })
}

fn parse_condition_f64(value: &str, label: &'static str) -> Result<TmFloat> {
    if value.starts_with("NAN") {
        return Ok(0.0);
    }
    value.parse::<TmFloat>().map_err(|_| TreeError::Parse {
        offset: 0,
        message: format!("expected float for {label}, found {value:?}"),
    })
}

fn parse_condition_bool(value: &str) -> bool {
    value == "true"
}

fn is_tiny(value: TmFloat) -> bool {
    value.abs() < DIST_TOL
}

fn node_loc(tree: &Tree, index: usize) -> Option<Point> {
    tree.nodes.get(index.checked_sub(1)?).map(|node| node.loc)
}

fn edge_ref(tree: &Tree, index: usize) -> Option<&Edge> {
    tree.edges.get(index.checked_sub(1)?)
}

fn path_active_with_nodes(tree: &Tree, node1: usize, node2: usize) -> Option<(Point, Point)> {
    tree.find_leaf_path_between(node1, node2)
        .filter(|path| path.is_active)?;
    Some((node_loc(tree, node1)?, node_loc(tree, node2)?))
}

fn corner_coord(value: TmFloat, width_or_height: TmFloat) -> TmFloat {
    value * (value - width_or_height)
}

fn stick_to_edge(loc: Point, width: TmFloat, height: TmFloat) -> TmFloat {
    10.0 * loc.x * (loc.x - width) * loc.y * (loc.y - height)
}

fn stick_to_line(loc: Point, point: Point, angle: TmFloat) -> TmFloat {
    let radians = angle * DEGREES;
    (-loc.x + point.x) * radians.sin() + (loc.y - point.y) * radians.cos()
}

fn pair_fn_1a(a: Point, b: Point, angle: TmFloat) -> TmFloat {
    let radians = angle * DEGREES;
    (a.x - b.x) * radians.cos() + (a.y - b.y) * radians.sin()
}

fn pair_fn_1b(a: Point, b: Point, point: Point, angle: TmFloat) -> TmFloat {
    let radians = angle * DEGREES;
    (-a.x - b.x + 2.0 * point.x) * radians.sin() + (a.y + b.y - 2.0 * point.y) * radians.cos()
}

fn collinear_fn_1(a: Point, b: Point, c: Point) -> TmFloat {
    (b.y - a.y) * (c.x - b.x) - (c.y - b.y) * (b.x - a.x)
}

fn path_angle_fn_1(a: Point, b: Point, angle: TmFloat) -> TmFloat {
    let radians = angle * DEGREES;
    (a.x - b.x) * radians.sin() + (b.y - a.y) * radians.cos()
}

fn quantize_angle_fn_1(a: Point, b: Point, quant: usize, quant_offset: TmFloat) -> TmFloat {
    if quant == 0 {
        return TmFloat::NAN;
    }
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let r2 = dx * dx + dy * dy;
    let f1 = r2.powf(-0.5 * quant as TmFloat);
    let offset = quant_offset * DEGREES;
    let step = 180.0 * DEGREES / quant as TmFloat;
    let mut f2 = 1.0;
    for k in 0..quant {
        let angle = k as TmFloat * step - offset;
        f2 *= dx * angle.sin() - dy * angle.cos();
    }
    2.0_f64.powi(quant as i32 - 1) * f1 * f2
}

fn fmt_float(value: TmFloat, precision: usize) -> String {
    format!("{value:.precision$}")
}

fn validate_condition_rest_len(tag: &str, got: usize) -> Result<()> {
    let expected = match tag {
        "CNxn" => 8,
        "CNfn" => 5,
        "CNkn" | "CNen" | "CNsn" | "CNfe" => 1,
        "CNpn" | "CNes" | "CNap" => 2,
        "CNcn" | "CNfp" => 3,
        "CNqp" => 4,
        "CNxp" => 7,
        _ => {
            return Err(TreeError::UnsupportedOperation(
                "unrecognized condition tags are not preserved until condition parsing is ported",
            ));
        }
    };
    if got != expected {
        return Err(TreeError::Parse {
            offset: 0,
            message: format!("condition {tag} expected {expected} rest lines, found {got}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_1: &str = include_str!("../../../tests/fixtures/tmModelTester_1.tmd5");
    const FIXTURE_2: &str = include_str!("../../../tests/fixtures/tmModelTester_2.tmd5");
    const FIXTURE_4: &str = include_str!("../../../tests/fixtures/tmModelTester_4.tmd5");
    const FIXTURE_5: &str = include_str!("../../../tests/fixtures/tmModelTester_5.tmd5");

    #[test]
    fn parses_v4_fixture_summary() {
        let tree = Tree::from_tmd_str(FIXTURE_1).unwrap();
        let summary = tree.summary();
        assert_eq!(summary.source_version, "4.0");
        assert_eq!(summary.nodes, 4);
        assert_eq!(summary.edges, 3);
        assert_eq!(summary.paths, 6);
        assert_eq!(summary.leaf_nodes, 3);
        assert_eq!(summary.leaf_paths, 3);
        assert!(summary.is_feasible);
    }

    #[test]
    fn parses_larger_fixtures() {
        let tree2 = Tree::from_tmd_str(FIXTURE_2).unwrap();
        let tree5 = Tree::from_tmd_str(FIXTURE_5).unwrap();
        assert!(tree2.summary().nodes > 10);
        assert!(tree5.summary().conditions > 0);
    }

    #[test]
    fn round_trips_through_v5_writer() {
        let tree = Tree::from_tmd_str(FIXTURE_1).unwrap();
        let serialized = tree.to_tmd5_string();
        let reparsed = Tree::from_tmd_str(&serialized).unwrap();
        assert_eq!(tree.summary().nodes, reparsed.summary().nodes);
        assert_eq!(tree.summary().edges, reparsed.summary().edges);
        assert_eq!(tree.summary().paths, reparsed.summary().paths);
        assert_eq!(tree.summary().leaf_paths, reparsed.summary().leaf_paths);
    }

    #[test]
    fn scale_optimizer_uses_alm_port() {
        let mut tree = Tree::from_tmd_str(FIXTURE_1).unwrap();
        let report = tree.optimize_scale().unwrap();
        assert!(report.converged);
        assert!(tree.is_feasible());
        assert!((report.new_scale - 0.517637).abs() < 1.0e-4);
    }

    #[test]
    fn edge_optimizer_uses_alm_port() {
        let mut tree = Tree::from_tmd_str(FIXTURE_4).unwrap();
        let report = tree.optimize_edges().unwrap();
        assert!(report.converged);
        assert!(tree.is_feasible());
        let max_strain = tree
            .edges
            .iter()
            .map(|edge| edge.strain)
            .fold(TmFloat::NEG_INFINITY, TmFloat::max);
        assert!((max_strain - 0.573142).abs() < 1.0e-4);
    }

    #[test]
    fn strain_optimizer_uses_alm_port() {
        let mut tree = Tree::from_tmd_str(FIXTURE_5).unwrap();
        let report = tree.optimize_strain().unwrap();
        assert!(report.converged);
        assert!(tree.is_feasible());
        let weighted = tree
            .edges
            .iter()
            .map(|edge| edge.stiffness * edge.strain.powi(2))
            .sum::<TmFloat>()
            / tree.edges.len() as TmFloat;
        assert!((100.0 * weighted.sqrt() - 3.580266).abs() < 1.0e-4);
    }

    #[test]
    fn unsupported_algorithm_ports_are_explicit() {
        let mut tree = Tree::from_tmd_str(FIXTURE_1).unwrap();
        assert!(matches!(
            tree.build_polys_and_crease_pattern(),
            Err(TreeError::UnsupportedOperation(_))
        ));
    }
}
