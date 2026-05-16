use std::collections::{HashMap, VecDeque};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use treemaker_core::{
    Condition, ConditionKind, Crease, Edge, Facet, Node, OptimizationKind, OwnerRef, Path, Point,
    Poly, TmFloat, Tree, Vertex,
};

const PANEL_W: TmFloat = 760.0;
const PANEL_H: TmFloat = 660.0;
const GAP: TmFloat = 54.0;
const MARGIN: TmFloat = 54.0;
const SVG_W: TmFloat = PANEL_W * 2.0 + GAP + 2.0 * MARGIN;
const SVG_H: TmFloat = PANEL_H + 2.0 * MARGIN + 58.0;

#[derive(Clone)]
struct NodeSpec {
    label: &'static str,
    loc: Point,
}

#[derive(Clone)]
struct EdgeSpec {
    a: usize,
    b: usize,
    length: TmFloat,
}

#[derive(Clone)]
struct ExampleSpec {
    slug: &'static str,
    title: &'static str,
    subtitle: &'static str,
    has_symmetry: bool,
    symmetry_angle: TmFloat,
    nodes: Vec<NodeSpec>,
    edges: Vec<EdgeSpec>,
    conditions: Vec<ConditionKind>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = output_dir();
    fs::create_dir_all(&out_dir)?;

    for spec in examples() {
        let mut tree = build_tree(&spec);
        let before = tree.to_tmd5_string();
        fs::write(out_dir.join(format!("{}-input.tmd5", spec.slug)), before)?;

        let report = tree.optimize_scale()?;
        assert_eq!(report.kind, OptimizationKind::Scale);
        fs::write(
            out_dir.join(format!("{}-optimized.tmd5", spec.slug)),
            tree.to_tmd5_string(),
        )?;
        tree.build_polys_and_crease_pattern()?;

        let svg = render_example(&spec, &tree);
        fs::write(out_dir.join(format!("{}.svg", spec.slug)), svg)?;
        fs::write(
            out_dir.join(format!("{}-built.tmd5", spec.slug)),
            tree.to_tmd5_string(),
        )?;

        let summary = tree.summary();
        println!(
            "{}: scale={:.6}, cp_status={:?}, vertices={}, creases={}, facets={}",
            spec.slug,
            tree.scale,
            summary.cp_status,
            summary.vertices,
            summary.creases,
            summary.facets
        );
    }

    Ok(())
}

fn output_dir() -> PathBuf {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--out"
            && let Some(path) = args.next()
        {
            return PathBuf::from(path);
        }
    }
    PathBuf::from("artifacts/visualizations")
}

fn examples() -> Vec<ExampleSpec> {
    vec![
        ExampleSpec {
            slug: "triad",
            title: "Three terminal flaps",
            subtitle: "simple 3-leaf tree after scale optimization",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.50),
                ns("t0", 0.14, 0.16),
                ns("t1", 0.86, 0.17),
                ns("t2", 0.50, 0.88),
            ],
            edges: vec![es(1, 2, 1.0), es(1, 3, 1.0), es(1, 4, 1.0)],
            conditions: Vec::new(),
        },
        ExampleSpec {
            slug: "mirrored-fork",
            title: "Mirrored four-flap fork",
            subtitle: "paired terminal nodes constrained across the vertical symmetry axis",
            has_symmetry: true,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.50, 0.46),
                ns("t0", 0.16, 0.18),
                ns("t1", 0.84, 0.18),
                ns("t2", 0.18, 0.82),
                ns("t3", 0.82, 0.82),
            ],
            edges: vec![
                es(1, 2, 0.92),
                es(1, 3, 0.92),
                es(1, 4, 0.82),
                es(1, 5, 0.82),
            ],
            conditions: vec![
                ConditionKind::NodesPaired { node1: 2, node2: 3 },
                ConditionKind::NodesPaired { node1: 4, node2: 5 },
            ],
        },
        ExampleSpec {
            slug: "asymmetric-antler",
            title: "Asymmetric antler tree",
            subtitle: "six uneven terminal flaps on a branched trunk",
            has_symmetry: false,
            symmetry_angle: 90.0,
            nodes: vec![
                ns("root", 0.48, 0.18),
                ns("h0", 0.48, 0.40),
                ns("h1", 0.40, 0.64),
                ns("h2", 0.63, 0.58),
                ns("t0", 0.12, 0.16),
                ns("t1", 0.86, 0.22),
                ns("t2", 0.15, 0.68),
                ns("t3", 0.44, 0.92),
                ns("t4", 0.88, 0.78),
                ns("t5", 0.90, 0.50),
            ],
            edges: vec![
                es(1, 2, 0.36),
                es(2, 3, 0.34),
                es(2, 4, 0.32),
                es(1, 5, 0.50),
                es(1, 6, 0.58),
                es(3, 7, 0.42),
                es(3, 8, 0.38),
                es(4, 9, 0.48),
                es(4, 10, 0.34),
            ],
            conditions: Vec::new(),
        },
    ]
}

