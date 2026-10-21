use taffy::prelude::*;

#[derive(Clone)]
enum NodeContext {
    Text(TextContext),
}

#[derive(Clone)]
struct TextContext {
    text_content: String,
    writing_mode: WritingMode,
}

#[derive(Clone)]
enum WritingMode {
    Horizontal,
}

#[derive(Clone)]
struct FontMetrics {
    char_width: f32,
    char_height: f32,
}

fn text_measure_function(
    known_dimensions: taffy::geometry::Size<Option<f32>>,
    available_space: taffy::geometry::Size<taffy::style::AvailableSpace>,
    text_context: &TextContext,
    font_metrics: &FontMetrics,
) -> taffy::geometry::Size<f32> {
    use taffy::geometry::AbsoluteAxis;
    use taffy::prelude::*;

    let inline_axis = AbsoluteAxis::Horizontal;
    let block_axis = inline_axis.other_axis();
    let words: Vec<&str> = text_context.text_content.split_whitespace().collect();

    if words.is_empty() {
        return Size::ZERO;
    }

    let min_line_length: usize = words.iter().map(|line| line.len()).max().unwrap_or(0);
    let max_line_length: usize = words.iter().map(|line| line.len()).sum();
    let inline_size = known_dimensions.get_abs(inline_axis).unwrap_or_else(|| match available_space.get_abs(inline_axis) {
        AvailableSpace::MinContent => min_line_length as f32 * font_metrics.char_width,
        AvailableSpace::MaxContent => max_line_length as f32 * font_metrics.char_width,
        AvailableSpace::Definite(inline_size) => inline_size.min(max_line_length as f32 * font_metrics.char_width).max(min_line_length as f32 * font_metrics.char_width),
    });
    let block_size = known_dimensions.get_abs(block_axis).unwrap_or_else(|| {
        let inline_line_length = (inline_size / font_metrics.char_width).floor() as usize;
        let mut line_count = 1;
        let mut current_line_length = 0;
        for word in &words {
            if current_line_length == 0 {
                current_line_length = word.len();
            } else if current_line_length + word.len() + 1 > inline_line_length {
                line_count += 1;
                current_line_length = word.len();
            } else {
                current_line_length += word.len() + 1;
            };
        }
        (line_count as f32) * font_metrics.char_height
    });

    match text_context.writing_mode {
        WritingMode::Horizontal => Size { width: inline_size, height: block_size },
    }
}

fn measure_function(
    known_dimensions: taffy::geometry::Size<Option<f32>>,
    available_space: taffy::geometry::Size<taffy::style::AvailableSpace>,
    node_context: Option<&mut NodeContext>,
    font_metrics: &FontMetrics,
) -> Size<f32> {
    if let Size { width: Some(width), height: Some(height) } = known_dimensions {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(NodeContext::Text(text_context)) => text_measure_function(known_dimensions, available_space, text_context, font_metrics),
    }
}

struct Cell {
    ch: char,
}

const COLS: usize = 100;
const ROWS: usize = 38;
const FONT_W: f32 = 8.0;
const FONT_H: f32 = 16.0;

fn fill_rect(buf: &mut [Vec<Cell>], x: f32, y: f32, w: f32, h: f32, bg: char) {
    let c0 = (x / FONT_W) as usize;
    let r0 = (y / FONT_H) as usize;
    let c1 = ((x + w) / FONT_W) as usize;
    let r1 = ((y + h) / FONT_H) as usize;
    for r in r0..r1 {
        for c in c0..c1 {
            buf[r][c].ch = bg;
        }
    }
}

