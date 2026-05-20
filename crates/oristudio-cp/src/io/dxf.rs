use crate::geometry::LineColor;
use crate::model::CreasePatternModel;

/// Export Oriedita-style AutoCAD R12 DXF line entities.
pub fn export_dxf_string(model: &CreasePatternModel) -> String {
    let mut output = String::new();
    push_line(&mut output, "  0");
    push_line(&mut output, "SECTION");
    push_line(&mut output, "  2");
    push_line(&mut output, "HEADER");
    push_line(&mut output, "  9");
    push_line(&mut output, "$ACADVER");
    push_line(&mut output, "  1");
    push_line(&mut output, "AC1009");
    push_line(&mut output, "  0");
    push_line(&mut output, "ENDSEC");
    push_line(&mut output, "  0");
    push_line(&mut output, "SECTION");
    push_line(&mut output, "  2");
    push_line(&mut output, "ENTITIES");

    for segment in &model.line_segments {
        let (layer, color_number) = dxf_layer(segment.color);
        push_line(&mut output, "  0");
        push_line(&mut output, "LINE");
        push_line(&mut output, "  8");
        push_line(&mut output, layer);
        push_line(&mut output, "  6");
        push_line(&mut output, "CONTINUOUS");
        push_line(&mut output, "  62");
        push_line(&mut output, &color_number.to_string());
        push_line(&mut output, "  10");
        push_line(
            &mut output,
            &scale(segment.a.x + 200.0, 3.0, 4.0).to_string(),
        );
        push_line(&mut output, "  20");
        push_line(
            &mut output,
            &scale(segment.a.y - 200.0, -3.0, 4.0).to_string(),
        );
        push_line(&mut output, "  11");
        push_line(
            &mut output,
            &scale(segment.b.x + 200.0, 3.0, 4.0).to_string(),
        );
        push_line(&mut output, "  21");
        push_line(
            &mut output,
            &scale(segment.b.y - 200.0, -3.0, 4.0).to_string(),
        );
    }

    push_line(&mut output, "  0");
    push_line(&mut output, "ENDSEC");
    push_line(&mut output, "  0");
    push_line(&mut output, "EOF");
    output
}

fn dxf_layer(line_color: LineColor) -> (&'static str, i32) {
    match line_color {
        LineColor::Black0 => ("CutLine", 250),
        LineColor::Red1 => ("MountainLine", 1),
        LineColor::Blue2 => ("ValleyLine", 5),
        LineColor::Cyan3 => ("AuxiliaryLine", 4),
        _ => ("noname", 0),
    }
}

fn scale(value: f64, scale: f64, center: f64) -> f64 {
    value * scale + center
}

fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
