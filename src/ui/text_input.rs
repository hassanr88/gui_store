/*
Made by: Mathew Dusome
Jul 31, 2026
Adds a text input object

================================================================================
CLIPBOARD SUPPORT
================================================================================
No WEB Support
Ctrl+C, Ctrl+V, and Ctrl+X are implemented on windows, linux and macOS using system clipboard utilities
(wl-copy/wl-paste for Wayland, xclip for X11, pbcopy/pbpaste for macOS, and clip/Get-Clipboard for Windows).
Running the following command in terminal:

    cargo add arboard@3.6 --features wayland-data-control --target 'cfg(not(target_arch = "wasm32"))'

Or to manunal add if adding to Cargo.toml:

    [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
    arboard = { version = "3.6", features = ["wayland-data-control"] }


In your src/ui.rs file add the following to the end of the file
        pub mod text_input;

Add the following in the screen you want to use the textinput in with the other use statements
    use crate::ui::text_input::TextInput;

Then to use this you would put the following above the loop:
    let mut txt_input = TextInput::new(100.0, 100.0, 300.0, 40.0, 25.0);
Where the parameters are x, y, width, height, font size


You can customize the text box using various methods:

LIMITS AND MULTILINE:
    // Set a maximum number of characters
    txt_input.set_max_chars(50);

    // Restrict input to specific characters
    txt_input.set_allowed_chars("0123456789");

    // Enable multiline mode (text wraps within the box)
    txt_input.set_multiline(true);


APPEARANCE CUSTOMIZATION:
    // Set colors (text, border, background, cursor)
    txt_input.with_colors(WHITE, BLUE, DARKGRAY, RED);

    // Set individual colors
    txt_input.set_text_color(WHITE)
          .set_border_color(BLUE)
          .set_background_color(DARKGRAY)
          .set_cursor_color(RED);

    // Set custom font
    txt_input.with_font(my_font.clone());

    // Change position and dimensions
    txt_input.set_position(150.0, 150.0);
    txt_input.set_dimensions(250.0, 50.0);

    // Set prompt text and color (shown when input is empty)
    txt_input.set_prompt("Enter your name...");
    txt_input.set_prompt_color(DARKGRAY);

    // Enable or disable the text input
    txt_input.set_enabled(false); // Disable the text input (becomes read-only)
    txt_input.set_enabled(true);  // Enable the text input
    txt_input.set_disabled_color(Color::new(0.7, 0.7, 0.7, 0.5)); // Customize disabled appearance

TEXT MANIPULATION:
    // Get current text
    let current_text = txt_input.get_text();

    // Set text content
    txt_input.set_text("Hello World");

    // Check active state
    if txt_input.is_active() {
        // Do something when textbox is active
    }

    // Set cursor position
    txt_input.set_cursor_index(5);

    // Customize key repeat behavior (for arrow keys, backspace, delete)
    txt_input.set_key_repeat_delay(0.3);    // Initial delay before key repeat starts (seconds)
    txt_input.set_key_repeat_rate(0.03);    // Time between repeats after initial delay (seconds)
    // Or set both at once
    txt_input.with_key_repeat_settings(0.3, 0.03);

Then in the main loop you would use:
    // Update and draw the textbox in one step
    txt_input.draw();
*/

#[cfg(feature = "scale")]
use crate::utils::scale::mouse_position_world as mouse_position;
use macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    pub fn mq_request_paste();
    pub fn mq_get_paste_len() -> usize;
    pub fn mq_fill_paste_buffer(ptr: *mut u8);
    pub fn mq_clear_paste();
    pub fn mq_copy_to_clipboard(ptr: *const u8, len: usize);
}



pub fn copy_to_clipboard(text: String) {
    // =========================
    // 🌐 WEB (using JavaScript clipboard API)
    // =========================
    #[cfg(target_arch = "wasm32")]
    {
        let text_bytes = text.as_bytes();
        unsafe {
            mq_copy_to_clipboard(text_bytes.as_ptr(), text_bytes.len());
        }
        return;
    }
    // =========================
    // 🐧 Everything else
    // =========================
    #[cfg(not(target_arch = "wasm32"))]
    {
        use arboard::Clipboard;

        let clipboard = Clipboard::new();
        if let Ok(mut clipboard) = clipboard {
            clipboard.set_text(text).unwrap();
        }
        return;
    }
}
#[allow(unused)]
pub fn paste_from_clipboard() -> String {
    // =========================
    // 🌐 WEB (using JavaScript clipboard API)
    // =========================
    #[cfg(target_arch = "wasm32")]
    {
        // Request paste from JavaScript (async)
        // JavaScript handles browser-specific logic (disabled for Firefox)
        unsafe {
            mq_request_paste();
        }
        // Return empty - actual content comes via async polling
        return String::new();
    }

    // =========================
    // Everything else
    // =========================
    #[cfg(not(target_arch = "wasm32"))]
    {
        use arboard::Clipboard;

        let clipboard = Clipboard::new();
        if let Ok(mut clipboard) = clipboard {
            return clipboard.get_text().unwrap_or_default();
        }
        return String::new();
    }
}