fn ns(label: &'static str, x: TmFloat, y: TmFloat) -> NodeSpec {
    NodeSpec {
        label,
        loc: Point { x, y },
    }
}

fn es(a: usize, b: usize, length: TmFloat) -> EdgeSpec {
    EdgeSpec { a, b, length }
}

fn build_tree(spec: &ExampleSpec) -> Tree {
    let n = spec.nodes.len();
    let mut adjacency = vec![Vec::<(usize, usize)>::new(); n + 1];
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (i, node) in spec.nodes.iter().enumerate() {
        nodes.push(Node {
            index: i + 1,
            label: node.label.to_string(),
            loc: node.loc,
            depth: -999.0,
            elevation: 0.0,
            is_leaf: false,
            is_sub: false,
            is_border: false,
            is_pinned: false,
            is_polygon: false,
            is_junction: false,
            is_conditioned: false,
            owned_vertices: Vec::new(),
            edges: Vec::new(),
            leaf_paths: Vec::new(),
            owner: OwnerRef::Tree,
        });
    }

    for (i, edge) in spec.edges.iter().enumerate() {
        let edge_id = i + 1;
        edges.push(Edge {
            index: edge_id,
            label: format!("e{edge_id}"),
            length: edge.length,
            strain: 0.0,
            stiffness: 1.0,
            is_pinned: false,
            is_conditioned: false,
            nodes: vec![edge.a, edge.b],
        });
        adjacency[edge.a].push((edge.b, edge_id));
        adjacency[edge.b].push((edge.a, edge_id));
        nodes[edge.a - 1].edges.push(edge_id);
        nodes[edge.b - 1].edges.push(edge_id);
    }

    let leaf = |node_id: usize, adjacency: &[Vec<(usize, usize)>]| adjacency[node_id].len() == 1;
    for node_id in 1..=n {
        nodes[node_id - 1].is_leaf = leaf(node_id, &adjacency);
    }

    let mut paths = Vec::new();
    for a in 1..=n {
        for b in a + 1..=n {
            let (path_nodes, path_edges) = tree_path(a, b, &adjacency);
            let path_id = paths.len() + 1;
            let min_tree_length = path_edges
                .iter()
                .map(|edge_id| edges[*edge_id - 1].length)
                .sum::<TmFloat>();
            let is_leaf = leaf(a, &adjacency) && leaf(b, &adjacency);
            if is_leaf {
                nodes[a - 1].leaf_paths.push(path_id);
                nodes[b - 1].leaf_paths.push(path_id);
            }
            paths.push(Path {
                index: path_id,
                min_tree_length,
                min_paper_length: min_tree_length * 0.1,
                act_tree_length: 0.0,
                act_paper_length: 0.0,
                is_leaf,
                is_sub: false,
                is_feasible: false,
                is_active: false,
                is_border: false,
                is_polygon: false,
                is_conditioned: false,
                fwd_poly: None,
                bkd_poly: None,
                nodes: path_nodes,
                edges: path_edges,
                outset_path: None,
                front_reduction: 0.0,
                back_reduction: 0.0,
                min_depth: -999.0,
                min_depth_dist: -999.0,
                owned_vertices: Vec::new(),
                owned_creases: Vec::new(),
                owner: OwnerRef::Tree,
            });
        }
    }

    let conditions = spec
        .conditions
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, kind)| Condition {
            index: i + 1,
            is_feasible: true,
            kind,
        })
        .collect();

    Tree {
        source_version: "5.0".to_string(),
        paper_width: 1.0,
        paper_height: 1.0,
        scale: 0.1,
        has_symmetry: spec.has_symmetry,
        sym_loc: Point { x: 0.5, y: 0.0 },
        sym_angle: spec.symmetry_angle,
        is_feasible: false,
        is_polygon_valid: false,
        is_polygon_filled: false,
        is_vertex_depth_valid: false,
        is_facet_data_valid: false,
        is_local_root_connectable: false,
        needs_cleanup: true,
        nodes,
        edges,
        paths,
        polys: Vec::<Poly>::new(),
        vertices: Vec::<Vertex>::new(),
        creases: Vec::<Crease>::new(),
        facets: Vec::<Facet>::new(),
        conditions,
        owned_nodes: (1..=n).collect(),
        owned_edges: (1..=spec.edges.len()).collect(),
        owned_paths: (1..=paths_count(n)).collect(),
        owned_polys: Vec::new(),
    }
}