fn draw_text(buf: &mut [Vec<Cell>], x: f32, y: f32, text: &str, max_chars: usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_len = 0usize;
    for w in words {
        let wl = w.len();
        if cur.is_empty() {
            cur = w.to_string();
            cur_len = wl;
        } else if cur_len + 1 + wl <= max_chars {
            cur.push(' ');
            cur.push_str(w);
            cur_len += 1 + wl;
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = w.to_string();
            cur_len = wl;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    let mut row = (y / FONT_H) as usize;
    let col = (x / FONT_W) as usize;
    for line in lines {
        let mut c = col;
        for ch in line.chars() {
            buf[row][c].ch = ch;
            c += 1;
        }
        row += (FONT_H / 16.0) as usize;
    }
}

fn print_buf(buf: &[Vec<Cell>]) {
    for row in buf.iter() {
        let mut line = String::new();
        for cell in row.iter() {
            line.push(cell.ch);
        }
        println!("{}", line);
    }
}

fn main() -> Result<(), taffy::TaffyError> {
    let mut taffy: TaffyTree<NodeContext> = TaffyTree::new();

    let font_metrics = FontMetrics { char_width: 8.0, char_height: 16.0 };

    let header = taffy.new_leaf_with_context(
        Style {
            display: Display::Grid,
            size: Size { width: length(800.0), height: length(150.0) },
            grid_row: line(1),
            grid_column: span(3),
            ..Default::default()
        },
        NodeContext::Text(TextContext { text_content: "THE GREAT PUMPKIN".into(), writing_mode: WritingMode::Horizontal }),
    )?;

    let roasted = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "Roasted".into(), writing_mode: WritingMode::Horizontal }),
    )?;
    let spice = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "Pumpkin Spice".into(), writing_mode: WritingMode::Horizontal }),
    )?;
    let carving = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "Carving".into(), writing_mode: WritingMode::Horizontal }),
    )?;
    let seeds = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "Seeds".into(), writing_mode: WritingMode::Horizontal }),
    );
    let nav = taffy.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size { width: length(250.0), height: auto() },
            ..Default::default()
        },
        &[roasted, spice, carving, seeds?],
    )?;

    let headline = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "The Great Pumpkin".into(), writing_mode: WritingMode::Horizontal }),
    )?;
    let paragraph = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "The biggest pumpkin in the patch. The most orange. The most round. The most legendary. One patch, one vote, one champion.".into(), writing_mode: WritingMode::Horizontal }),
    );
    let content = taffy.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size { width: length(250.0), height: auto() },
            ..Default::default()
        },
        &[headline, paragraph?],
    )?;

    let fact1 = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "1 patch = 1 vote".into(), writing_mode: WritingMode::Horizontal }),
    )?;
    let fact2 = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "Orange is the new orange".into(), writing_mode: WritingMode::Horizontal }),
    )?;
    let fact3 = taffy.new_leaf_with_context(
        Style { display: Display::Block, ..Default::default() },
        NodeContext::Text(TextContext { text_content: "Size matters".into(), writing_mode: WritingMode::Horizontal }),
    );
    let aside = taffy.new_with_children(
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size { width: length(250.0), height: auto() },
            ..Default::default()
        },
        &[fact1, fact2, fact3?],
    )?;

    let footer = taffy.new_leaf_with_context(
        Style {
            display: Display::Grid,
            size: Size { width: length(800.0), height: length(80.0) },
            grid_row: line(3),
            grid_column: span(3),
            ..Default::default()
        },
        NodeContext::Text(TextContext { text_content: "Copyright Pumpkin Inc. All rights reserved.".into(), writing_mode: WritingMode::Horizontal }),
    )?;

    let root = taffy.new_with_children(
        Style {
            display: Display::Grid,
            size: Size { width: length(800.0), height: length(600.0) },
            grid_template_columns: vec![length(250.0), fr(1.0), length(250.0)],
            grid_template_rows: vec![length(150.0), fr(1.0), length(80.0)],
            ..Default::default()
        },
        &[header, nav, content, aside, footer],
    )?;

    taffy.compute_layout_with_measure(
        root,
        Size { width: length(800.0), height: length(600.0) },
        |inputs, _node_id, node_context, style| {
            taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, |known_dimensions, available_space| {
                measure_function(known_dimensions, available_space, node_context, &font_metrics)
            })
        },
    )?;

    let header_l = taffy.layout(header)?;
    let nav_l = taffy.layout(nav)?;
    let content_l = taffy.layout(content)?;
    let aside_l = taffy.layout(aside)?;
    let footer_l = taffy.layout(footer)?;

    let header_rect = (header_l.location.x, header_l.location.y, header_l.size.width, header_l.size.height);
    let nav_rect = (nav_l.location.x, nav_l.location.y, nav_l.size.width, nav_l.size.height);
    let content_rect = (content_l.location.x, content_l.location.y, content_l.size.width, content_l.size.height);
    let aside_rect = (aside_l.location.x, aside_l.location.y, aside_l.size.width, aside_l.size.height);
    let footer_rect = (footer_l.location.x, footer_l.location.y, footer_l.size.width, footer_l.size.height);

    let (hx, hy, hw, hh) = header_rect;
    let (fx, fy, fw, fh) = footer_rect;
    let (sx, sy, sw, sh) = nav_rect;
    let (cx, cy, cw, ch) = content_rect;
    let (ax, ay, aw, ah) = aside_rect;

    let mut buf: Vec<Vec<Cell>> = (0..ROWS).map(|_| (0..COLS).map(|_| Cell { ch: ' ' }).collect()).collect();

    fill_rect(&mut buf, hx, hy, hw, hh, '#');
    fill_rect(&mut buf, fx, fy, fw, fh, '#');
    fill_rect(&mut buf, sx, sy, sw, sh, '-');
    fill_rect(&mut buf, cx, cy, cw, ch, '.');
    fill_rect(&mut buf, ax, ay, aw, ah, '-');

    draw_text(&mut buf, hx + (hw - 128.0) / 2.0, hy + (hh - 16.0) / 2.0, "THE GREAT PUMPKIN", 20);
    draw_text(&mut buf, sx + 20.0, sy + 20.0, "Roasted", 20);
    draw_text(&mut buf, sx + 20.0, sy + 40.0, "Pumpkin Spice", 20);
    draw_text(&mut buf, sx + 20.0, sy + 60.0, "Carving", 20);
    draw_text(&mut buf, sx + 20.0, sy + 80.0, "Seeds", 20);
    draw_text(&mut buf, cx + (cw - 128.0) / 2.0, cy + 24.0, "The Great Pumpkin", 20);
    draw_text(&mut buf, cx + 16.0, cy + 60.0, "The biggest pumpkin in the patch. The most orange. The most round. The most legendary. One patch, one vote, one champion.", ((cw - 32.0) / FONT_W) as usize);
    draw_text(&mut buf, ax + 20.0, ay + 20.0, "1 patch = 1 vote", 20);
    draw_text(&mut buf, ax + 20.0, ay + 40.0, "Orange is the new orange", 20);
    draw_text(&mut buf, ax + 20.0, ay + 60.0, "Size matters", 20);
    draw_text(&mut buf, fx + (fw - 360.0) / 2.0, fy + 20.0, "Copyright Pumpkin Inc. All rights reserved.", 45);

    print_buf(&buf);

    Ok(())
}