pub struct TextInput {
    // For vertical navigation, store the preferred column
    preferred_col: Option<usize>,
    #[cfg(target_arch = "wasm32")]
    paste_requested: bool,
    // Make all fields private for complete encapsulation
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    active: bool,
    cursor_index: usize,
    cursor_timer: f32,
    cursor_visible: bool,
    font_size: f32,
    text_color: Color,
    border_color: Color,
    background_color: Color,
    cursor_color: Color,
    font: Option<Font>,
    prompt: Option<String>, // New field for prompt text
    prompt_color: Color,    // Color for the prompt text
    // Add key repeat functionality
    key_repeat_delay: f32,     // Initial delay before key starts repeating (in seconds)
    key_repeat_rate: f32,      // How often the key repeats after initial delay (in seconds)
    key_repeat_timer: f32,     // Timer to track key repeat
    last_key: Option<KeyCode>, // Track the last key that was pressed
    enabled: bool,             // Controls whether the text input can be interacted with
    disabled_color: Color,     // Color used when the text input is disabled
    // New: Multiline and max chars support
    multiline: bool,                 // If true, wraps text to next line within box
    max_chars: Option<usize>,        // Optional maximum number of characters
    allowed_chars: Option<String>,   // Optional whitelist of allowed typed characters
    selection_anchor: Option<usize>, // Selection anchor byte index for range selection
    is_dragging_selection: bool,     // Tracks active mouse drag selection
    drag_start_position: Option<(f32, f32)>, // Track where drag started to distinguish click from drag
}

impl TextInput {
    fn is_char_allowed(&self, c: char) -> bool {
        self.allowed_chars.as_ref().map_or(true, |allowed| allowed.contains(c))
    }

    fn apply_text_constraints(&self, text: &str) -> String {
        let mut constrained = String::new();

        for c in text.chars() {
            if c == '\n' && self.multiline {
                constrained.push(c);
            } else if self.is_char_allowed(c) {
                constrained.push(c);
            }

            if self.max_chars.is_some_and(|max| constrained.chars().count() >= max) {
                break;
            }
        }

        constrained
    }

    fn can_insert_char(&self, c: char) -> bool {
        if c == '\n' && !self.multiline {
            return false;
        }

        if c != '\n' && !self.is_char_allowed(c) {
            return false;
        }

        self.max_chars.map_or(true, |max| self.text.chars().count() < max)
    }