fn paths_count(n: usize) -> usize {
    n * (n - 1) / 2
}

fn tree_path(
    start: usize,
    end: usize,
    adjacency: &[Vec<(usize, usize)>],
) -> (Vec<usize>, Vec<usize>) {
    let mut parent: HashMap<usize, (usize, usize)> = HashMap::new();
    let mut queue = VecDeque::from([start]);
    parent.insert(start, (0, 0));
    while let Some(node) = queue.pop_front() {
        if node == end {
            break;
        }
        for (next, edge_id) in adjacency[node].iter().copied() {
            if parent.contains_key(&next) {
                continue;
            }
            parent.insert(next, (node, edge_id));
            queue.push_back(next);
        }
    }

    let mut nodes = vec![end];
    let mut edges = Vec::new();
    let mut cur = end;
    while cur != start {
        let (prev, edge_id) = parent[&cur];
        edges.push(edge_id);
        nodes.push(prev);
        cur = prev;
    }
    nodes.reverse();
    edges.reverse();
    (nodes, edges)
}

fn render_example(spec: &ExampleSpec, tree: &Tree) -> String {
    let mut svg = String::new();
    let left = MARGIN;
    let right = MARGIN + PANEL_W + GAP;
    let top = MARGIN + 28.0;
    let summary = tree.summary();

    write!(
        svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{SVG_W}" height="{SVG_H}" viewBox="0 0 {SVG_W} {SVG_H}">
<rect width="100%" height="100%" fill="#f8fafc"/>
<style>
text {{ font-family: Arial, Helvetica, sans-serif; fill: #334155; }}
.small {{ font-size: 13px; }}
.tiny {{ font-size: 11px; }}
.title {{ font-size: 24px; font-weight: 700; fill: #0f172a; }}
.subtitle {{ font-size: 14px; fill: #64748b; }}
.panel-title {{ font-size: 15px; font-weight: 700; fill: #334155; }}
</style>
"##
    )
    .unwrap();

    text(&mut svg, left, 30.0, spec.title, "title");
    text(
        &mut svg,
        left,
        52.0,
        &format!(
            "{} | scale {:.4} | {:?} | V/C/F {}/{}/{}",
            spec.subtitle,
            tree.scale,
            summary.cp_status,
            summary.vertices,
            summary.creases,
            summary.facets
        ),
        "subtitle",
    );

    panel(&mut svg, left, top, PANEL_W, PANEL_H, "input tree");
    panel(
        &mut svg,
        right,
        top,
        PANEL_W,
        PANEL_H,
        "generated crease pattern",
    );
    draw_grid(&mut svg, left, top, PANEL_W, PANEL_H);
    draw_grid(&mut svg, right, top, PANEL_W, PANEL_H);
    draw_tree(&mut svg, tree, left, top, PANEL_W, PANEL_H);
    draw_crease_pattern(&mut svg, tree, right, top, PANEL_W, PANEL_H);
    draw_tree_legend(&mut svg, left + 20.0, top + PANEL_H - 74.0);
    draw_cp_legend(&mut svg, right + 20.0, top + PANEL_H - 94.0);

    svg.push_str("</svg>\n");
    svg
}

fn panel(svg: &mut String, x: TmFloat, y: TmFloat, w: TmFloat, h: TmFloat, label: &str) {
    writeln!(
        svg,
        r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="14" fill="#ffffff" stroke="#cbd5e1" stroke-width="2"/>"##
    )
    .unwrap();
    text(svg, x + 18.0, y - 10.0, label, "panel-title");
}

fn draw_grid(svg: &mut String, x: TmFloat, y: TmFloat, w: TmFloat, h: TmFloat) {
    let p = plot_rect(x, y, w, h);
    writeln!(
        svg,
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#f8fafc" stroke="#dbe4ef" stroke-width="1"/>"##,
        p.x, p.y, p.w, p.h
    )
    .unwrap();
    for i in 1..10 {
        let gx = p.x + p.w * i as TmFloat / 10.0;
        let gy = p.y + p.h * i as TmFloat / 10.0;
        line(svg, (gx, p.y), (gx, p.y + p.h), "#dbe4ef", 1.0, 1.0);
        line(svg, (p.x, gy), (p.x + p.w, gy), "#dbe4ef", 1.0, 1.0);
    }
}

fn draw_tree(svg: &mut String, tree: &Tree, x: TmFloat, y: TmFloat, w: TmFloat, h: TmFloat) {
    let p = plot_rect(x, y, w, h);
    if tree.has_symmetry {
        let sx = map_x(p, tree.sym_loc.x);
        line(svg, (sx, p.y), (sx, p.y + p.h), "#94a3b8", 1.6, 1.0);
        text(svg, sx + 8.0, p.y + 20.0, "vertical mirror axis", "small");
    }
    for edge_id in tree.owned_edges.iter().copied() {
        let edge = &tree.edges[edge_id - 1];
        let a = edge.nodes[0];
        let b = edge.nodes[1];
        let pa = tree.nodes[a - 1].loc;
        let pb = tree.nodes[b - 1].loc;
        let internal = !tree.nodes[a - 1].is_leaf && !tree.nodes[b - 1].is_leaf;
        let color = if internal { "#ef4444" } else { "#334155" };
        line(
            svg,
            (map_x(p, pa.x), map_y(p, pa.y)),
            (map_x(p, pb.x), map_y(p, pb.y)),
            color,
            4.0,
            1.0,
        );
        let mx = (map_x(p, pa.x) + map_x(p, pb.x)) / 2.0;
        let my = (map_y(p, pa.y) + map_y(p, pb.y)) / 2.0;
        edge_label(svg, mx, my, &format!("L {:.2}", edge.length));
    }

    for node_id in tree.owned_nodes.iter().copied() {
        let node = &tree.nodes[node_id - 1];
        let (fill, radius) = if node_id == 1 {
            ("#f59e0b", 6.5)
        } else if node.is_leaf {
            ("#2563eb", 5.2)
        } else {
            ("#ef4444", 6.0)
        };
        circle(
            svg,
            map_x(p, node.loc.x),
            map_y(p, node.loc.y),
            radius,
            fill,
            "#ffffff",
        );
        text(
            svg,
            map_x(p, node.loc.x) + 8.0,
            map_y(p, node.loc.y) + 4.0,
            &node.label,
            "small",
        );
    }
}

fn draw_crease_pattern(
    svg: &mut String,
    tree: &Tree,
    x: TmFloat,
    y: TmFloat,
    w: TmFloat,
    h: TmFloat,
) {
    let p = plot_rect(x, y, w, h);
    for facet in &tree.facets {
        if facet.vertices.len() < 3 {
            continue;
        }
        let fill = match facet.color {
            1 => "#dbeafe",
            2 => "#fee2e2",
            _ => "#f1f5f9",
        };
        let mut pts = String::new();
        for vertex_id in &facet.vertices {
            let v = &tree.vertices[*vertex_id - 1];
            write!(pts, "{:.3},{:.3} ", map_x(p, v.loc.x), map_y(p, v.loc.y)).unwrap();
        }
        writeln!(
            svg,
            r##"<polygon points="{}" fill="{fill}" fill-opacity="0.22" stroke="none"/>"##,
            pts.trim()
        )
        .unwrap();
    }

    for crease in &tree.creases {
        if crease.vertices.len() < 2 {
            continue;
        }
        let a = &tree.vertices[crease.vertices[0] - 1];
        let b = &tree.vertices[crease.vertices[1] - 1];
        let (color, width, opacity) = crease_style(crease.fold, crease.kind);
        line(
            svg,
            (map_x(p, a.loc.x), map_y(p, a.loc.y)),
            (map_x(p, b.loc.x), map_y(p, b.loc.y)),
            color,
            width,
            opacity,
        );
    }

    writeln!(
        svg,
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#111827" stroke-width="4"/>"##,
        p.x, p.y, p.w, p.h
    )
    .unwrap();

    for vertex in &tree.vertices {
        circle(
            svg,
            map_x(p, vertex.loc.x),
            map_y(p, vertex.loc.y),
            2.1,
            "#0f172a",
            "none",
        );
    }
}

fn crease_style(fold: i32, kind: i32) -> (&'static str, TmFloat, TmFloat) {
    match fold {
        1 => ("#ef4444", 3.0, 1.0),
        2 => ("#0b63ff", 3.0, 1.0),
        3 => ("#111827", 3.2, 1.0),
        _ => match kind {
            0..=2 => ("#94a3b8", 1.4, 0.75),
            _ => ("#64748b", 1.0, 0.55),
        },
    }
}

fn draw_tree_legend(svg: &mut String, x: TmFloat, y: TmFloat) {
    legend_dot(svg, x, y, "#f59e0b", "root/body node");
    legend_dot(svg, x, y + 18.0, "#ef4444", "branch hub/path");
    legend_dot(svg, x, y + 36.0, "#2563eb", "terminal flap node");
}

fn draw_cp_legend(svg: &mut String, x: TmFloat, y: TmFloat) {
    legend_line(svg, x, y, "#ef4444", "M mountain");
    legend_line(svg, x, y + 20.0, "#0b63ff", "V valley");
    legend_line(svg, x, y + 40.0, "#111827", "B border");
    legend_line(svg, x, y + 60.0, "#94a3b8", "F/U flat or construction");
}

fn legend_dot(svg: &mut String, x: TmFloat, y: TmFloat, color: &str, label: &str) {
    circle(svg, x, y - 4.0, 5.0, color, "#ffffff");
    text(svg, x + 12.0, y, label, "small");
}

fn legend_line(svg: &mut String, x: TmFloat, y: TmFloat, color: &str, label: &str) {
    line(svg, (x, y - 4.0), (x + 34.0, y - 4.0), color, 4.0, 1.0);
    text(svg, x + 42.0, y, label, "small");
}

#[derive(Clone, Copy)]
struct PlotRect {
    x: TmFloat,
    y: TmFloat,
    w: TmFloat,
    h: TmFloat,
}

fn plot_rect(x: TmFloat, y: TmFloat, w: TmFloat, h: TmFloat) -> PlotRect {
    let pad = 58.0;
    let size = (w - 2.0 * pad).min(h - 2.0 * pad - 10.0);
    PlotRect {
        x: x + (w - size) / 2.0,
        y: y + pad,
        w: size,
        h: size,
    }
}

fn map_x(p: PlotRect, x: TmFloat) -> TmFloat {
    p.x + x * p.w
}

fn map_y(p: PlotRect, y: TmFloat) -> TmFloat {
    p.y + (1.0 - y) * p.h
}

fn line(
    svg: &mut String,
    start: (TmFloat, TmFloat),
    end: (TmFloat, TmFloat),
    color: &str,
    width: TmFloat,
    opacity: TmFloat,
) {
    let (x1, y1) = start;
    let (x2, y2) = end;
    writeln!(
        svg,
        r##"<line x1="{x1:.3}" y1="{y1:.3}" x2="{x2:.3}" y2="{y2:.3}" stroke="{color}" stroke-width="{width}" stroke-linecap="round" opacity="{opacity}"/>"##
    )
    .unwrap();
}

fn circle(svg: &mut String, x: TmFloat, y: TmFloat, r: TmFloat, fill: &str, stroke: &str) {
    writeln!(
        svg,
        r##"<circle cx="{x:.3}" cy="{y:.3}" r="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1.2"/>"##
    )
    .unwrap();
}

fn text(svg: &mut String, x: TmFloat, y: TmFloat, value: &str, class_name: &str) {
    writeln!(
        svg,
        r##"<text x="{x:.3}" y="{y:.3}" class="{class_name}">{}</text>"##,
        escape(value)
    )
    .unwrap();
}

fn edge_label(svg: &mut String, x: TmFloat, y: TmFloat, value: &str) {
    let width = 42.0 + value.len() as TmFloat * 3.2;
    writeln!(
        svg,
        r##"<rect x="{:.3}" y="{:.3}" width="{width:.3}" height="20" rx="6" fill="#ffffff" stroke="#cbd5e1"/>"##,
        x - width / 2.0,
        y - 14.0
    )
    .unwrap();
    text(svg, x - width / 2.0 + 7.0, y + 1.0, value, "tiny");
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
