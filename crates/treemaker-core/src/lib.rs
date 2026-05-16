use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod nlco;

pub type TmFloat = f64;

const DIST_TOL: TmFloat = 1.0e-4;
const MIN_EDGE_LENGTH: TmFloat = 0.01;
const DEPTH_NOT_SET: TmFloat = -999.0;
const DEGREES: TmFloat = 0.017453292519943296;
const PI: TmFloat = std::f64::consts::PI;
const TWO_PI: TmFloat = 2.0 * std::f64::consts::PI;
const CONVEXITY_TOL: TmFloat = 1.0e-4;
const MOVE_TOL: TmFloat = 1.0e-6;

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
pub struct Poly {
    pub index: usize,
    pub centroid: Point,
    pub is_sub_poly: bool,
    pub ring_nodes: Vec<usize>,
    pub ring_paths: Vec<usize>,
    pub cross_paths: Vec<usize>,
    pub inset_nodes: Vec<usize>,
    pub spoke_paths: Vec<usize>,
    pub ridge_path: Option<usize>,
    pub node_locs: Vec<Point>,
    pub local_root_vertices: Vec<usize>,
    pub local_root_creases: Vec<usize>,
    pub owned_nodes: Vec<usize>,
    pub owned_paths: Vec<usize>,
    pub owned_polys: Vec<usize>,
    pub owned_creases: Vec<usize>,
    pub owned_facets: Vec<usize>,
    pub owner: OwnerRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub index: usize,
    pub loc: Point,
    pub elevation: TmFloat,
    pub is_border: bool,
    pub tree_node: Option<usize>,
    pub left_pseudohinge_mate: Option<usize>,
    pub right_pseudohinge_mate: Option<usize>,
    pub creases: Vec<usize>,
    pub depth: TmFloat,
    pub discrete_depth: usize,
    pub cc_flag: i32,
    pub st_flag: i32,
    pub owner: OwnerRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crease {
    pub index: usize,
    pub kind: i32,
    pub vertices: Vec<usize>,
    pub fwd_facet: Option<usize>,
    pub bkd_facet: Option<usize>,
    pub fold: i32,
    pub cc_flag: i32,
    pub st_flag: i32,
    pub owner: OwnerRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    pub index: usize,
    pub centroid: Point,
    pub is_well_formed: bool,
    pub vertices: Vec<usize>,
    pub creases: Vec<usize>,
    pub corridor_edge: Option<usize>,
    pub head_facets: Vec<usize>,
    pub tail_facets: Vec<usize>,
    pub order: usize,
    pub color: i32,
    pub owner: OwnerRef,
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
    pub polys: Vec<Poly>,
    pub vertices: Vec<Vertex>,
    pub creases: Vec<Crease>,
    pub facets: Vec<Facet>,
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
    pub feasible_paths: usize,
    pub active_paths: usize,
    pub border_nodes: usize,
    pub border_paths: usize,
    pub polygon_nodes: usize,
    pub polygon_paths: usize,
    pub pinned_nodes: usize,
    pub pinned_edges: usize,
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
                tree.cleanup_after_edit();
                tree
            }
            "5.0" => {
                let tree = Self::read_v5(&mut reader, version)?;
                tree.validate()?;
                tree
            }
            "3.0" => {
                let mut tree = Self::read_v3(&mut reader, version)?;
                tree.validate()?;
                tree.cleanup_after_edit();
                tree
            }
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
            out.poly_v5(poly);
        }
        for vertex in &self.vertices {
            out.vertex_v5(vertex);
        }
        for crease in &self.creases {
            out.crease_v5(crease);
        }
        for facet in &self.facets {
            out.facet_v5(facet);
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
            feasible_paths: self.paths.iter().filter(|p| p.is_feasible).count(),
            active_paths: self.paths.iter().filter(|p| p.is_active).count(),
            border_nodes: self.nodes.iter().filter(|n| n.is_border).count(),
            border_paths: self.paths.iter().filter(|p| p.is_border).count(),
            polygon_nodes: self.nodes.iter().filter(|n| n.is_polygon).count(),
            polygon_paths: self.paths.iter().filter(|p| p.is_polygon).count(),
            pinned_nodes: self.nodes.iter().filter(|n| n.is_pinned).count(),
            pinned_edges: self.edges.iter().filter(|e| e.is_pinned).count(),
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
        self.cleanup_after_edit();

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
        self.cleanup_after_edit();

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
        self.cleanup_after_edit();

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

    fn read_v3(reader: &mut Reader<'_>, source_version: String) -> Result<Self> {
        let paper_width = reader.read_f64("paper width")?;
        let paper_height = reader.read_f64("paper height")?;
        let scale = reader.read_f64("scale")?;
        let has_symmetry = reader.read_bool("has symmetry")?;
        let sym_loc = reader.read_point("symmetry location")?;
        let sym_angle = reader.read_f64("symmetry angle")?;
        let num_nodes = reader.read_usize("node count")?;
        let num_edges = reader.read_usize("edge count")?;
        let num_paths = reader.read_usize("path count")?;
        let _num_polys = reader.read_usize("poly count")?;

        let mut nodes = Vec::with_capacity(num_nodes);
        let mut edges = Vec::with_capacity(num_edges);
        let mut paths = Vec::with_capacity(num_paths);
        let mut conditions = Vec::new();

        for _ in 0..num_nodes {
            nodes.push(reader.read_node_v3(&mut conditions)?);
        }
        for _ in 0..num_edges {
            edges.push(reader.read_edge_v3()?);
        }
        for _ in 0..num_paths {
            paths.push(reader.read_path_v3(&mut conditions)?);
        }

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
            polys: Vec::new(),
            vertices: Vec::new(),
            creases: Vec::new(),
            facets: Vec::new(),
            conditions,
            owned_nodes: (1..=num_nodes).collect(),
            owned_edges: (1..=num_edges).collect(),
            owned_paths: (1..=num_paths).collect(),
            owned_polys: Vec::new(),
        })
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
            paths.push(reader.read_path_v4(num_polys)?);
        }
        for _ in 0..num_polys {
            polys.push(reader.read_poly_v4(num_paths)?);
        }
        for index in 1..=num_vertices {
            vertices.push(reader.read_vertex_v4(index)?);
        }
        for index in 1..=num_creases {
            creases.push(reader.read_crease_v4(index)?);
        }

        let mut conditions = Vec::with_capacity(num_conditions);
        for i in 0..num_conditions {
            conditions.push(reader.read_condition_v4(i + 1)?);
        }

        let owned_nodes = reader.read_index_array("owned nodes")?;
        let owned_edges = reader.read_index_array("owned edges")?;
        let owned_paths = reader.read_index_array("owned paths")?;
        let _owned_polys = reader.read_index_array("owned polys")?;

        kill_v4_crease_pattern_refs(&mut nodes, &mut paths);

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
            polys: Vec::new(),
            vertices: Vec::new(),
            creases: Vec::new(),
            facets: Vec::new(),
            conditions,
            owned_nodes,
            owned_edges,
            owned_paths,
            owned_polys: Vec::new(),
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
            paths.push(reader.read_path_v5(num_polys, num_paths)?);
        }
        for _ in 0..num_polys {
            polys.push(reader.read_poly_v5(num_paths)?);
        }
        for _ in 0..num_vertices {
            vertices.push(reader.read_vertex_v5(num_nodes, num_vertices)?);
        }
        for _ in 0..num_creases {
            creases.push(reader.read_crease_v5(num_facets)?);
        }
        for _ in 0..num_facets {
            facets.push(reader.read_facet_v5(num_edges)?);
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
        for poly in &self.polys {
            for id in &poly.ring_nodes {
                self.check_ref("node", *id, self.nodes.len())?;
            }
            for id in &poly.inset_nodes {
                self.check_ref("node", *id, self.nodes.len())?;
            }
            for id in &poly.owned_nodes {
                self.check_ref("node", *id, self.nodes.len())?;
            }
            for id in poly
                .ring_paths
                .iter()
                .chain(&poly.cross_paths)
                .chain(&poly.spoke_paths)
                .chain(&poly.owned_paths)
            {
                self.check_ref("path", *id, self.paths.len())?;
            }
            if let Some(id) = poly.ridge_path {
                self.check_ref("path", id, self.paths.len())?;
            }
            for id in &poly.local_root_vertices {
                self.check_ref("vertex", *id, self.vertices.len())?;
            }
            for id in poly.local_root_creases.iter().chain(&poly.owned_creases) {
                self.check_ref("crease", *id, self.creases.len())?;
            }
            for id in &poly.owned_polys {
                self.check_ref("poly", *id, self.polys.len())?;
            }
            for id in &poly.owned_facets {
                self.check_ref("facet", *id, self.facets.len())?;
            }
            self.check_owner(&poly.owner)?;
        }
        for vertex in &self.vertices {
            if let Some(id) = vertex.tree_node {
                self.check_ref("node", id, self.nodes.len())?;
            }
            if let Some(id) = vertex.left_pseudohinge_mate {
                self.check_ref("vertex", id, self.vertices.len())?;
            }
            if let Some(id) = vertex.right_pseudohinge_mate {
                self.check_ref("vertex", id, self.vertices.len())?;
            }
            for id in &vertex.creases {
                self.check_ref("crease", *id, self.creases.len())?;
            }
            self.check_owner(&vertex.owner)?;
        }
        for crease in &self.creases {
            for id in &crease.vertices {
                self.check_ref("vertex", *id, self.vertices.len())?;
            }
            if let Some(id) = crease.fwd_facet {
                self.check_ref("facet", id, self.facets.len())?;
            }
            if let Some(id) = crease.bkd_facet {
                self.check_ref("facet", id, self.facets.len())?;
            }
            self.check_owner(&crease.owner)?;
        }
        for facet in &self.facets {
            for id in &facet.vertices {
                self.check_ref("vertex", *id, self.vertices.len())?;
            }
            for id in &facet.creases {
                self.check_ref("crease", *id, self.creases.len())?;
            }
            if let Some(id) = facet.corridor_edge {
                self.check_ref("edge", id, self.edges.len())?;
            }
            for id in facet.head_facets.iter().chain(&facet.tail_facets) {
                self.check_ref("facet", *id, self.facets.len())?;
            }
            self.check_owner(&facet.owner)?;
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

    fn cleanup_after_edit(&mut self) {
        self.is_feasible = false;
        self.is_polygon_valid = false;
        self.is_polygon_filled = false;
        self.is_vertex_depth_valid = false;
        self.is_facet_data_valid = false;

        self.kill_invalid_conditions();

        if self.owned_nodes.is_empty() {
            self.needs_cleanup = false;
            return;
        }

        for node_id in self.owned_nodes.iter().copied() {
            let node = &mut self.nodes[node_id - 1];
            node.loc.x = node.loc.x.clamp(0.0, self.paper_width);
            node.loc.y = node.loc.y.clamp(0.0, self.paper_height);
            node.is_border = false;
            node.is_pinned = false;
            node.is_polygon = false;
            node.is_conditioned = false;
        }

        for edge_id in self.owned_edges.iter().copied() {
            let edge = &mut self.edges[edge_id - 1];
            edge.is_pinned = false;
            edge.is_conditioned = false;
        }

        let node_locs: Vec<Point> = self.nodes.iter().map(|n| n.loc).collect();
        let edge_lengths: Vec<TmFloat> = self.edges.iter().map(Edge::strained_length).collect();
        for path_id in self.owned_paths.iter().copied() {
            let path = &mut self.paths[path_id - 1];
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
                path.act_tree_length = path.act_paper_length / self.scale;
                path.is_feasible = path.act_paper_length >= path.min_paper_length - DIST_TOL;
                path.is_active = (path.act_paper_length - path.min_paper_length).abs() < DIST_TOL;
            } else {
                path.act_paper_length = 0.0;
                path.act_tree_length = 0.0;
                path.is_feasible = false;
                path.is_active = false;
            }
            path.is_border = false;
            path.is_polygon = false;
            path.is_conditioned = false;
        }

        let leaf_nodes = self.leaf_nodes_in_owned_order();
        let leaf_paths = self.leaf_paths_in_owned_order();
        let leaf_paths_feasible = leaf_paths
            .iter()
            .copied()
            .all(|path_id| self.paths[path_id - 1].is_feasible);
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
        self.rebuild_conditioned_flags();
        self.calc_border_nodes_and_paths(&leaf_nodes);
        self.calc_pinned_nodes_and_edges(&leaf_nodes, &leaf_paths);
        self.calc_polygon_network(&leaf_nodes, &leaf_paths);
        self.calc_polygon_validity(&leaf_nodes);
        self.kill_orphan_vertices_and_creases();
        self.promote_first_tree_node_to_root();
        self.renumber_part_indices();
        self.clear_crease_pattern_cleanup_data();
        self.calc_polygon_filled();
        self.needs_cleanup = false;
    }

    fn kill_invalid_conditions(&mut self) {
        let mut conditions = std::mem::take(&mut self.conditions);
        conditions.retain(|condition| self.condition_is_valid(&condition.kind));
        for (i, condition) in conditions.iter_mut().enumerate() {
            condition.index = i + 1;
        }
        self.conditions = conditions;
    }

    fn condition_is_valid(&self, kind: &ConditionKind) -> bool {
        let node_is_leaf = |node: usize| {
            node.checked_sub(1)
                .and_then(|index| self.nodes.get(index))
                .is_some_and(|node| node.is_leaf)
        };
        let node_exists = |node: usize| node > 0 && node <= self.nodes.len();
        let edge_exists = |edge: usize| edge > 0 && edge <= self.edges.len();
        let path_exists_between =
            |node1: usize, node2: usize| self.find_leaf_path_between(node1, node2).is_some();

        match *kind {
            ConditionKind::NodeCombo { node, .. }
            | ConditionKind::NodeFixed { node, .. }
            | ConditionKind::NodeOnCorner { node }
            | ConditionKind::NodeOnEdge { node }
            | ConditionKind::NodeSymmetric { node } => node_is_leaf(node),
            ConditionKind::NodesPaired { node1, node2 } => {
                node_is_leaf(node1) && node_is_leaf(node2)
            }
            ConditionKind::NodesCollinear {
                node1,
                node2,
                node3,
            } => node_is_leaf(node1) && node_is_leaf(node2) && node_is_leaf(node3),
            ConditionKind::EdgeLengthFixed { edge } => edge_exists(edge),
            ConditionKind::EdgesSameStrain { edge1, edge2 } => {
                edge_exists(edge1) && edge_exists(edge2)
            }
            ConditionKind::PathActive { node1, node2 }
            | ConditionKind::PathAngleFixed { node1, node2, .. }
            | ConditionKind::PathAngleQuant { node1, node2, .. } => {
                node_exists(node1) && node_exists(node2) && path_exists_between(node1, node2)
            }
            ConditionKind::PathCombo { node1, node2, .. } => {
                node_exists(node1) && node_exists(node2) && path_exists_between(node1, node2)
            }
        }
    }

    fn leaf_paths_in_owned_order(&self) -> Vec<usize> {
        self.owned_paths
            .iter()
            .copied()
            .filter(|id| self.paths[id - 1].is_leaf)
            .collect()
    }

    fn calc_border_nodes_and_paths(&mut self, leaf_nodes: &[usize]) {
        if leaf_nodes.len() < 3 {
            return;
        }

        let start_pt = Point { x: -1.0, y: -1.0 };
        let Some((&start_node, _)) = leaf_nodes
            .iter()
            .map(|id| (id, angle(point_sub(self.nodes[*id - 1].loc, start_pt))))
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
        else {
            return;
        };

        let mut border_nodes = vec![start_node];
        let mut prev_node = start_node;
        let mut prev_pt = start_pt;
        let mut this_node = start_node;
        let mut this_pt = self.nodes[this_node - 1].loc;

        loop {
            let mut best_node = None;
            let mut best_angle = TWO_PI;
            let mut best_dist = TmFloat::INFINITY;

            for node_id in leaf_nodes.iter().copied() {
                if node_id == prev_node || node_id == this_node {
                    continue;
                }
                let the_pt = self.nodes[node_id - 1].loc;
                let the_angle = angle_change(prev_pt, this_pt, the_pt);
                if the_angle < -PI / 2.0 {
                    continue;
                }
                let the_dist = this_pt.distance(the_pt);
                if the_angle < best_angle - CONVEXITY_TOL
                    || ((the_angle - best_angle).abs() < CONVEXITY_TOL && the_dist < best_dist)
                {
                    best_node = Some(node_id);
                    best_angle = the_angle;
                    best_dist = the_dist;
                }
            }

            let Some(next_node) = best_node else {
                return;
            };
            if next_node == start_node {
                break;
            }
            border_nodes.push(next_node);
            prev_node = this_node;
            prev_pt = this_pt;
            this_node = next_node;
            this_pt = self.nodes[this_node - 1].loc;
        }

        if let Some(first) = border_nodes.first().copied() {
            self.nodes[first - 1].is_border = true;
        }
        for i in 1..border_nodes.len() {
            let prev = border_nodes[i - 1];
            let next = border_nodes[i];
            self.nodes[next - 1].is_border = true;
            if let Some(path_id) = self.leaf_path_id_between(prev, next) {
                self.paths[path_id - 1].is_border = true;
            }
        }
        if let Some(path_id) = self.leaf_path_id_between(
            *border_nodes.last().unwrap(),
            *border_nodes.first().unwrap(),
        ) {
            self.paths[path_id - 1].is_border = true;
        }
    }

    fn calc_pinned_nodes_and_edges(&mut self, leaf_nodes: &[usize], leaf_paths: &[usize]) {
        for node_id in leaf_nodes.iter().copied() {
            self.nodes[node_id - 1].is_pinned = self.calc_is_pinned_node(node_id);
        }

        for path_id in leaf_paths.iter().copied() {
            let path = &self.paths[path_id - 1];
            if !path.is_active || path.nodes.len() < 2 {
                continue;
            }
            let node1 = path.nodes[0];
            let node2 = *path.nodes.last().unwrap();
            if self.nodes[node1 - 1].is_pinned && self.nodes[node2 - 1].is_pinned {
                let edge_ids = path.edges.clone();
                for edge_id in edge_ids {
                    self.edges[edge_id - 1].is_pinned = true;
                }
            }
        }
    }

    fn calc_is_pinned_node(&self, node_id: usize) -> bool {
        let node = &self.nodes[node_id - 1];
        let mut angles = Vec::new();
        for path_id in &node.leaf_paths {
            let path = &self.paths[*path_id - 1];
            if !path.is_active || path.nodes.len() < 2 {
                continue;
            }
            let other = if path.nodes[0] == node_id {
                *path.nodes.last().unwrap()
            } else {
                path.nodes[0]
            };
            angles.push(angle(point_sub(self.nodes[other - 1].loc, node.loc)));
        }

        if is_tiny(node.loc.x) {
            angles.push(-PI);
        }
        if is_tiny(node.loc.x - self.paper_width) {
            angles.push(0.0);
        }
        if is_tiny(node.loc.y - self.paper_height) {
            angles.push(PI / 2.0);
        }
        if is_tiny(node.loc.y) {
            angles.push(-PI / 2.0);
        }

        if angles.len() < 2 {
            return false;
        }
        angles.sort_by(TmFloat::total_cmp);
        for i in 0..angles.len() - 1 {
            if angles[i + 1] - angles[i] > PI + CONVEXITY_TOL {
                return false;
            }
        }
        angles[0] - angles[angles.len() - 1] + PI <= CONVEXITY_TOL
    }

    fn calc_polygon_network(&mut self, leaf_nodes: &[usize], leaf_paths: &[usize]) {
        for path_id in leaf_paths.iter().copied() {
            let path = &mut self.paths[path_id - 1];
            path.is_polygon = path.is_active || (path.is_border && path.is_feasible);
        }

        for node_id in leaf_nodes.iter().copied() {
            let node = &mut self.nodes[node_id - 1];
            if node.is_pinned || node.is_border {
                node.is_polygon = true;
            }
        }

        for path_id in leaf_paths.iter().copied() {
            let path = &self.paths[path_id - 1];
            if path.is_feasible || path.nodes.len() < 2 {
                continue;
            }
            let node1 = path.nodes[0];
            let node2 = *path.nodes.last().unwrap();
            self.nodes[node1 - 1].is_polygon = false;
            self.nodes[node1 - 1].is_pinned = false;
            self.nodes[node2 - 1].is_polygon = false;
            self.nodes[node2 - 1].is_pinned = false;
        }

        loop {
            let mut something_changed = false;

            for path_id in leaf_paths.iter().copied() {
                let path = &self.paths[path_id - 1];
                if !path.is_polygon || path.nodes.len() < 2 {
                    continue;
                }
                let node1 = path.nodes[0];
                let node2 = *path.nodes.last().unwrap();
                if !(self.nodes[node1 - 1].is_polygon && self.nodes[node2 - 1].is_polygon) {
                    self.paths[path_id - 1].is_polygon = false;
                    something_changed = true;
                }
            }

            for node_id in leaf_nodes.iter().copied() {
                if !self.nodes[node_id - 1].is_polygon {
                    continue;
                }
                let poly_paths = self.nodes[node_id - 1]
                    .leaf_paths
                    .iter()
                    .filter(|path_id| self.paths[**path_id - 1].is_polygon)
                    .count();
                if poly_paths < 2 {
                    self.nodes[node_id - 1].is_polygon = false;
                    something_changed = true;
                }
            }

            if !something_changed {
                break;
            }
        }

        let doomed_polys: Vec<usize> = self
            .polys
            .iter()
            .filter(|poly| !self.calc_poly_is_valid(poly.index, leaf_nodes))
            .map(|poly| poly.index)
            .collect();
        if !doomed_polys.is_empty() {
            self.delete_polys(&doomed_polys);
        }
    }

    fn calc_polygon_validity(&mut self, leaf_nodes: &[usize]) {
        self.is_polygon_valid = true;
        for node_id in leaf_nodes.iter().copied() {
            let polygon_paths = self.nodes[node_id - 1]
                .leaf_paths
                .iter()
                .filter(|path_id| self.paths[**path_id - 1].is_polygon)
                .count();
            if polygon_paths < 2 {
                self.is_polygon_valid = false;
                return;
            }
        }

        for path_id in self.owned_paths.iter().copied() {
            let path = &self.paths[path_id - 1];
            if !path.is_polygon {
                continue;
            }
            if path.is_border {
                if path.fwd_poly.is_none() && path.bkd_poly.is_none() {
                    self.is_polygon_valid = false;
                    return;
                }
            } else if path.fwd_poly.is_none() || path.bkd_poly.is_none() {
                self.is_polygon_valid = false;
                return;
            }
        }
    }

    fn calc_poly_is_valid(&self, poly_id: usize, leaf_nodes: &[usize]) -> bool {
        let Some(poly) = self.polys.get(poly_id.saturating_sub(1)) else {
            return false;
        };
        if poly.is_sub_poly {
            return true;
        }
        if poly.node_locs.len() != poly.ring_nodes.len() {
            return false;
        }
        for (i, node_id) in poly.ring_nodes.iter().copied().enumerate() {
            let Some(node) = self.nodes.get(node_id.saturating_sub(1)) else {
                return false;
            };
            if poly.node_locs[i].distance(node.loc) > MOVE_TOL {
                return false;
            }
        }
        for path_id in poly.ring_paths.iter().copied() {
            let Some(path) = self.paths.get(path_id.saturating_sub(1)) else {
                return false;
            };
            if !path.is_polygon {
                return false;
            }
        }
        if !self.poly_is_convex(poly) {
            return false;
        }
        !self.poly_encloses_leaf_node(poly, leaf_nodes)
    }

    fn poly_is_convex(&self, poly: &Poly) -> bool {
        let n = poly.ring_nodes.len();
        if n < 3 {
            return false;
        }
        for i in 0..n - 2 {
            let Some(p1) = self
                .nodes
                .get(poly.ring_nodes[i].saturating_sub(1))
                .map(|node| node.loc)
            else {
                return false;
            };
            let Some(p2) = self
                .nodes
                .get(poly.ring_nodes[(i + 1) % n].saturating_sub(1))
                .map(|node| node.loc)
            else {
                return false;
            };
            let Some(p3) = self
                .nodes
                .get(poly.ring_nodes[(i + 2) % n].saturating_sub(1))
                .map(|node| node.loc)
            else {
                return false;
            };
            if angle_change(p1, p2, p3) < -CONVEXITY_TOL {
                return false;
            }
        }
        true
    }

    fn poly_encloses_leaf_node(&self, poly: &Poly, leaf_nodes: &[usize]) -> bool {
        for node_id in leaf_nodes.iter().copied() {
            if poly.ring_nodes.contains(&node_id) {
                continue;
            }
            let Some(node) = self.nodes.get(node_id.saturating_sub(1)) else {
                continue;
            };
            if self.poly_convex_encloses(poly, node.loc) {
                return true;
            }
        }
        false
    }

    fn poly_convex_encloses(&self, poly: &Poly, point: Point) -> bool {
        for path_id in poly.ring_paths.iter().copied() {
            let Some(path) = self.paths.get(path_id.saturating_sub(1)) else {
                return false;
            };
            let Some((node1, node2)) = path.nodes.first().zip(path.nodes.last()) else {
                return false;
            };
            let Some(p1) = self.nodes.get(node1.saturating_sub(1)).map(|node| node.loc) else {
                return false;
            };
            let Some(p2) = self.nodes.get(node2.saturating_sub(1)).map(|node| node.loc) else {
                return false;
            };
            let mut q = rotate_ccw90(point_sub(p2, p1));
            if inner(point_sub(poly.centroid, p1), q) < 0.0 {
                q.x *= -1.0;
                q.y *= -1.0;
            }
            if inner(point_sub(point, p1), q) < 0.0 {
                return false;
            }
        }
        true
    }

    fn kill_orphan_vertices_and_creases(&mut self) {
        let mut doomed_creases: Vec<usize> = self
            .creases
            .iter()
            .filter(|crease| self.crease_is_orphan(crease))
            .map(|crease| crease.index)
            .collect();

        let doomed_vertices: Vec<usize> = self
            .vertices
            .iter()
            .filter(|vertex| self.vertex_is_orphan(vertex))
            .map(|vertex| vertex.index)
            .collect();

        for crease in &self.creases {
            if crease
                .vertices
                .iter()
                .any(|vertex_id| doomed_vertices.contains(vertex_id))
            {
                doomed_creases.push(crease.index);
            }
        }
        doomed_creases.sort_unstable();
        doomed_creases.dedup();

        self.delete_creases(&doomed_creases);
        self.delete_vertices(&doomed_vertices);
    }

    fn crease_is_orphan(&self, crease: &Crease) -> bool {
        match crease.owner {
            OwnerRef::Poly(poly_id) => poly_id > 0 && poly_id <= self.polys.len(),
            OwnerRef::Path(path_id) => self
                .paths
                .get(path_id.saturating_sub(1))
                .is_none_or(|path| !path.is_sub && !self.path_is_incident_to_filled_poly(path_id)),
            _ => true,
        }
    }

    fn vertex_is_orphan(&self, vertex: &Vertex) -> bool {
        match vertex.owner {
            OwnerRef::Node(node_id) => {
                let Some(node) = self.nodes.get(node_id.saturating_sub(1)) else {
                    return true;
                };
                if node.is_sub {
                    return false;
                }
                if !node.is_leaf {
                    return true;
                }
                !node
                    .leaf_paths
                    .iter()
                    .copied()
                    .any(|path_id| self.path_is_incident_to_filled_poly(path_id))
            }
            OwnerRef::Path(path_id) => self
                .paths
                .get(path_id.saturating_sub(1))
                .is_none_or(|path| !path.is_sub && !self.path_is_incident_to_filled_poly(path_id)),
            _ => true,
        }
    }

    fn path_is_incident_to_filled_poly(&self, path_id: usize) -> bool {
        let Some(path) = self.paths.get(path_id.saturating_sub(1)) else {
            return false;
        };
        path.fwd_poly
            .and_then(|poly_id| self.polys.get(poly_id.saturating_sub(1)))
            .is_some_and(|poly| !poly.owned_nodes.is_empty())
            || path
                .bkd_poly
                .and_then(|poly_id| self.polys.get(poly_id.saturating_sub(1)))
                .is_some_and(|poly| !poly.owned_nodes.is_empty())
    }

    fn delete_polys(&mut self, doomed: &[usize]) {
        if doomed.is_empty() {
            return;
        }

        let mut doomed_flags = ids_to_flags(doomed, self.polys.len());
        loop {
            let mut changed = false;
            for poly in &self.polys {
                if doomed_flags[poly.index] {
                    continue;
                }
                if let OwnerRef::Poly(owner) = poly.owner
                    && doomed_flags.get(owner).copied().unwrap_or(false)
                {
                    doomed_flags[poly.index] = true;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let doomed_polys: Vec<usize> = doomed_flags
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(id, doomed)| doomed.then_some(id))
            .collect();
        let doomed_facets: Vec<usize> = self
            .facets
            .iter()
            .filter_map(|facet| match facet.owner {
                OwnerRef::Poly(poly_id) if doomed_flags.get(poly_id).copied().unwrap_or(false) => {
                    Some(facet.index)
                }
                _ => None,
            })
            .collect();
        let doomed_creases: Vec<usize> = self
            .creases
            .iter()
            .filter_map(|crease| match crease.owner {
                OwnerRef::Poly(poly_id) if doomed_flags.get(poly_id).copied().unwrap_or(false) => {
                    Some(crease.index)
                }
                _ => None,
            })
            .collect();

        self.delete_facets(&doomed_facets);
        self.delete_creases(&doomed_creases);

        let map = keep_map(self.polys.len(), &doomed_polys);
        for path in &mut self.paths {
            remap_option(&mut path.fwd_poly, &map);
            remap_option(&mut path.bkd_poly, &map);
        }
        for poly in &mut self.polys {
            remap_vec(&mut poly.owned_polys, &map);
            remap_owner(&mut poly.owner, PartKind::Poly, &map);
        }
        for node in &mut self.nodes {
            remap_owner(&mut node.owner, PartKind::Poly, &map);
        }
        for path in &mut self.paths {
            remap_owner(&mut path.owner, PartKind::Poly, &map);
        }
        for crease in &mut self.creases {
            remap_owner(&mut crease.owner, PartKind::Poly, &map);
        }
        for facet in &mut self.facets {
            remap_owner(&mut facet.owner, PartKind::Poly, &map);
        }
        remap_vec(&mut self.owned_polys, &map);

        self.polys = self
            .polys
            .drain(..)
            .filter(|poly| map[poly.index].is_some())
            .enumerate()
            .map(|(i, mut poly)| {
                poly.index = i + 1;
                poly
            })
            .collect();
    }

    fn delete_facets(&mut self, doomed: &[usize]) {
        if doomed.is_empty() {
            return;
        }
        let map = keep_map(self.facets.len(), doomed);
        for crease in &mut self.creases {
            remap_option(&mut crease.fwd_facet, &map);
            remap_option(&mut crease.bkd_facet, &map);
        }
        for poly in &mut self.polys {
            remap_vec(&mut poly.owned_facets, &map);
        }
        for facet in &mut self.facets {
            remap_vec(&mut facet.head_facets, &map);
            remap_vec(&mut facet.tail_facets, &map);
        }
        self.facets = self
            .facets
            .drain(..)
            .filter(|facet| map[facet.index].is_some())
            .enumerate()
            .map(|(i, mut facet)| {
                facet.index = i + 1;
                facet
            })
            .collect();
    }

    fn delete_creases(&mut self, doomed: &[usize]) {
        if doomed.is_empty() {
            return;
        }
        let map = keep_map(self.creases.len(), doomed);
        for vertex in &mut self.vertices {
            remap_vec(&mut vertex.creases, &map);
        }
        for path in &mut self.paths {
            remap_vec(&mut path.owned_creases, &map);
        }
        for poly in &mut self.polys {
            remap_vec(&mut poly.local_root_creases, &map);
            remap_vec(&mut poly.owned_creases, &map);
        }
        for facet in &mut self.facets {
            remap_vec(&mut facet.creases, &map);
        }
        self.creases = self
            .creases
            .drain(..)
            .filter(|crease| map[crease.index].is_some())
            .enumerate()
            .map(|(i, mut crease)| {
                crease.index = i + 1;
                crease
            })
            .collect();
    }

    fn delete_vertices(&mut self, doomed: &[usize]) {
        if doomed.is_empty() {
            return;
        }
        let map = keep_map(self.vertices.len(), doomed);
        for node in &mut self.nodes {
            remap_vec(&mut node.owned_vertices, &map);
        }
        for path in &mut self.paths {
            remap_vec(&mut path.owned_vertices, &map);
        }
        for poly in &mut self.polys {
            remap_vec(&mut poly.local_root_vertices, &map);
        }
        for vertex in &mut self.vertices {
            remap_option(&mut vertex.left_pseudohinge_mate, &map);
            remap_option(&mut vertex.right_pseudohinge_mate, &map);
        }
        for crease in &mut self.creases {
            remap_vec(&mut crease.vertices, &map);
        }
        for facet in &mut self.facets {
            remap_vec(&mut facet.vertices, &map);
        }
        self.vertices = self
            .vertices
            .drain(..)
            .filter(|vertex| map[vertex.index].is_some())
            .enumerate()
            .map(|(i, mut vertex)| {
                vertex.index = i + 1;
                vertex
            })
            .collect();
    }

    fn promote_first_tree_node_to_root(&mut self) {
        let Some(pos) = self.nodes.iter().position(|node| !node.is_sub) else {
            return;
        };
        if pos == 0 {
            return;
        }
        let old_order: Vec<usize> = std::iter::once(pos + 1)
            .chain((1..=self.nodes.len()).filter(|id| *id != pos + 1))
            .collect();
        let mut map = vec![None; self.nodes.len() + 1];
        for (new_index, old_index) in old_order.iter().copied().enumerate() {
            map[old_index] = Some(new_index + 1);
        }

        let old_nodes = self.nodes.clone();
        self.nodes = old_order
            .iter()
            .map(|old_index| old_nodes[*old_index - 1].clone())
            .collect();

        for node in &mut self.nodes {
            node.index = map[node.index].expect("node reorder map");
            remap_owner(&mut node.owner, PartKind::Node, &map);
        }
        for edge in &mut self.edges {
            remap_vec(&mut edge.nodes, &map);
        }
        for path in &mut self.paths {
            remap_vec(&mut path.nodes, &map);
        }
        for poly in &mut self.polys {
            remap_vec(&mut poly.ring_nodes, &map);
            remap_vec(&mut poly.inset_nodes, &map);
            remap_vec(&mut poly.owned_nodes, &map);
        }
        for vertex in &mut self.vertices {
            remap_option(&mut vertex.tree_node, &map);
            remap_owner(&mut vertex.owner, PartKind::Node, &map);
        }
        for condition in &mut self.conditions {
            condition.kind.remap_nodes(&map);
        }
        remap_vec(&mut self.owned_nodes, &map);
    }

    fn renumber_part_indices(&mut self) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.index = i + 1;
        }
        for (i, edge) in self.edges.iter_mut().enumerate() {
            edge.index = i + 1;
        }
        for (i, path) in self.paths.iter_mut().enumerate() {
            path.index = i + 1;
        }
        for (i, poly) in self.polys.iter_mut().enumerate() {
            poly.index = i + 1;
        }
        for (i, vertex) in self.vertices.iter_mut().enumerate() {
            vertex.index = i + 1;
        }
        for (i, crease) in self.creases.iter_mut().enumerate() {
            crease.index = i + 1;
        }
        for (i, facet) in self.facets.iter_mut().enumerate() {
            facet.index = i + 1;
        }
        for (i, condition) in self.conditions.iter_mut().enumerate() {
            condition.index = i + 1;
        }
    }

    fn clear_crease_pattern_cleanup_data(&mut self) {
        for vertex in &mut self.vertices {
            vertex.depth = DEPTH_NOT_SET;
            vertex.discrete_depth = usize::MAX;
        }
        for crease in &mut self.creases {
            crease.fold = 0;
        }
        for facet in &mut self.facets {
            facet.corridor_edge = None;
            facet.head_facets.clear();
            facet.tail_facets.clear();
            facet.order = usize::MAX;
            facet.color = 0;
        }
    }

    fn calc_polygon_filled(&mut self) {
        self.is_polygon_filled = false;
        if self.owned_polys.is_empty() {
            return;
        }
        for poly_id in self.owned_polys.iter().copied() {
            let Some(poly) = self.polys.get(poly_id.saturating_sub(1)) else {
                return;
            };
            if poly.owned_nodes.is_empty() {
                return;
            }
        }
        self.is_polygon_filled = true;
    }

    fn leaf_path_id_between(&self, node1: usize, node2: usize) -> Option<usize> {
        self.paths
            .iter()
            .find(|path| {
                path.is_leaf
                    && matches!(
                        path.nodes.first().copied().zip(path.nodes.last().copied()),
                        Some((a, b)) if (a == node1 && b == node2) || (a == node2 && b == node1)
                    )
            })
            .map(|path| path.index)
    }

    fn rebuild_conditioned_flags(&mut self) {
        let mut conditioned_nodes = vec![false; self.nodes.len()];
        let mut conditioned_edges = vec![false; self.edges.len()];
        let mut conditioned_paths = vec![false; self.paths.len()];

        for condition in &self.conditions {
            condition.kind.collect_conditioned_parts(
                self,
                &mut conditioned_nodes,
                &mut conditioned_edges,
                &mut conditioned_paths,
            );
        }

        for (node, conditioned) in self.nodes.iter_mut().zip(conditioned_nodes) {
            node.is_conditioned = conditioned;
        }
        for (edge, conditioned) in self.edges.iter_mut().zip(conditioned_edges) {
            edge.is_conditioned = conditioned;
        }
        for (path, conditioned) in self.paths.iter_mut().zip(conditioned_paths) {
            path.is_conditioned = conditioned;
        }
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

    fn read_i32(&mut self, label: &'static str) -> Result<i32> {
        let offset = self.pos;
        let token = self.read_token(label)?;
        token.parse::<i32>().map_err(|_| TreeError::Parse {
            offset,
            message: format!("expected integer for {label}, found {token:?}"),
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

    fn read_optional_index_in_range(
        &mut self,
        label: &'static str,
        max: usize,
    ) -> Result<Option<usize>> {
        match self.read_usize(label)? {
            0 => Ok(None),
            n if n <= max => Ok(Some(n)),
            _ => Ok(None),
        }
    }

    fn read_point_array(&mut self, label: &'static str) -> Result<Vec<Point>> {
        let n = self.read_usize(label)?;
        let mut points = Vec::with_capacity(n);
        for _ in 0..n {
            points.push(self.read_point(label)?);
        }
        Ok(points)
    }

    fn read_node_v3(&mut self, conditions: &mut Vec<Condition>) -> Result<Node> {
        self.expect_tag("node")?;
        let index = self.read_usize("node index")?;
        let label = self.read_line_field("node label")?;
        let loc = self.read_point("node location")?;

        let node_is_symmetric = self.read_bool("node symmetric flag")?;
        if node_is_symmetric {
            push_condition(conditions, ConditionKind::NodeSymmetric { node: index });
        }

        let node_is_paired = self.read_bool("node paired flag")?;
        let paired_node = self.read_usize("paired node")?;
        if node_is_paired && index > paired_node {
            push_condition(
                conditions,
                ConditionKind::NodesPaired {
                    node1: index,
                    node2: paired_node,
                },
            );
        }

        let x_fixed = self.read_bool("node x fixed flag")?;
        let y_fixed = self.read_bool("node y fixed flag")?;
        let x_fix_value = self.read_f64("node x fixed value")?;
        let y_fix_value = self.read_f64("node y fixed value")?;
        if x_fixed || y_fixed {
            push_condition(
                conditions,
                ConditionKind::NodeFixed {
                    node: index,
                    x_fixed,
                    y_fixed,
                    x_fix_value: if x_fixed { x_fix_value } else { 0.0 },
                    y_fix_value: if y_fixed { y_fix_value } else { 0.0 },
                },
            );
        }

        let node_stick_to_edge = self.read_bool("node stick-to-edge flag")?;
        if node_stick_to_edge {
            push_condition(conditions, ConditionKind::NodeOnEdge { node: index });
        }

        let node_is_collinear = self.read_bool("node collinear flag")?;
        let collinear_node1 = self.read_usize("collinear node 1")?;
        let collinear_node2 = self.read_usize("collinear node 2")?;
        if node_is_collinear && index > collinear_node1 && index > collinear_node2 {
            push_condition(
                conditions,
                ConditionKind::NodesCollinear {
                    node1: index,
                    node2: collinear_node1,
                    node3: collinear_node2,
                },
            );
        }

        Ok(Node {
            index,
            label,
            loc,
            depth: DEPTH_NOT_SET,
            elevation: 0.0,
            is_leaf: self.read_bool("node leaf flag")?,
            is_sub: false,
            is_border: self.read_bool("node border flag")?,
            is_pinned: self.read_bool("node pinned flag")?,
            is_polygon: self.read_bool("node polygon flag")?,
            is_junction: false,
            is_conditioned: false,
            owned_vertices: Vec::new(),
            edges: self.read_index_array("node edges")?,
            leaf_paths: self.read_index_array("node leaf paths")?,
            owner: OwnerRef::Tree,
        })
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

    fn read_edge_v3(&mut self) -> Result<Edge> {
        self.expect_tag("edge")?;
        Ok(Edge {
            index: self.read_usize("edge index")?,
            label: self.read_line_field("edge label")?,
            length: self.read_f64("edge length")?,
            strain: 0.0,
            stiffness: 0.0,
            is_pinned: self.read_bool("edge pinned flag")?,
            is_conditioned: false,
            nodes: self.read_index_array("edge nodes")?,
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

    fn read_path_v3(&mut self, conditions: &mut Vec<Condition>) -> Result<Path> {
        self.expect_tag("path")?;
        let index = self.read_usize("path index")?;
        let min_tree_length = self.read_f64("path min tree length")?;
        let path_fixed_length = self.read_bool("path fixed length flag")?;
        let path_fixed_length_value = self.read_f64("path fixed length value")?;
        let path_fixed_angle = self.read_bool("path fixed angle flag")?;
        let _path_fixed_angle_value = self.read_f64("path fixed angle value")?;
        let is_leaf = self.read_bool("path leaf flag")?;
        let is_active = self.read_bool("path active flag")?;
        let is_border = self.read_bool("path border flag")?;
        let is_polygon = self.read_bool("path polygon flag")?;
        let _legacy_fwd_poly = self.read_usize("path fwd poly")?;
        let _legacy_bkd_poly = self.read_usize("path bkd poly")?;
        let nodes = self.read_index_array("path nodes")?;
        let edges = self.read_index_array("path edges")?;

        if let Some((node1, node2)) = nodes.first().copied().zip(nodes.last().copied()) {
            if path_fixed_length && min_tree_length == path_fixed_length_value {
                push_condition(conditions, ConditionKind::PathActive { node1, node2 });
            }
            if path_fixed_angle {
                // TreeMaker 5.0.1 reads the serialized angle value, but assigns
                // the fixed-angle boolean to the condition angle in v3 import.
                push_condition(
                    conditions,
                    ConditionKind::PathAngleFixed {
                        node1,
                        node2,
                        angle: 1.0,
                    },
                );
            }
        }

        Ok(Path {
            index,
            min_tree_length,
            min_paper_length: 0.0,
            act_tree_length: 0.0,
            act_paper_length: 0.0,
            is_leaf,
            is_sub: false,
            is_feasible: false,
            is_active,
            is_border,
            is_polygon,
            is_conditioned: false,
            fwd_poly: None,
            bkd_poly: None,
            nodes,
            edges,
            outset_path: None,
            front_reduction: 0.0,
            back_reduction: 0.0,
            min_depth: DEPTH_NOT_SET,
            min_depth_dist: DEPTH_NOT_SET,
            owned_vertices: Vec::new(),
            owned_creases: Vec::new(),
            owner: OwnerRef::Tree,
        })
    }

    fn read_path_v4(&mut self, poly_count: usize) -> Result<Path> {
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
            fwd_poly: self.read_optional_index_in_range("path fwd poly", poly_count)?,
            bkd_poly: self.read_optional_index_in_range("path bkd poly", poly_count)?,
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

    fn read_path_v5(&mut self, poly_count: usize, path_count: usize) -> Result<Path> {
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
            fwd_poly: self.read_optional_index_in_range("path fwd poly", poly_count)?,
            bkd_poly: self.read_optional_index_in_range("path bkd poly", poly_count)?,
            nodes: self.read_index_array("path nodes")?,
            edges: self.read_index_array("path edges")?,
            outset_path: self.read_optional_index_in_range("path outset", path_count)?,
            front_reduction: self.read_f64("path front reduction")?,
            back_reduction: self.read_f64("path back reduction")?,
            min_depth: self.read_f64("path min depth")?,
            min_depth_dist: self.read_f64("path min depth distance")?,
            owned_vertices: self.read_index_array("path owned vertices")?,
            owned_creases: self.read_index_array("path owned creases")?,
            owner: self.read_path_owner()?,
        })
    }

    fn read_poly_v4(&mut self, path_count: usize) -> Result<Poly> {
        self.expect_tag("poly")?;
        let index = self.read_usize("poly index")?;
        let centroid = self.read_point("poly centroid")?;
        let node_locs = self.read_point_array("poly node locations")?;
        let is_sub_poly = self.read_bool("poly sub flag")?;
        let owned_nodes = self.read_index_array("poly owned nodes")?;
        let owned_paths = self.read_index_array("poly owned paths")?;
        let owned_polys = self.read_index_array("poly owned polys")?;
        let owned_creases = self.read_index_array("poly owned creases")?;
        let ring_nodes = self.read_index_array("poly ring nodes")?;
        let ring_paths = self.read_index_array("poly ring paths")?;
        let cross_paths = self.read_index_array("poly cross paths")?;
        let inset_nodes = self.read_index_array("poly inset nodes")?;
        let spoke_paths = self.read_index_array("poly spoke paths")?;
        let ridge_path = self.read_optional_index_in_range("poly ridge path", path_count)?;
        let owner = self.read_poly_owner()?;
        Ok(Poly {
            index,
            centroid,
            is_sub_poly,
            ring_nodes,
            ring_paths,
            cross_paths,
            inset_nodes,
            spoke_paths,
            ridge_path,
            node_locs,
            local_root_vertices: Vec::new(),
            local_root_creases: Vec::new(),
            owned_nodes,
            owned_paths,
            owned_polys,
            owned_creases,
            owned_facets: Vec::new(),
            owner,
        })
    }

    fn read_poly_v5(&mut self, path_count: usize) -> Result<Poly> {
        self.expect_tag("poly")?;
        let index = self.read_usize("poly index")?;
        let centroid = self.read_point("poly centroid")?;
        let is_sub_poly = self.read_bool("poly sub flag")?;
        Ok(Poly {
            index,
            centroid,
            is_sub_poly,
            ring_nodes: self.read_index_array("poly ring nodes")?,
            ring_paths: self.read_index_array("poly ring paths")?,
            cross_paths: self.read_index_array("poly cross paths")?,
            inset_nodes: self.read_index_array("poly inset nodes")?,
            spoke_paths: self.read_index_array("poly spoke paths")?,
            ridge_path: self.read_optional_index_in_range("poly ridge path", path_count)?,
            node_locs: self.read_point_array("poly node locations")?,
            local_root_vertices: self.read_index_array("poly local root vertices")?,
            local_root_creases: self.read_index_array("poly local root creases")?,
            owned_nodes: self.read_index_array("poly owned nodes")?,
            owned_paths: self.read_index_array("poly owned paths")?,
            owned_polys: self.read_index_array("poly owned polys")?,
            owned_creases: self.read_index_array("poly owned creases")?,
            owned_facets: self.read_index_array("poly owned facets")?,
            owner: self.read_poly_owner()?,
        })
    }

    fn read_vertex_v4(&mut self, index: usize) -> Result<Vertex> {
        self.expect_tag("vrtx")?;
        Ok(Vertex {
            index,
            loc: self.read_point("vertex location")?,
            elevation: 0.0,
            is_border: false,
            tree_node: None,
            left_pseudohinge_mate: None,
            right_pseudohinge_mate: None,
            creases: self.read_index_array("vertex creases")?,
            depth: DEPTH_NOT_SET,
            discrete_depth: 0,
            cc_flag: 0,
            st_flag: 0,
            owner: self.read_vertex_owner()?,
        })
    }

    fn read_vertex_v5(&mut self, node_count: usize, vertex_count: usize) -> Result<Vertex> {
        self.expect_tag("vrtx")?;
        let index = self.read_usize("vertex index")?;
        Ok(Vertex {
            index,
            loc: self.read_point("vertex location")?,
            elevation: self.read_f64("vertex elevation")?,
            is_border: self.read_bool("vertex border flag")?,
            tree_node: self.read_optional_index_in_range("vertex tree node", node_count)?,
            left_pseudohinge_mate: self
                .read_optional_index_in_range("vertex left pseudohinge mate", vertex_count)?,
            right_pseudohinge_mate: self
                .read_optional_index_in_range("vertex right pseudohinge mate", vertex_count)?,
            creases: self.read_index_array("vertex creases")?,
            depth: self.read_f64("vertex depth")?,
            discrete_depth: self.read_usize("vertex discrete depth")?,
            cc_flag: self.read_i32("vertex cc flag")?,
            st_flag: self.read_i32("vertex st flag")?,
            owner: self.read_vertex_owner()?,
        })
    }

    fn read_crease_v4(&mut self, index: usize) -> Result<Crease> {
        self.expect_tag("crse")?;
        Ok(Crease {
            index,
            kind: self.read_i32("crease kind")?,
            vertices: self.read_index_array("crease vertices")?,
            fwd_facet: None,
            bkd_facet: None,
            fold: 0,
            cc_flag: 0,
            st_flag: 0,
            owner: self.read_crease_owner()?,
        })
    }

    fn read_crease_v5(&mut self, facet_count: usize) -> Result<Crease> {
        self.expect_tag("crse")?;
        let index = self.read_usize("crease index")?;
        Ok(Crease {
            index,
            kind: self.read_i32("crease kind")?,
            vertices: self.read_index_array("crease vertices")?,
            fwd_facet: self.read_optional_index_in_range("crease fwd facet", facet_count)?,
            bkd_facet: self.read_optional_index_in_range("crease bkd facet", facet_count)?,
            fold: self.read_i32("crease fold")?,
            cc_flag: self.read_i32("crease cc flag")?,
            st_flag: self.read_i32("crease st flag")?,
            owner: self.read_crease_owner()?,
        })
    }

    fn read_facet_v5(&mut self, edge_count: usize) -> Result<Facet> {
        self.expect_tag("fact")?;
        let index = self.read_usize("facet index")?;
        Ok(Facet {
            index,
            centroid: self.read_point("facet centroid")?,
            is_well_formed: self.read_bool("facet well-formed flag")?,
            vertices: self.read_index_array("facet vertices")?,
            creases: self.read_index_array("facet creases")?,
            corridor_edge: self.read_optional_index_in_range("facet corridor edge", edge_count)?,
            head_facets: self.read_index_array("facet head facets")?,
            tail_facets: self.read_index_array("facet tail facets")?,
            order: self.read_usize("facet order")?,
            color: self.read_i32("facet color")?,
            owner: self.read_facet_owner()?,
        })
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

    fn read_poly_owner(&mut self) -> Result<OwnerRef> {
        if self.read_usize("poly owner is poly")? == 1 {
            Ok(OwnerRef::Poly(self.read_usize("poly owner poly")?))
        } else {
            Ok(OwnerRef::Tree)
        }
    }

    fn read_vertex_owner(&mut self) -> Result<OwnerRef> {
        if self.read_usize("vertex owner is node")? == 1 {
            Ok(OwnerRef::Node(self.read_usize("vertex owner node")?))
        } else {
            Ok(OwnerRef::Path(self.read_usize("vertex owner path")?))
        }
    }

    fn read_crease_owner(&mut self) -> Result<OwnerRef> {
        if self.read_usize("crease owner is poly")? == 1 {
            Ok(OwnerRef::Poly(self.read_usize("crease owner poly")?))
        } else {
            Ok(OwnerRef::Path(self.read_usize("crease owner path")?))
        }
    }

    fn read_facet_owner(&mut self) -> Result<OwnerRef> {
        Ok(OwnerRef::Poly(self.read_usize("facet owner poly")?))
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

    fn i(&mut self, value: i32) {
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

    fn point_array(&mut self, values: &[Point]) {
        self.u(values.len());
        for value in values {
            self.point(*value);
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

    fn owner_vertex(&mut self, owner: &OwnerRef) {
        match owner {
            OwnerRef::Node(id) => {
                self.u(1);
                self.u(*id);
            }
            OwnerRef::Path(id) => {
                self.u(0);
                self.u(*id);
            }
            _ => {
                self.u(0);
                self.u(0);
            }
        }
    }

    fn owner_crease(&mut self, owner: &OwnerRef) {
        match owner {
            OwnerRef::Poly(id) => {
                self.u(1);
                self.u(*id);
            }
            OwnerRef::Path(id) => {
                self.u(0);
                self.u(*id);
            }
            _ => {
                self.u(0);
                self.u(0);
            }
        }
    }

    fn owner_facet(&mut self, owner: &OwnerRef) {
        match owner {
            OwnerRef::Poly(id) => self.u(*id),
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

    fn poly_v5(&mut self, poly: &Poly) {
        self.s("poly");
        self.u(poly.index);
        self.point(poly.centroid);
        self.b(poly.is_sub_poly);
        self.array(&poly.ring_nodes);
        self.array(&poly.ring_paths);
        self.array(&poly.cross_paths);
        self.array(&poly.inset_nodes);
        self.array(&poly.spoke_paths);
        self.u(poly.ridge_path.unwrap_or(0));
        self.point_array(&poly.node_locs);
        self.array(&poly.local_root_vertices);
        self.array(&poly.local_root_creases);
        self.array(&poly.owned_nodes);
        self.array(&poly.owned_paths);
        self.array(&poly.owned_polys);
        self.array(&poly.owned_creases);
        self.array(&poly.owned_facets);
        self.owner_node_or_tree(&poly.owner);
    }

    fn vertex_v5(&mut self, vertex: &Vertex) {
        self.s("vrtx");
        self.u(vertex.index);
        self.point(vertex.loc);
        self.f(vertex.elevation);
        self.b(vertex.is_border);
        self.u(vertex.tree_node.unwrap_or(0));
        self.u(vertex.left_pseudohinge_mate.unwrap_or(0));
        self.u(vertex.right_pseudohinge_mate.unwrap_or(0));
        self.array(&vertex.creases);
        self.f(vertex.depth);
        self.u(vertex.discrete_depth);
        self.i(vertex.cc_flag);
        self.i(vertex.st_flag);
        self.owner_vertex(&vertex.owner);
    }

    fn crease_v5(&mut self, crease: &Crease) {
        self.s("crse");
        self.u(crease.index);
        self.i(crease.kind);
        self.array(&crease.vertices);
        self.u(crease.fwd_facet.unwrap_or(0));
        self.u(crease.bkd_facet.unwrap_or(0));
        self.i(crease.fold);
        self.i(crease.cc_flag);
        self.i(crease.st_flag);
        self.owner_crease(&crease.owner);
    }

    fn facet_v5(&mut self, facet: &Facet) {
        self.s("fact");
        self.u(facet.index);
        self.point(facet.centroid);
        self.b(facet.is_well_formed);
        self.array(&facet.vertices);
        self.array(&facet.creases);
        self.u(facet.corridor_edge.unwrap_or(0));
        self.array(&facet.head_facets);
        self.array(&facet.tail_facets);
        self.u(facet.order);
        self.i(facet.color);
        self.owner_facet(&facet.owner);
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

    fn collect_conditioned_parts(
        &self,
        tree: &Tree,
        nodes: &mut [bool],
        edges: &mut [bool],
        paths: &mut [bool],
    ) {
        match *self {
            Self::NodeCombo { node, .. }
            | Self::NodeFixed { node, .. }
            | Self::NodeOnCorner { node }
            | Self::NodeOnEdge { node }
            | Self::NodeSymmetric { node } => mark_1_based(nodes, node),
            Self::NodesPaired { node1, node2 } => {
                mark_1_based(nodes, node1);
                mark_1_based(nodes, node2);
            }
            Self::PathActive { node1, node2 }
            | Self::PathAngleFixed { node1, node2, .. }
            | Self::PathAngleQuant { node1, node2, .. }
            | Self::PathCombo { node1, node2, .. } => {
                mark_1_based(nodes, node1);
                mark_1_based(nodes, node2);
                if let Some(path) = tree.find_leaf_path_between(node1, node2) {
                    mark_1_based(paths, path.index);
                }
            }
            Self::NodesCollinear {
                node1,
                node2,
                node3,
            } => {
                mark_1_based(nodes, node1);
                mark_1_based(nodes, node2);
                mark_1_based(nodes, node3);
            }
            Self::EdgeLengthFixed { edge } => mark_1_based(edges, edge),
            Self::EdgesSameStrain { edge1, edge2 } => {
                mark_1_based(edges, edge1);
                mark_1_based(edges, edge2);
            }
        }
    }

    fn remap_nodes(&mut self, map: &[Option<usize>]) {
        match self {
            Self::NodeCombo { node, .. }
            | Self::NodeFixed { node, .. }
            | Self::NodeOnCorner { node }
            | Self::NodeOnEdge { node }
            | Self::NodeSymmetric { node } => remap_value(node, map),
            Self::NodesPaired { node1, node2 }
            | Self::PathActive { node1, node2 }
            | Self::PathAngleFixed { node1, node2, .. }
            | Self::PathAngleQuant { node1, node2, .. }
            | Self::PathCombo { node1, node2, .. } => {
                remap_value(node1, map);
                remap_value(node2, map);
            }
            Self::NodesCollinear {
                node1,
                node2,
                node3,
            } => {
                remap_value(node1, map);
                remap_value(node2, map);
                remap_value(node3, map);
            }
            Self::EdgeLengthFixed { .. } | Self::EdgesSameStrain { .. } => {}
        }
    }
}

fn push_condition(conditions: &mut Vec<Condition>, kind: ConditionKind) {
    conditions.push(Condition {
        index: conditions.len() + 1,
        is_feasible: true,
        kind,
    });
}

fn kill_v4_crease_pattern_refs(nodes: &mut [Node], paths: &mut [Path]) {
    for node in nodes {
        node.owned_vertices.clear();
        if matches!(node.owner, OwnerRef::Poly(_)) {
            node.owner = OwnerRef::Tree;
        }
    }
    for path in paths {
        path.owned_vertices.clear();
        path.owned_creases.clear();
        path.fwd_poly = None;
        path.bkd_poly = None;
        path.outset_path = None;
        if matches!(path.owner, OwnerRef::Poly(_)) {
            path.owner = OwnerRef::Tree;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartKind {
    Node,
    Path,
    Poly,
}

fn ids_to_flags(ids: &[usize], len: usize) -> Vec<bool> {
    let mut flags = vec![false; len + 1];
    for id in ids {
        if *id > 0 && *id <= len {
            flags[*id] = true;
        }
    }
    flags
}

fn keep_map(len: usize, doomed: &[usize]) -> Vec<Option<usize>> {
    let doomed = ids_to_flags(doomed, len);
    let mut map = vec![None; len + 1];
    let mut next = 1;
    for old in 1..=len {
        if !doomed[old] {
            map[old] = Some(next);
            next += 1;
        }
    }
    map
}

fn remap_value(value: &mut usize, map: &[Option<usize>]) {
    if let Some(Some(mapped)) = map.get(*value) {
        *value = *mapped;
    }
}

fn remap_option(value: &mut Option<usize>, map: &[Option<usize>]) {
    *value = value.and_then(|id| map.get(id).copied().flatten());
}

fn remap_vec(values: &mut Vec<usize>, map: &[Option<usize>]) {
    let mapped = values
        .iter()
        .filter_map(|id| map.get(*id).copied().flatten())
        .collect();
    *values = mapped;
}

fn remap_owner(owner: &mut OwnerRef, kind: PartKind, map: &[Option<usize>]) {
    match owner {
        OwnerRef::Node(id) if kind == PartKind::Node => {
            if let Some(mapped) = map.get(*id).copied().flatten() {
                *id = mapped;
            } else {
                *owner = OwnerRef::Tree;
            }
        }
        OwnerRef::Path(id) if kind == PartKind::Path => {
            if let Some(mapped) = map.get(*id).copied().flatten() {
                *id = mapped;
            } else {
                *owner = OwnerRef::Tree;
            }
        }
        OwnerRef::Poly(id) if kind == PartKind::Poly => {
            if let Some(mapped) = map.get(*id).copied().flatten() {
                *id = mapped;
            } else {
                *owner = OwnerRef::Tree;
            }
        }
        _ => {}
    }
}

fn mark_1_based(flags: &mut [bool], index: usize) {
    if let Some(flag) = index.checked_sub(1).and_then(|pos| flags.get_mut(pos)) {
        *flag = true;
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

fn point_sub(a: Point, b: Point) -> Point {
    Point {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn rotate_ccw90(p: Point) -> Point {
    Point { x: -p.y, y: p.x }
}

fn inner(a: Point, b: Point) -> TmFloat {
    a.x * b.x + a.y * b.y
}

fn angle(p: Point) -> TmFloat {
    p.y.atan2(p.x)
}

fn angle_change(p1: Point, p2: Point, p3: Point) -> TmFloat {
    let a = angle(point_sub(p3, p2)) - angle(point_sub(p2, p1));
    if a < -PI {
        a + TWO_PI
    } else if a >= PI {
        a - TWO_PI
    } else {
        a
    }
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
    const FIXTURE_V3: &str = include_str!("../../../tests/fixtures/minimal_v3.tmd");
    const FIXTURE_CP_V4: &str = include_str!("../../../tests/fixtures/minimal_cp_v4.tmd4");
    const FIXTURE_CP_V5: &str = include_str!("../../../tests/fixtures/minimal_cp_v5.tmd5");

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
    fn parses_v3_and_translates_legacy_conditions() {
        let tree = Tree::from_tmd_str(FIXTURE_V3).unwrap();
        let summary = tree.summary();
        assert_eq!(summary.source_version, "3.0");
        assert_eq!(summary.nodes, 2);
        assert_eq!(summary.edges, 1);
        assert_eq!(summary.paths, 1);
        assert_eq!(summary.conditions, 3);
        assert_eq!(summary.conditioned_nodes, 2);
        assert_eq!(summary.conditioned_paths, 1);
        assert_eq!(summary.conditions_by_tag.get("CNfn"), Some(&2));
        assert_eq!(summary.conditions_by_tag.get("CNap"), Some(&1));
        assert!(summary.is_feasible);
    }

    #[test]
    fn v3_fixed_angle_preserves_501_import_behavior() {
        let text = FIXTURE_V3.replacen("false\n0.0000000000\ntrue", "true\n45.0000000000\ntrue", 1);
        let tree = Tree::from_tmd_str(&text).unwrap();
        let angle = tree.conditions.iter().find_map(|condition| {
            if let ConditionKind::PathAngleFixed { angle, .. } = condition.kind {
                Some(angle)
            } else {
                None
            }
        });
        assert_eq!(angle, Some(1.0));
    }

    #[test]
    fn parses_and_round_trips_v5_crease_pattern_payload() {
        let tree = Tree::from_tmd_str(FIXTURE_CP_V5).unwrap();
        let summary = tree.summary();
        assert_eq!(summary.source_version, "5.0");
        assert_eq!(summary.polys, 1);
        assert_eq!(summary.vertices, 2);
        assert_eq!(summary.creases, 1);
        assert_eq!(summary.facets, 1);
        assert_eq!(tree.polys[0].ring_nodes, vec![1, 2]);
        assert_eq!(tree.vertices[0].owner, OwnerRef::Node(1));
        assert_eq!(tree.creases[0].owner, OwnerRef::Path(1));
        assert_eq!(tree.facets[0].owner, OwnerRef::Poly(1));

        let reparsed = Tree::from_tmd_str(&tree.to_tmd5_string()).unwrap();
        assert_eq!(reparsed.summary().polys, 1);
        assert_eq!(reparsed.summary().vertices, 2);
        assert_eq!(reparsed.summary().creases, 1);
        assert_eq!(reparsed.summary().facets, 1);
    }

    #[test]
    fn cleanup_after_optimizer_removes_stale_crease_pattern_payload() {
        let mut tree = Tree::from_tmd_str(FIXTURE_CP_V5).unwrap();
        assert_eq!(tree.summary().polys, 1);

        tree.optimize_scale().unwrap();
        let summary = tree.summary();
        assert_eq!(summary.polys, 0);
        assert_eq!(summary.vertices, 0);
        assert_eq!(summary.creases, 0);
        assert_eq!(summary.facets, 0);
        assert!(!tree.is_polygon_valid);
        assert!(!tree.is_polygon_filled);
        assert!(!tree.is_vertex_depth_valid);
        assert!(!tree.is_facet_data_valid);
    }

    #[test]
    fn consumes_and_discards_v4_crease_pattern_payload() {
        let tree = Tree::from_tmd_str(FIXTURE_CP_V4).unwrap();
        let summary = tree.summary();
        assert_eq!(summary.source_version, "4.0");
        assert_eq!(summary.nodes, 2);
        assert_eq!(summary.edges, 1);
        assert_eq!(summary.paths, 1);
        assert_eq!(summary.polys, 0);
        assert_eq!(summary.vertices, 0);
        assert_eq!(summary.creases, 0);
        assert_eq!(tree.nodes[0].owned_vertices.len(), 0);
        assert_eq!(tree.paths[0].fwd_poly, None);
        assert_eq!(tree.paths[0].owned_vertices.len(), 0);
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
