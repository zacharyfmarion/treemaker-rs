use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{App, Emitter};

pub fn setup_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let app_about = PredefinedMenuItem::about(app, None, None)?;
    let app_quit = MenuItemBuilder::with_id("app.quit", "Quit TreeMaker")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;
    let app_menu = SubmenuBuilder::new(app, "TreeMaker")
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
    let file_export_svg = MenuItemBuilder::with_id("file.exportSvg", "Export SVG...").build(app)?;
    let file_export_png = MenuItemBuilder::with_id("file.exportPng", "Export PNG...").build(app)?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&file_new)
        .item(&file_open)
        .separator()
        .item(&file_save)
        .item(&file_save_as)
        .separator()
        .item(&file_export_v4)
        .item(&file_export_svg)
        .item(&file_export_png)
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
    let edit_delete = MenuItemBuilder::with_id("edit.delete", "Delete Selected")
        .accelerator("Backspace")
        .build(app)?;

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
        .item(&edit_delete)
        .build()?;

    let view_design = MenuItemBuilder::with_id("view.design", "Design").build(app)?;
    let view_cp = MenuItemBuilder::with_id("view.creasePattern", "Crease Pattern").build(app)?;
    let view_reset = MenuItemBuilder::with_id("view.resetLayout", "Reset Layout").build(app)?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&view_design)
        .item(&view_cp)
        .separator()
        .item(&view_reset)
        .build()?;

    let optimize_scale = MenuItemBuilder::with_id("optimize.scale", "Optimize Scale")
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    let build_cp = MenuItemBuilder::with_id("cp.build", "Build Crease Pattern")
        .accelerator("CmdOrCtrl+B")
        .build(app)?;

    let design_menu = SubmenuBuilder::new(app, "Design")
        .item(&optimize_scale)
        .item(&build_cp)
        .build()?;

    let help_about = MenuItemBuilder::with_id("help.about", "About TreeMaker").build(app)?;
    let help_menu = SubmenuBuilder::new(app, "Help").item(&help_about).build()?;

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
