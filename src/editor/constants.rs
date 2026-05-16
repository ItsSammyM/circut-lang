use egui::Color32;

// ─────────────────────────────────────────────────────────────────────────────
//  Layout / geometry constants
// ─────────────────────────────────────────────────────────────────────────────

/// Width of a gate node box on the canvas (canvas-space units).
pub const NODE_WIDTH: f32 = 108.0;

/// Vertical distance from the top of a node body to its first port dot.
pub const PORT_TOP_PADDING: f32 = 28.0;

/// Vertical distance between consecutive port dots on the same node.
pub const PORT_VERTICAL_STEP: f32 = 22.0;

/// Radius of port dots drawn on gate nodes.
pub const PORT_RADIUS: f32 = 6.0;

/// Size of one background grid cell in canvas-space units.
pub const GRID_CELL_SIZE: f32 = 20.0;

/// Vertical distance between consecutive I/O rail port dots.
pub const IO_RAIL_STEP: f32 = 52.0;

// ─────────────────────────────────────────────────────────────────────────────
//  Color palette
// ─────────────────────────────────────────────────────────────────────────────

pub const COLOR_BACKGROUND: Color32        = Color32::from_rgb(22, 24, 34);
pub const COLOR_GRID: Color32              = Color32::from_rgb(38, 42, 58);
pub const COLOR_PANEL_BG: Color32          = Color32::from_rgb(28, 30, 42);
pub const COLOR_NODE_FILL: Color32         = Color32::from_rgb(48, 52, 76);
pub const COLOR_NODE_HOVERED: Color32      = Color32::from_rgb(68, 74, 110);
pub const COLOR_NODE_STROKE: Color32       = Color32::from_rgb(90, 100, 150);
pub const COLOR_PORT_INPUT: Color32        = Color32::from_rgb(80, 175, 235);
pub const COLOR_PORT_OUTPUT: Color32       = Color32::from_rgb(235, 175, 60);
pub const COLOR_WIRE: Color32              = Color32::from_rgb(80, 100, 110);
pub const COLOR_WIRE_HIGH: Color32         = Color32::from_rgb(100, 230, 120);
pub const COLOR_WIRE_LOW: Color32          = Color32::from_rgb(60, 80, 100);
pub const COLOR_WIRE_PENDING: Color32      = Color32::from_rgb(240, 220, 60);
pub const COLOR_SIGNAL_HIGH: Color32       = Color32::from_rgb(70, 230, 90);
pub const COLOR_SIGNAL_LOW: Color32        = Color32::from_rgb(55, 60, 85);
pub const COLOR_TEXT: Color32              = Color32::from_rgb(210, 218, 255);
pub const COLOR_DIM: Color32               = Color32::from_rgb(120, 130, 170);
pub const COLOR_BOX_SELECT: Color32        = Color32::from_rgba_premultiplied(80, 160, 255, 30);
pub const COLOR_BOX_SELECT_BORDER: Color32 = Color32::from_rgb(80, 160, 255);

// ─────────────────────────────────────────────────────────────────────────────
//  Font sizes (Typographic scale)
// ─────────────────────────────────────────────────────────────────────────────

/// Font size for small dim text (e.g., panel hints, tips).
pub const FONT_SIZE_SMALL: f32 = 10.0;

/// Font size for standard body text (e.g., buttons, subheaders, context menu items).
pub const FONT_SIZE_BODY: f32 = 11.0;

/// Font size for subheadings (e.g., status text, item headers).
pub const FONT_SIZE_SUBHEAD: f32 = 12.0;

/// Font size for headings (e.g., section headers, context menu titles).
pub const FONT_SIZE_HEADING: f32 = 13.0;

/// Font size for icons (e.g., drag handles).
pub const FONT_SIZE_ICON: f32 = 14.0;

/// Font size for large icons (e.g., top bar app icon).
pub const FONT_SIZE_LARGE: f32 = 22.0;

// Canvas-specific font sizes (zoom-scaled, so they must be separate)

/// Font size for I/O rail port labels on the canvas.
pub const FONT_SIZE_IO_RAIL_LABEL: f32 = 12.0;

/// Font size for gate node titles on the canvas.
pub const FONT_SIZE_NODE_LABEL: f32 = 11.0;

/// Font size for gate node port labels on the canvas.
pub const FONT_SIZE_NODE_PORT_LABEL: f32 = 9.0;