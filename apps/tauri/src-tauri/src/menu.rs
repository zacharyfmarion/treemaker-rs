use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{App, Emitter};

pub fn setup_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let app_about = MenuItemBuilder::with_id("app.about", "About Ori Studio").build(app)?;
    let app_quit = MenuItemBuilder::with_id("app.quit", "Quit Ori Studio")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;
    let app_menu = SubmenuBuilder::new(app, "Ori Studio")
        .item(&app_about)
        .separator()
        .item(&app_quit)
        .build()?;

    let file_new = MenuItemBuilder::with_id("file.new", "New")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    let file_open = MenuItemBuilder::with_id("file.open", "Open...")
        .accelerator("CmdOrCtrl+O")
        .build(app)?;
    let file_save = MenuItemBuilder::with_id("file.save", "Save")
        .accelerator("CmdOrCtrl+S")
        .build(app)?;
    let file_save_as = MenuItemBuilder::with_id("file.saveAs", "Save As...")
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?;
    let file_export_v4 =
        MenuItemBuilder::with_id("file.exportV4", "Export TreeMaker 4...").build(app)?;
    let file_export_fold =
        MenuItemBuilder::with_id("file.exportFold", "Export FOLD...").build(app)?;
    let file_export_svg = MenuItemBuilder::with_id("file.exportSvg", "Export SVG...").build(app)?;
    let file_export_png = MenuItemBuilder::with_id("file.exportPng", "Export PNG...").build(app)?;
    let file_settings = MenuItemBuilder::with_id("file.settings", "Settings")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&file_new)
        .item(&file_open)
        .separator()
        .item(&file_save)
        .item(&file_save_as)
        .separator()
        .item(&file_export_v4)
        .item(&file_export_fold)
        .item(&file_export_svg)
        .item(&file_export_png)
        .separator()
        .item(&file_settings)
        .build()?;

    let edit_undo = MenuItemBuilder::with_id("edit.undo", "Undo")
        .accelerator("CmdOrCtrl+Z")
        .build(app)?;
    let edit_redo = MenuItemBuilder::with_id("edit.redo", "Redo")
        .accelerator("CmdOrCtrl+Shift+Z")
        .build(app)?;
    let edit_cut = MenuItemBuilder::with_id("edit.cut", "Cut")
        .accelerator("CmdOrCtrl+X")
        .build(app)?;
    let edit_copy = MenuItemBuilder::with_id("edit.copy", "Copy")
        .accelerator("CmdOrCtrl+C")
        .build(app)?;
    let edit_paste = MenuItemBuilder::with_id("edit.paste", "Paste")
        .accelerator("CmdOrCtrl+V")
        .build(app)?;
    let edit_select_all = MenuItemBuilder::with_id("edit.selectAll", "Select All")
        .accelerator("CmdOrCtrl+A")
        .build(app)?;
    let edit_deselect = MenuItemBuilder::with_id("edit.deselectAll", "Deselect All").build(app)?;
    let edit_select_by_index =
        MenuItemBuilder::with_id("edit.selectByIndex", "Select By Index...").build(app)?;
    let edit_select_movable =
        MenuItemBuilder::with_id("edit.selectMovableParts", "Select Movable Parts").build(app)?;
    let edit_select_corridor =
        MenuItemBuilder::with_id("edit.selectCorridorFacets", "Select Corridor Facets")
            .build(app)?;
    let edit_delete = MenuItemBuilder::with_id("edit.delete", "Delete Selected")
        .accelerator("Backspace")
        .build(app)?;
    let edit_make_root = MenuItemBuilder::with_id("edit.makeRoot", "Make Root").build(app)?;
    let edit_split_edge = MenuItemBuilder::with_id("edit.splitEdge", "Split Edge...").build(app)?;
    let edit_set_edge_length =
        MenuItemBuilder::with_id("edit.setEdgeLength", "Set Edge Length...").build(app)?;
    let edit_scale_edge_lengths =
        MenuItemBuilder::with_id("edit.scaleEdgeLengths", "Scale Edge Lengths...").build(app)?;
    let edit_renormalize_edge =
        MenuItemBuilder::with_id("edit.renormalizeToEdge", "Renormalize To Edge").build(app)?;
    let edit_renormalize_scale =
        MenuItemBuilder::with_id("edit.renormalizeToUnitScale", "Renormalize To Unit Scale")
            .build(app)?;
    let edit_absorb_nodes =
        MenuItemBuilder::with_id("edit.absorbNodes", "Absorb Nodes").build(app)?;
    let edit_absorb_redundant =
        MenuItemBuilder::with_id("edit.absorbRedundantNodes", "Absorb Redundant Nodes")
            .build(app)?;
    let edit_absorb_edges =
        MenuItemBuilder::with_id("edit.absorbEdges", "Absorb Edges").build(app)?;
    let edit_perturb_nodes =
        MenuItemBuilder::with_id("edit.perturbNodes", "Perturb Nodes").build(app)?;
    let edit_perturb_all =
        MenuItemBuilder::with_id("edit.perturbAllNodes", "Perturb All Nodes").build(app)?;
    let edit_remove_strain =
        MenuItemBuilder::with_id("edit.removeStrain", "Remove Strain").build(app)?;
    let edit_remove_all_strain =
        MenuItemBuilder::with_id("edit.removeAllStrain", "Remove All Strain").build(app)?;
    let edit_relieve_strain =
        MenuItemBuilder::with_id("edit.relieveStrain", "Relieve Strain").build(app)?;
    let edit_relieve_all_strain =
        MenuItemBuilder::with_id("edit.relieveAllStrain", "Relieve All Strain").build(app)?;
    let edit_stub_nodes =
        MenuItemBuilder::with_id("edit.addLargestStubForNodes", "Add Largest Stub From Nodes")
            .build(app)?;
    let edit_stub_poly =
        MenuItemBuilder::with_id("edit.addLargestStubForPoly", "Add Largest Stub From Poly")
            .build(app)?;
    let edit_triangulate =
        MenuItemBuilder::with_id("edit.triangulateTree", "Triangulate Tree").build(app)?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&edit_undo)
        .item(&edit_redo)
        .separator()
        .item(&edit_cut)
        .item(&edit_copy)
        .item(&edit_paste)
        .separator()
        .item(&edit_select_all)
        .item(&edit_deselect)
        .item(&edit_select_by_index)
        .item(&edit_select_movable)
        .item(&edit_select_corridor)
        .item(&edit_delete)
        .separator()
        .item(&edit_make_root)
        .item(&edit_split_edge)
        .item(&edit_set_edge_length)
        .item(&edit_scale_edge_lengths)
        .item(&edit_renormalize_edge)
        .item(&edit_renormalize_scale)
        .item(&edit_absorb_nodes)
        .item(&edit_absorb_redundant)
        .item(&edit_absorb_edges)
        .item(&edit_perturb_nodes)
        .item(&edit_perturb_all)
        .item(&edit_remove_strain)
        .item(&edit_remove_all_strain)
        .item(&edit_relieve_strain)
        .item(&edit_relieve_all_strain)
        .item(&edit_stub_nodes)
        .item(&edit_stub_poly)
        .item(&edit_triangulate)
        .build()?;

    let view_design = MenuItemBuilder::with_id("view.design", "Design").build(app)?;
    let view_cp = MenuItemBuilder::with_id("view.creasePattern", "Crease Pattern").build(app)?;
    let view_simulator = MenuItemBuilder::with_id("view.simulator", "Simulator").build(app)?;
    let view_folded_base = MenuItemBuilder::with_id("view.foldedBase", "Folded Base").build(app)?;
    let view_conditions = MenuItemBuilder::with_id("view.conditions", "Conditions").build(app)?;
    let view_reset = MenuItemBuilder::with_id("view.resetLayout", "Reset Layout").build(app)?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&view_design)
        .item(&view_cp)
        .item(&view_simulator)
        .item(&view_folded_base)
        .item(&view_conditions)
        .separator()
        .item(&view_reset)
        .build()?;

    let optimize_scale = MenuItemBuilder::with_id("optimize.scale", "Optimize Scale")
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    let optimize_edges = MenuItemBuilder::with_id("optimize.edges", "Optimize Edges").build(app)?;
    let optimize_strain =
        MenuItemBuilder::with_id("optimize.strain", "Optimize Strain").build(app)?;
    let build_cp = MenuItemBuilder::with_id("cp.build", "Build Crease Pattern")
        .accelerator("CmdOrCtrl+B")
        .build(app)?;

    let design_menu = SubmenuBuilder::new(app, "Design")
        .item(&optimize_scale)
        .item(&optimize_edges)
        .item(&optimize_strain)
        .item(&build_cp)
        .build()?;

    let help_docs = MenuItemBuilder::with_id("help.documentation", "Ori Studio Help")
        .accelerator("F1")
        .build(app)?;
    let help_about = MenuItemBuilder::with_id("help.about", "About Ori Studio").build(app)?;
    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&help_docs)
        .separator()
        .item(&help_about)
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&design_menu)
        .item(&help_menu)
        .build()?;

    app.set_menu(menu)?;

    app.on_menu_event(move |app_handle, event| {
        let id = event.id().0.as_str();
        if let Err(error) = app_handle.emit("menu-action", id) {
            eprintln!("Failed to emit menu-action: {error}");
        }
    });

    Ok(())
}
