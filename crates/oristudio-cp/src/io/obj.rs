use super::{IoError, Result};
use crate::geometry::{Line, LineColor, LineSegment, Point};
use crate::model::CreasePatternModel;

/// Import the subset of OBJ Oriedita reads: vertices, faces, and `#e` color
/// constraint comments.
pub fn import_obj_str(input: &str) -> Result<CreasePatternModel> {
    let mut points = vec![Point::origin()];
    let mut lines = vec![Line::default()];

    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "v" => {
                if tokens.len() < 4 {
                    return invalid_obj(line_number, "vertex needs x y z");
                }
                points.push(Point::new(
                    parse_f64(tokens[1], line_number)?,
                    parse_f64(tokens[2], line_number)?,
                ));
            }
            "f" => {
                if tokens.len() < 4 {
                    return invalid_obj(line_number, "face needs at least three vertices");
                }
                let mut face = Vec::with_capacity(tokens.len());
                face.push(0);
                for token in &tokens[1..] {
                    face.push(parse_obj_index(token, line_number)?);
                }
                let last = face[face.len() - 1];
                face[0] = last;
                for index in 0..face.len() - 1 {
                    let begin = face[index];
                    let end = face[index + 1];
                    if !lines.iter().any(|line| {
                        (line.begin == begin && line.end == end)
                            || (line.begin == end && line.end == begin)
                    }) {
                        lines.push(Line::new(begin, end, LineColor::Black0));
                    }
                }
            }
            "#e" => {
                if tokens.len() < 5 {
                    return invalid_obj(line_number, "#e needs begin end color id");
                }
                let begin = parse_i32(tokens[1], line_number)?;
                let end = parse_i32(tokens[2], line_number)?;
                let color = tokens[3].parse::<LineColor>()?;
                for line in &mut lines {
                    if (line.begin == begin && line.end == end)
                        || (line.begin == end && line.end == begin)
                    {
                        *line = line.with_color(color);
                    }
                }
            }
            _ => {}
        }
    }

    let mut model = CreasePatternModel::default();
    for line in lines {
        let Some(begin) = usize::try_from(line.begin)
            .ok()
            .and_then(|index| points.get(index))
        else {
            continue;
        };
        let Some(end) = usize::try_from(line.end)
            .ok()
            .and_then(|index| points.get(index))
        else {
            continue;
        };
        let color = obj_postprocess_color(line.color)?;
        model.add_line_segment(LineSegment::with_color(*begin, *end, color));
    }

    Ok(model)
}

fn obj_postprocess_color(color: LineColor) -> Result<LineColor> {
    let mut imported = LineColor::from_number(color.number() - 1)?;
    if imported == LineColor::Red1 {
        imported = LineColor::Blue2;
    }
    if imported == LineColor::Blue2 {
        imported = LineColor::Red1;
    }
    Ok(imported)
}

fn invalid_obj<T>(line: usize, message: impl Into<String>) -> Result<T> {
    Err(IoError::InvalidLine {
        format: "obj",
        line,
        message: message.into(),
    })
}

fn parse_obj_index(token: &str, line: usize) -> Result<i32> {
    parse_i32(token.split('/').next().unwrap_or(token), line)
}

fn parse_i32(token: &str, line: usize) -> Result<i32> {
    token.parse::<i32>().map_err(|error| IoError::InvalidLine {
        format: "obj",
        line,
        message: error.to_string(),
    })
}

fn parse_f64(token: &str, line: usize) -> Result<f64> {
    token.parse::<f64>().map_err(|error| IoError::InvalidLine {
        format: "obj",
        line,
        message: error.to_string(),
    })
}