    /// Returns (wrapped_lines, mapping) where mapping[byte_idx] = (line, col)
    fn get_wrapped_lines_and_mapping(&self) -> (Vec<String>, Vec<(usize, usize)>) {
        if !self.multiline {
            // Single line: mapping is just (0, col) for each char
            let mut mapping = Vec::new();
            let mut col = 0;
            for (i, c) in self.text.char_indices() {
                mapping.resize(i + c.len_utf8(), (0, col));
                col += 1;
            }
            mapping.resize(self.text.len() + 1, (0, col));
            return (vec![self.text.clone()], mapping);
        }
        let mut lines = Vec::new();
        let mut mapping = vec![(0, 0); self.text.len() + 1];
        let padding = 5.0;
        let max_width = self.width - 2.0 * padding;
        let font = self.font.as_ref();
        let mut line_idx: usize = 0;
        let mut col_idx = 0;
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut byte_idx = 0;
        let mut chars = self.text.chars().peekable();
        while let Some(c) = chars.next() {
            let c_width = measure_text(&c.to_string(), font, self.font_size as u16, 1.0).width;
            let is_newline = c == '\n';
            // If wrapping needed (but not for newline)
            if !is_newline && current_width + c_width > max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
                line_idx += 1;
                col_idx = 0;
            }
            if is_newline {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
                line_idx += 1;
                col_idx = 0;
                mapping[byte_idx] = (line_idx, col_idx); // Map the newline byte
                byte_idx += c.len_utf8();
                continue;
            }
            current_line.push(c);
            // Map every byte of this char to the current (line, col)
            let char_bytes = c.len_utf8();
            for i in 0..char_bytes {
                if byte_idx + i < mapping.len() {
                    mapping[byte_idx + i] = (line_idx, col_idx);
                }
            }
            byte_idx += char_bytes;
            current_width += c_width;
            col_idx += 1;
        }
        if !current_line.is_empty() {
            lines.push(current_line);
            line_idx += 1;
        }
        // Map the end of the text
        if byte_idx < mapping.len() {
            mapping[byte_idx] = (line_idx.saturating_sub(1), lines.last().map(|l| l.chars().count()).unwrap_or(0));
        }
        (lines, mapping)
    }

    /// Ensure the cursor index and selection anchor are always at a valid UTF-8 boundary and in bounds
    fn ensure_indices_validity(&mut self) {
        // Ensure cursor_index is valid
        if self.cursor_index > self.text.len() {
            self.cursor_index = self.text.len();
        }
        // Clamp to char boundary
        while self.cursor_index > 0 && !self.text.is_char_boundary(self.cursor_index) {
            self.cursor_index -= 1;
        }

        // Ensure selection_anchor is valid if it exists
        if let Some(ref mut anchor) = self.selection_anchor {
            if *anchor > self.text.len() {
                *anchor = self.text.len();
            }
            // Clamp to char boundary
            while *anchor > 0 && !self.text.is_char_boundary(*anchor) {
                *anchor -= 1;
            }
        }
    }

    fn get_selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        let mut start = anchor.min(self.cursor_index);
        let mut end = anchor.max(self.cursor_index);

        while start > 0 && !self.text.is_char_boundary(start) {
            start -= 1;
        }
        while end > 0 && end < self.text.len() && !self.text.is_char_boundary(end) {
            end -= 1;
        }

        if start == end { None } else { Some((start, end)) }
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.get_selection_range() {
            self.text.replace_range(start..end, "");
            self.cursor_index = start;
            self.ensure_indices_validity();
            self.clear_selection();
            return true;
        }
        false
    }

    fn index_from_local_point(&self, local_x: f32, local_y: f32) -> usize {
        if self.text.is_empty() {
            return 0;
        }

        let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
        if wrapped_lines.is_empty() {
            return self.text.len();
        }

        let line_height = self.font_size + 2.0;
        let mut clicked_line = (local_y / line_height).floor() as isize;
        if clicked_line < 0 {
            clicked_line = 0;
        }
        let clicked_line = (clicked_line as usize).min(wrapped_lines.len().saturating_sub(1));

        let mut col = 0usize;
        let mut x_offset = 0.0;
        let font = self.font.as_ref();
        let line = &wrapped_lines[clicked_line];
        for (i, c) in line.chars().enumerate() {
            let c_width = measure_text(&c.to_string(), font, self.font_size as u16, 1.0).width;
            if x_offset + c_width / 2.0 > local_x {
                break;
            }
            x_offset += c_width;
            col = i + 1;
        }

        let mut last_match = None;
        for (byte_idx, &(line_idx, ccol)) in mapping.iter().enumerate() {
            if line_idx == clicked_line && ccol == col {
                last_match = Some(byte_idx);
            }
        }

        last_match.unwrap_or(self.text.len())
    }

    fn horizontal_offset_for_col(&self, line: &str, col: usize) -> f32 {
        let font = self.font.as_ref();
        line.chars()
            .take(col)
            .map(|c| measure_text(&c.to_string(), font, self.font_size as u16, 1.0).width)
            .sum()
    }

    fn line_col_for_index(&self, mapping: &[(usize, usize)], wrapped_lines: &[String], index: usize) -> (usize, usize) {
        let idx = index.min(self.text.len());
        if idx < mapping.len() {
            mapping[idx]
        } else {
            (
                wrapped_lines.len().saturating_sub(1),
                wrapped_lines.last().map(|l| l.chars().count()).unwrap_or(0),
            )
        }
    }

    pub fn new(x: f32, y: f32, width: f32, height: f32, font_size: f32) -> Self {
        Self {
            preferred_col: None,
            #[cfg(target_arch = "wasm32")]
            paste_requested: false,
            x,
            y,
            width,
            height,
            text: String::new(),
            active: false,
            cursor_index: 0,
            cursor_timer: 0.0,
            cursor_visible: true,
            font_size,
            text_color: BLACK,           // Default color for text
            border_color: DARKGRAY,      // Default color for border
            background_color: LIGHTGRAY, // Default color for background
            cursor_color: BLACK,         // Default color for cursor
            font: None,                  // Default to None (use system font)
            prompt: None,                // Default to None (no prompt text)
            prompt_color: GRAY,          // Default color for prompt text
            // Initialize key repeat values
            key_repeat_delay: 0.4, // 400ms initial delay before repeat
            key_repeat_rate: 0.05, // 50ms between repeats after initial delay
            key_repeat_timer: 0.0,
            last_key: None,
            enabled: true,                                  // Default to enabled
            disabled_color: Color::new(0.7, 0.7, 0.7, 0.5), // Semi-transparent gray for disabled state
            multiline: false,
            max_chars: None,
            allowed_chars: None,
            selection_anchor: None,
            is_dragging_selection: false,
            drag_start_position: None,
        }
    }

    /// Enable or disable multiline mode (wrapping within box)
    #[allow(unused)]
    pub fn set_multiline(&mut self, multiline: bool) -> &mut Self {
        self.multiline = multiline;
        self
    }
    #[allow(unused)]
    pub fn is_multiline(&self) -> bool {
        self.multiline
    }

    // Position and dimension getters/setters
    #[allow(unused)]
    pub fn get_x(&self) -> f32 {
        self.x
    }

    #[allow(unused)]
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.x = x;
        self
    }

    #[allow(unused)]
    pub fn get_y(&self) -> f32 {
        self.y
    }

    #[allow(unused)]
    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.y = y;
        self
    }

    #[allow(unused)]
    pub fn get_width(&self) -> f32 {
        self.width
    }

    #[allow(unused)]
    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }

    #[allow(unused)]
    pub fn get_height(&self) -> f32 {
        self.height
    }

    #[allow(unused)]
    pub fn set_height(&mut self, height: f32) -> &mut Self {
        self.height = height;
        self
    }

    // Position convenience methods
    #[allow(unused)]
    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[allow(unused)]
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    // Dimension convenience methods
    #[allow(unused)]
    pub fn get_dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    #[allow(unused)]
    pub fn set_dimensions(&mut self, width: f32, height: f32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    // Add a method to change colors
    #[allow(unused)]
    pub fn with_colors(&mut self, text_color: Color, border_color: Color, background_color: Color, cursor_color: Color) -> &mut Self {
        self.text_color = text_color;
        self.border_color = border_color;
        self.background_color = background_color;
        self.cursor_color = cursor_color;
        self
    }

    // Method to set custom font
    #[allow(unused)]
    pub fn with_font(&mut self, font: Font) -> &mut Self {
        self.font = Some(font);
        self
    }

    // Get the current text content
    #[allow(unused)]
    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    // Set the text content - now accepts both String and &str
    #[allow(unused)]
    pub fn set_text<T: Into<String>>(&mut self, text: T) -> &mut Self {
        self.text = self.apply_text_constraints(&text.into());
        self.ensure_indices_validity();
        self.clear_selection();
        self
    }

    // Active state getters/setters
    #[allow(unused)]
    pub fn is_active(&self) -> bool {
        self.active
    }

    #[allow(unused)]
    pub fn set_active(&mut self, active: bool) -> &mut Self {
        self.active = active;
        if !active {
            self.clear_selection();
            self.is_dragging_selection = false;
        }
        self
    }

    // Cursor index getters/setters
    #[allow(unused)]
    pub fn get_cursor_index(&self) -> usize {
        self.cursor_index
    }

    #[allow(unused)]
    pub fn set_cursor_index(&mut self, index: usize) -> &mut Self {
        let mut clamped = index.min(self.text.len());
        while clamped > 0 && !self.text.is_char_boundary(clamped) {
            clamped -= 1;
        }
        self.cursor_index = clamped;
        self.clear_selection();
        self
    }

    // Font size getters/setters
    #[allow(unused)]
    pub fn get_font_size(&self) -> f32 {
        self.font_size
    }

    #[allow(unused)]
    pub fn set_font_size(&mut self, size: f32) -> &mut Self {
        self.font_size = size;
        self
    }

    // Color getters/setters
    #[allow(unused)]
    pub fn get_text_color(&self) -> Color {
        self.text_color
    }

    #[allow(unused)]
    pub fn set_text_color(&mut self, color: Color) -> &mut Self {
        self.text_color = color;
        self
    }

    #[allow(unused)]
    pub fn get_border_color(&self) -> Color {
        self.border_color
    }

    #[allow(unused)]
    pub fn set_border_color(&mut self, color: Color) -> &mut Self {
        self.border_color = color;
        self
    }

    #[allow(unused)]
    pub fn get_background_color(&self) -> Color {
        self.background_color
    }

    #[allow(unused)]
    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        self.background_color = color;
        self
    }

    #[allow(unused)]
    pub fn get_cursor_color(&self) -> Color {
        self.cursor_color
    }

    #[allow(unused)]
    pub fn set_cursor_color(&mut self, color: Color) -> &mut Self {
        self.cursor_color = color;
        self
    }

    // Font getter/setter
    #[allow(unused)]
    pub fn get_font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    // Prompt text getters/setters
    #[allow(unused)]
    pub fn get_prompt(&self) -> Option<&String> {
        self.prompt.as_ref()
    }

    #[allow(unused)]
    pub fn set_prompt<T: Into<String>>(&mut self, prompt: T) -> &mut Self {
        self.prompt = Some(prompt.into());
        self
    }

    #[allow(unused)]
    pub fn get_prompt_color(&self) -> Color {
        self.prompt_color
    }

    #[allow(unused)]
    pub fn set_prompt_color(&mut self, color: Color) -> &mut Self {
        self.prompt_color = color;
        self
    }

    // Key repeat settings getters/setters
    #[allow(unused)]
    pub fn get_key_repeat_delay(&self) -> f32 {
        self.key_repeat_delay
    }

    #[allow(unused)]
    pub fn set_key_repeat_delay(&mut self, delay: f32) -> &mut Self {
        self.key_repeat_delay = delay;
        self
    }

    #[allow(unused)]
    pub fn get_key_repeat_rate(&self) -> f32 {
        self.key_repeat_rate
    }

    #[allow(unused)]
    pub fn set_key_repeat_rate(&mut self, rate: f32) -> &mut Self {
        self.key_repeat_rate = rate;
        self
    }

    // Convenience method to set both key repeat values at once
    #[allow(unused)]
    pub fn with_key_repeat_settings(&mut self, delay: f32, rate: f32) -> &mut Self {
        self.key_repeat_delay = delay;
        self.key_repeat_rate = rate;
        self
    }

    // Enable/disable functionality
    #[allow(unused)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[allow(unused)]
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        if !enabled {
            self.active = false; // Deactivate if disabled
            self.clear_selection();
            self.is_dragging_selection = false;
        }
        self
    }

    #[allow(unused)]
    pub fn get_disabled_color(&self) -> Color {
        self.disabled_color
    }

    #[allow(unused)]
    pub fn set_disabled_color(&mut self, color: Color) -> &mut Self {
        self.disabled_color = color;
        self
    }

    /// Set the maximum number of characters allowed in the text input.
    #[allow(unused)]
    pub fn set_max_chars(&mut self, max: usize) -> &mut Self {
        self.max_chars = Some(max);
        self
    }

    /// Remove the character limit (unlimited input).
    #[allow(unused)]
    pub fn clear_max_chars(&mut self) -> &mut Self {
        self.max_chars = None;
        self
    }

    /// Restrict text input to only the characters in the provided whitelist.
    #[allow(unused)]
    pub fn set_allowed_chars<T: Into<String>>(&mut self, allowed_chars: T) -> &mut Self {
        self.allowed_chars = Some(allowed_chars.into());
        self.text = self.apply_text_constraints(&self.text);
        self.ensure_indices_validity();
        self
    }

    /// Remove any character whitelist and allow all characters again.
    #[allow(unused)]
    pub fn clear_allowed_chars(&mut self) -> &mut Self {
        self.allowed_chars = None;
        self
    }

    #[allow(unused)]
    pub fn get_allowed_chars(&self) -> Option<&str> {
        self.allowed_chars.as_deref()
    }

    // Primary method - both updates and draws the textbox
    #[allow(unused)]
    pub fn draw(&mut self) {
        self.update_internal();
        self.draw_internal();
    }

    // For cases when only drawing is needed without updating
    #[allow(unused)]
    pub fn draw_only(&self) {
        self.draw_internal();
    }

    // For cases when only updating is needed without drawing
    #[allow(unused)]
    pub fn update_only(&mut self) {
        self.update_internal();
    }

    // Now private - internal implementation only
    fn update_internal(&mut self) {
        // Guard against stale invalid UTF-8 byte indices before any string slicing.
        self.ensure_indices_validity();

        // Skip all interaction if disabled
        if !self.enabled {
            self.active = false;
            self.cursor_visible = false;
            self.clear_selection();
            self.is_dragging_selection = false;
            return;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            self.active = mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

            if self.active {
                let text_x = self.x + 5.0;
                let text_y = self.y + 5.0;
                let new_cursor = self.index_from_local_point(mx - text_x, my - text_y);
                self.cursor_index = new_cursor;
                self.ensure_indices_validity();
                self.clear_selection();
                self.is_dragging_selection = true;
                self.drag_start_position = Some((mx, my));
                self.cursor_visible = true;
                self.cursor_timer = 0.0;
            } else {
                self.clear_selection();
                self.is_dragging_selection = false;
            }
        }

        if self.active && self.is_dragging_selection && is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let text_x = self.x + 5.0;
            let text_y = self.y + 5.0;
            
            // Only start selection if we've actually dragged
            if let Some((start_x, start_y)) = self.drag_start_position {
                let drag_distance = ((mx - start_x).powi(2) + (my - start_y).powi(2)).sqrt();
                if drag_distance > 3.0 && self.selection_anchor.is_none() {
                    self.selection_anchor = Some(self.cursor_index);
                }
            }
            
            self.cursor_index = self.index_from_local_point(mx - text_x, my - text_y);
            self.ensure_indices_validity();
            self.cursor_visible = true;
            self.cursor_timer = 0.0;
        }

        if is_mouse_button_released(MouseButton::Left) {
            self.is_dragging_selection = false;
            self.drag_start_position = None;
        }

        if self.active {
            let shortcut_mod_down = is_key_down(KeyCode::LeftControl)
                || is_key_down(KeyCode::RightControl)
                || is_key_down(KeyCode::LeftSuper)
                || is_key_down(KeyCode::RightSuper)
                || is_key_pressed(KeyCode::LeftControl)
                || is_key_pressed(KeyCode::RightControl)
                || is_key_pressed(KeyCode::LeftSuper)
                || is_key_pressed(KeyCode::RightSuper);
            let mut consumed_shortcut = false;

            if shortcut_mod_down {
                if is_key_pressed(KeyCode::A) {
                    self.selection_anchor = Some(0);
                    self.cursor_index = self.text.len();
                    self.ensure_indices_validity();
                    consumed_shortcut = true;
                } else if is_key_pressed(KeyCode::C) {
                    // Copy to clipboard
                    if let Some((s, e)) = self.get_selection_range() {
                        if s != e {
                            let selected_text = self.text[s..e].to_string();
                            copy_to_clipboard(selected_text);
                        }
                    }
                    consumed_shortcut = true;
                } else if is_key_pressed(KeyCode::X) {
                    // Cut to clipboard
                    if let Some((s, e)) = self.get_selection_range() {
                        if s != e {
                            let selected_text = self.text[s..e].to_string();
                            copy_to_clipboard(selected_text);
                            self.delete_selection();
                        }
                    }
                    consumed_shortcut = true;
                } else if is_key_pressed(KeyCode::V) {
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.paste_requested = true;
                        paste_from_clipboard(); // Request async paste
                        consumed_shortcut = true;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let clipboard_text = paste_from_clipboard();
                        self.delete_selection();
                        for c in clipboard_text.chars() {
                            if self.can_insert_char(c) {
                                self.text.insert(self.cursor_index, c);
                                self.cursor_index += c.len_utf8();
                            }
                        }
                        self.ensure_indices_validity();
                        self.clear_selection();
                        consumed_shortcut = true;
                    }
                }
            }

            if consumed_shortcut {
                self.cursor_visible = true;
                self.cursor_timer = 0.0;
                self.last_key = None;
            }

            // Handle typing
            if shortcut_mod_down {
                // Avoid leaking Ctrl/Cmd shortcut letters into the input buffer.
                while get_char_pressed().is_some() {}
            } else if !consumed_shortcut {
                while let Some(c) = get_char_pressed() {
                    if !c.is_control() {
                        self.delete_selection();
                    }
                    if !c.is_control() && self.can_insert_char(c) {
                        self.text.insert(self.cursor_index, c);
                        self.cursor_index += c.len_utf8();
                        self.ensure_indices_validity();
                        self.clear_selection();
                    }
                }
            }
            // Handle Enter key for multiline
            if self.multiline && is_key_pressed(KeyCode::Enter) {
                self.delete_selection();
            }
            if self.multiline && is_key_pressed(KeyCode::Enter) && self.can_insert_char('\n') {
                self.text.insert(self.cursor_index, '\n');
                self.cursor_index += 1;
                self.ensure_indices_validity();
                self.clear_selection();
            }

            // Initial key presses
            let key_delete_pressed = is_key_pressed(KeyCode::Delete);
            let key_backspace_pressed = is_key_pressed(KeyCode::Backspace);
            let key_left_pressed = is_key_pressed(KeyCode::Left);
            let key_right_pressed = is_key_pressed(KeyCode::Right);
            let key_up_pressed = is_key_pressed(KeyCode::Up);
            let key_down_pressed = is_key_pressed(KeyCode::Down);
            let shift_down = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

            // Handle initial key presses
            if key_delete_pressed {
                if !self.delete_selection() && self.cursor_index < self.text.len() {
                    if let Some((_, c)) = self.text[self.cursor_index..].char_indices().next() {
                        let char_len = c.len_utf8();
                        self.text.replace_range(self.cursor_index..self.cursor_index + char_len, "");
                        self.ensure_indices_validity();
                    }
                }
                self.last_key = Some(KeyCode::Delete);
                self.key_repeat_timer = 0.0;
            } else if key_backspace_pressed {
                if !self.delete_selection() && self.cursor_index > 0 {
                    if let Some((prev_offset, _c)) = self.text[..self.cursor_index].char_indices().rev().next() {
                        self.text.replace_range(prev_offset..self.cursor_index, "");
                        self.cursor_index = prev_offset;
                        self.ensure_indices_validity();
                    }
                }
                self.last_key = Some(KeyCode::Backspace);
                self.key_repeat_timer = 0.0;
            } else if key_left_pressed && self.cursor_index > 0 {
                let mut collapse_only = false;
                if shift_down {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor_index);
                    }
                } else if let Some((start, _end)) = self.get_selection_range() {
                    self.cursor_index = start;
                    self.clear_selection();
                    self.ensure_indices_validity();
                    collapse_only = true;
                    self.last_key = Some(KeyCode::Left);
                    self.key_repeat_timer = 0.0;
                    self.preferred_col = None;
                } else {
                    self.clear_selection();
                }
                if !collapse_only {
                    let prev_char = self.text[..self.cursor_index].chars().last().unwrap();
                    let char_len = prev_char.len_utf8();
                    self.cursor_index -= char_len;
                    self.ensure_indices_validity();
                    self.last_key = Some(KeyCode::Left);
                    self.key_repeat_timer = 0.0;
                    self.preferred_col = None;
                }
            } else if key_right_pressed && self.cursor_index < self.text.len() {
                let mut collapse_only = false;
                if shift_down {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor_index);
                    }
                } else if let Some((_start, end)) = self.get_selection_range() {
                    self.cursor_index = end;
                    self.clear_selection();
                    self.ensure_indices_validity();
                    collapse_only = true;
                    self.last_key = Some(KeyCode::Right);
                    self.key_repeat_timer = 0.0;
                    self.preferred_col = None;
                } else {
                    self.clear_selection();
                }
                if !collapse_only {
                    let next_char = self.text[self.cursor_index..].chars().next().unwrap();
                    let char_len = next_char.len_utf8();
                    self.cursor_index += char_len;
                    self.ensure_indices_validity();
                    self.last_key = Some(KeyCode::Right);
                    self.key_repeat_timer = 0.0;
                    self.preferred_col = None;
                }
            } else if self.multiline && (key_up_pressed || key_down_pressed) {
                // Robust multiline up/down navigation using mapping
                let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
                let cursor_idx = self.cursor_index.min(self.text.len());
                let (cur_line, cur_col) = if cursor_idx < mapping.len() {
                    mapping[cursor_idx]
                } else {
                    (
                        wrapped_lines.len().saturating_sub(1),
                        wrapped_lines.last().map(|l| l.chars().count()).unwrap_or(0),
                    )
                };
                if shift_down {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor_index);
                    }
                } else {
                    self.clear_selection();
                }
                // Store preferred_col for vertical navigation
                if self.preferred_col.is_none() {
                    self.preferred_col = Some(cur_col);
                }
                let mut new_line = cur_line;
                if key_up_pressed && cur_line > 0 {
                    new_line = cur_line - 1;
                } else if key_down_pressed && cur_line + 1 < wrapped_lines.len() {
                    new_line = cur_line + 1;
                }
                if new_line != cur_line {
                    let preferred_col = self.preferred_col.unwrap_or(cur_col);
                    let new_line_len = wrapped_lines[new_line].chars().count();
                    let new_col = preferred_col.min(new_line_len);
                    // Find the last byte index in mapping that matches (new_line, new_col)
                    let mut last_match = None;
                    for (byte_idx, &(line, col)) in mapping.iter().enumerate() {
                        if line == new_line && col == new_col {
                            last_match = Some(byte_idx);
                        }
                    }
                    if let Some(byte_idx) = last_match {
                        self.cursor_index = byte_idx;
                        self.ensure_indices_validity();
                    } else {
                        self.cursor_index = self.text.len();
                        self.ensure_indices_validity();
                    }
                }
                // Reset preferred_col if left/right or typing
                if key_left_pressed || key_right_pressed {
                    self.preferred_col = None;
                }
            }

            // Handle key repeat functionality
            if let Some(key) = self.last_key {
                if is_key_down(key) {
                    self.key_repeat_timer += get_frame_time();
                    if self.key_repeat_timer >= self.key_repeat_delay {
                        self.key_repeat_timer -= self.key_repeat_rate;
                        match key {
                            KeyCode::Left => {
                                if self.cursor_index > 0 {
                                    let shift_repeat_down = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                                    if shift_repeat_down {
                                        if self.selection_anchor.is_none() {
                                            self.selection_anchor = Some(self.cursor_index);
                                        }
                                    } else {
                                        self.clear_selection();
                                    }
                                    let prev_char = self.text[..self.cursor_index].chars().last().unwrap();
                                    let char_len = prev_char.len_utf8();
                                    self.cursor_index -= char_len;
                                    self.ensure_indices_validity();
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_index < self.text.len() {
                                    let shift_repeat_down = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                                    if shift_repeat_down {
                                        if self.selection_anchor.is_none() {
                                            self.selection_anchor = Some(self.cursor_index);
                                        }
                                    } else {
                                        self.clear_selection();
                                    }
                                    let next_char = self.text[self.cursor_index..].chars().next().unwrap();
                                    let char_len = next_char.len_utf8();
                                    self.cursor_index += char_len;
                                    self.ensure_indices_validity();
                                }
                            }
                            KeyCode::Delete => {
                                if !self.delete_selection() && self.cursor_index < self.text.len() {
                                    if let Some((_, c)) = self.text[self.cursor_index..].char_indices().next() {
                                        let char_len = c.len_utf8();
                                        self.text.replace_range(self.cursor_index..self.cursor_index + char_len, "");
                                        self.ensure_indices_validity();
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                if !self.delete_selection() && self.cursor_index > 0 {
                                    if let Some((prev_offset, _c)) = self.text[..self.cursor_index].char_indices().rev().next() {
                                        self.text.replace_range(prev_offset..self.cursor_index, "");
                                        self.cursor_index = prev_offset;
                                        self.ensure_indices_validity();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    self.last_key = None;
                    self.key_repeat_timer = 0.0;
                }
            }

            self.cursor_timer += get_frame_time();
            if self.cursor_timer >= 0.5 {
                self.cursor_visible = !self.cursor_visible;
                self.cursor_timer = 0.0;
            }
        } else {
            self.cursor_visible = false;
        }
        self.paste_wasm32();
    }

    // WASM async paste polling
    fn paste_wasm32(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if self.paste_requested {
                unsafe {
                    let len = mq_get_paste_len();
                    if len > 0 {
                        let mut buffer = vec![0u8; len];
                        mq_fill_paste_buffer(buffer.as_mut_ptr());
                        mq_clear_paste();
                        let text = String::from_utf8_lossy(&buffer).to_string();
                        self.delete_selection();
                        for c in text.chars() {
                            if self.can_insert_char(c) {
                                self.text.insert(self.cursor_index, c);
                                self.cursor_index += c.len_utf8();
                            }
                        }
                        self.ensure_indices_validity();
                        self.clear_selection();
                        self.paste_requested = false;
                    }
                }
            }
        }
    }

    // Now private - internal implementation only
    fn draw_internal(&self) {
        let padding = 5.0;
        let text_x = self.x + padding;
        let text_y = self.y + self.font_size + padding;
        let line_height = self.font_size + 2.0;

        // Draw the background with customizable colors (or disabled color when disabled)
        if self.enabled {
            draw_rectangle(self.x, self.y, self.width, self.height, self.background_color);
        } else {
            draw_rectangle(self.x, self.y, self.width, self.height, self.disabled_color);
        }

        let text_color = if self.enabled { self.text_color } else { GRAY };
        let prompt_color = if self.enabled { self.prompt_color } else { GRAY };

        // Draw text selection highlight before text so glyphs remain readable.
        if self.enabled && self.active {
            if let Some((sel_start, sel_end)) = self.get_selection_range() {
                let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
                if !wrapped_lines.is_empty() {
                    let (start_line, start_col) = self.line_col_for_index(&mapping, &wrapped_lines, sel_start);
                    let (end_line, end_col) = self.line_col_for_index(&mapping, &wrapped_lines, sel_end);
                    let select_color = Color::new(0.2, 0.45, 0.95, 0.35);

                    for line_idx in start_line..=end_line {
                        if line_idx >= wrapped_lines.len() {
                            break;
                        }
                        let line = &wrapped_lines[line_idx];
                        let line_len = line.chars().count();
                        let from_col = if line_idx == start_line { start_col.min(line_len) } else { 0 };
                        let to_col = if line_idx == end_line { end_col.min(line_len) } else { line_len };

                        if to_col > from_col {
                            let start_x = text_x + self.horizontal_offset_for_col(line, from_col);
                            let end_x = text_x + self.horizontal_offset_for_col(line, to_col);
                            let y = text_y + line_idx as f32 * line_height - self.font_size * 0.8;
                            draw_rectangle(start_x, y, end_x - start_x, self.font_size + 6.0, select_color);
                        }
                    }
                }
            }
        }

        // Draw text (with wrapping if multiline)
        if self.text.is_empty() {
            if let Some(prompt) = &self.prompt {
                match &self.font {
                    Some(font) => {
                        draw_text_ex(
                            prompt,
                            text_x,
                            text_y,
                            TextParams {
                                font: Some(font),
                                font_size: self.font_size as u16,
                                color: prompt_color,
                                ..Default::default()
                            },
                        );
                    }
                    None => {
                        draw_text(prompt, text_x, text_y, self.font_size, prompt_color);
                    }
                }
            }
        } else {
            let (wrapped_lines, _mapping) = self.get_wrapped_lines_and_mapping();
            for (i, line) in wrapped_lines.iter().enumerate() {
                let y = text_y + i as f32 * (self.font_size + 2.0);
                match &self.font {
                    Some(font) => {
                        draw_text_ex(
                            line,
                            text_x,
                            y,
                            TextParams {
                                font: Some(font),
                                font_size: self.font_size as u16,
                                color: text_color,
                                ..Default::default()
                            },
                        );
                    }
                    None => {
                        draw_text(line, text_x, y, self.font_size, text_color);
                    }
                }
            }
        }

        // Cursor rendering for multiline (basic support)
        if self.enabled && self.active && self.cursor_visible {
            if self.multiline {
                // Use the same mapping as navigation for accurate cursor placement
                let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
                let cursor_idx = self.cursor_index.min(self.text.len());
                let (cursor_line, cursor_col) = self.line_col_for_index(&mapping, &wrapped_lines, cursor_idx);
                let mut cursor_offset = 0.0;
                if cursor_line < wrapped_lines.len() {
                    let line = &wrapped_lines[cursor_line];
                    cursor_offset = self.horizontal_offset_for_col(line, cursor_col);
                }
                let cursor_spacing = 2.0;
                let y = text_y + cursor_line as f32 * line_height;
                draw_line(
                    text_x + cursor_offset + cursor_spacing,
                    y - self.font_size * 0.7,
                    text_x + cursor_offset + cursor_spacing,
                    y + 2.0,
                    1.0,
                    self.cursor_color,
                );
            } else {
                let mut cursor_offset = 0.0;
                let mut safe_cursor_index = self.cursor_index.min(self.text.len());
                while safe_cursor_index > 0 && !self.text.is_char_boundary(safe_cursor_index) {
                    safe_cursor_index -= 1;
                }
                if safe_cursor_index > 0 {
                    let cursor_text = &self.text[..safe_cursor_index];
                    if let Some(font) = &self.font {
                        for c in cursor_text.chars() {
                            cursor_offset += measure_text(&c.to_string(), Some(font), self.font_size as u16, 1.0).width;
                        }
                    } else {
                        for c in cursor_text.chars() {
                            cursor_offset += measure_text(&c.to_string(), None, self.font_size as u16, 1.0).width;
                        }
                    }
                }
                let cursor_spacing = 2.0;
                draw_line(
                    text_x + cursor_offset + cursor_spacing,
                    text_y - self.font_size * 0.7,
                    text_x + cursor_offset + cursor_spacing,
                    text_y + 2.0,
                    1.0,
                    self.cursor_color,
                );
            }
        }

        let border_color = if self.enabled { self.border_color } else { GRAY };
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 2.0, border_color);
    }
}
