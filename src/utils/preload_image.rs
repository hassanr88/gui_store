/*
Made by: Mathew Dusome
Date: 2026-06-13
Program Details: Central texture manager for preloading and sharing textures with loading screen support

To use this:
1. In your utils.rs file add the following to the end of the file:
    pub mod preload_image;
In Cargo.toml add the following under dependencies:
    image = "0.25.10"

2. Add the following use commands:
    use crate::utils::preload_image::TextureManager;
    use crate::utils::preload_image::LoadingScreenOptions; // If you want to customize the loading screen
    use crate::utils::preload_image::GifLoadingScreenInfo; // If you want to add animated GIFs to loading screen

3. Create and initialize a TextureManager:
    let tm = TextureManager::new();

4. Preload your textures at startup - multiple approaches:

   // Option 1: Basic preloading without a loading screen
   // Preload a list of textures
   tm.preload_all(&["assets/image1.png", "assets/image2.png"]).await;

   // Or preload individual textures
   tm.preload("assets/image3.png").await;

   // Option 2: Preload with a built-in loading screen (best for web)
   // Using default loading screen appearance
  tm.preload_with_loading_screen(&["assets/image1.png","assets/image2.png"], None, None).await;

   // Using custom loading screen appearance
   let loading_options = LoadingScreenOptions {
       title: Some("MY GAME".to_string()),
       background_color: DARKBLUE,
       bar_fill_color: GOLD,
       // Use default values for other options
       ..Default::default()
   };
   tm.preload_with_loading_screen(&["assets/image1.png"], None, None).await;

   // Option 3: Preload with loading screen and load both PNGs and GIFs
   // The assets list can include both .png and .gif files - they'll be handled automatically
   let all_files = vec![
       "assets/image1.png",
       "assets/image2.png",
       "assets/animation.gif",  // This will be preloaded as an animated GIF
   ];
   tm.preload_with_loading_screen(&all_files, None).await;

   // Option 4: Preload with animated GIFs displayed on the loading screen
   let loading_options = LoadingScreenOptions {
       title: Some("MY GAME".to_string()),
       background_color: DARKBLUE,
       bar_fill_color: GOLD,
       loading_screen_gifs: vec![
           GifLoadingScreenInfo::new(
               "assets/loading_animation.gif".to_string(),
               screen_width() / 2.0 - 128.0,  // center x (assuming 256px width)
               100.0,                          // y position
               256.0,                          // width
               256.0,                          // height
           ),
       ],
       ..Default::default()
   };
   tm.preload_with_loading_screen(&all_assets, Some(loading_options)).await;

5. Get preloaded textures for use with StillImage - two approaches:

   // Approach 1: Using unwrap() - Simple but will panic if image doesn't exist
   // Only use this when you're certain the texture was preloaded
   img.set_preload(tm.get_preload("assets/image1.png").unwrap());

   // Approach 2: Using if let Some() - Safer, handles missing textures gracefully
   if let Some(preloaded) = tm.get_preload("assets/image2.png") {
       img.set_preload(preloaded);
   } else {
       println!("Warning: Image not found in texture manager");
       // Handle the error case (e.g., try to load it or use a placeholder)
   }

6. Access textures by index:
    // Using unwrap() approach:
    img.set_preload(tm.get_preload_by_index(0).unwrap());

    // Using if let Some() approach:
    if let Some(preloaded) = tm.get_preload_by_index(1) {
        img.set_preload(preloaded);
    }

7. Getting the number of preloaded textures:
    let count = tm.texture_count();

8. Customizing the loading screen appearance:
   // LoadingScreenOptions provides many customization options:
   let custom_options = LoadingScreenOptions {
       // Game title (optional)
       title: Some("YOUR GAME TITLE".to_string()),

       // Colors
       background_color: DARKGREEN,        // Background of the entire screen
       bar_background_color: DARKGRAY,     // Background of the progress bar
       bar_fill_color: GREEN,              // Fill color of the progress bar
       text_color: WHITE,                  // Color for title and progress text
       filename_color: SKYBLUE,            // Color for the filename text

       // Font sizes
       title_font_size: 60,                // Size of the title text
       progress_font_size: 30,             // Size of the progress percentage text
       filename_font_size: 20,             // Size of the filename text

       // Completion behavior
       show_completion_message: true,                    // Whether to show completion message
       completion_message: "Loading Complete!".to_string(), // Custom completion message
       completion_delay: 0.5,                            // Delay in seconds after completion

       // Animated GIFs
       loading_screen_gifs: vec![
           // Add as many GIFs as you want!
           GifLoadingScreenInfo::new(
               "assets/spinner.gif".to_string(),
               screen_width() / 2.0 - 64.0,   // Centered horizontally
               screen_height() / 3.0,          // Upper third of screen
               128.0,                          // width
               128.0,                          // height
           ),
       ],
   };

Note: This TextureManager implementation is thread-safe and web-compatible. The loading screen
uses coroutines to load assets in the background, avoiding black flashing on web platforms.
GIFs are pre-preloaded before the loading screen displays, ensuring smooth animation during loading.
*/
use macroquad::audio::{Sound, load_sound};
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::*;
use macroquad::texture::Texture2D;
use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;
use image::ImageDecoder;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
#[allow(unused)]
pub struct PreloadedAnimatedSpritesheet {
    pub texture: Texture2D,
    pub transparency_mask: Option<Vec<u8>>,
    pub path: String,
    pub cols: usize,
    pub rows: usize,
}

#[derive(Clone)]
#[allow(unused)]
pub struct PreloadedAnimatedGif {
    pub texture: Texture2D,
    pub transparency_mask: Option<Vec<u8>>,
    pub frame_masks: Vec<Vec<u8>>,
    pub frame_delays: Vec<f32>,
    pub path: String,
}

/// Configuration for displaying animated GIFs on the loading screen
#[derive(Clone)]
#[warn(unused)]
pub struct GifLoadingScreenInfo {
    /// Path to the animated GIF file
    pub gif_path: String,
    /// X position of the GIF
    pub x: f32,
    /// Y position of the GIF
    pub y: f32,
    /// Width of the GIF display
    pub width: f32,
    /// Height of the GIF display
    pub height: f32,
    /// Whether the GIF should loop
    pub loop_animation: bool,
}

impl GifLoadingScreenInfo {
    /// Create a new GIF loading screen info
    #[allow(unused)]
    pub fn new(gif_path: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            gif_path,
            x,
            y,
            width,
            height,
            loop_animation: true,
        }
    }
}

/// Options for customizing the loading screen appearance
pub struct LoadingScreenOptions {
    /// Title displayed at the top of the loading screen (default: none)
    pub title: Option<String>,
    /// Background color of the loading screen (default: DARKGREEN)
    pub background_color: Color,
    /// Progress bar background color (default: DARKGRAY)
    pub bar_background_color: Color,
    /// Progress bar fill color (default: GREEN)
    pub bar_fill_color: Color,
    /// Text color for all text elements (default: WHITE)
    pub text_color: Color,
    /// File name text color (default: SKYBLUE)
    pub filename_color: Color,
    /// Font size for the title (default: 60)
    pub title_font_size: u16,
    /// Font size for progress text (default: 30)
    pub progress_font_size: u16,
    /// Font size for filename text (default: 20)
    pub filename_font_size: u16,
    /// Whether to show the "Loading Complete!" message (default: true)
    pub show_completion_message: bool,
    /// Custom completion message (default: "Loading Complete!")
    pub completion_message: String,
    /// Delay in seconds after completion before continuing (default: 0.5)
    pub completion_delay: f32,
    /// Animated GIFs to display during loading (default: empty)
    pub loading_screen_gifs: Vec<GifLoadingScreenInfo>,
}

impl Default for LoadingScreenOptions {
    fn default() -> Self {
        Self {
            title: None,
            background_color: DARKGREEN,
            bar_background_color: DARKGRAY,
            bar_fill_color: GREEN,
            text_color: WHITE,
            filename_color: SKYBLUE,
            title_font_size: 60,
            progress_font_size: 30,
            filename_font_size: 20,
            show_completion_message: true,
            completion_message: "Loading Complete!".to_string(),
            completion_delay: 0.5,
            loading_screen_gifs: Vec::new(),
        }
    }
}

/// A central texture manager to preload and share textures
/// This reduces memory usage and prevents flickering when switching images
#[derive(Clone)]
pub struct TextureManager {
    textures: Arc<Mutex<HashMap<String, (Texture2D, Option<Vec<u8>>)>>>,
    load_order: Arc<Mutex<Vec<String>>>, // Store just the order textures were loaded in
    animated_spritesheets: Arc<Mutex<HashMap<String, PreloadedAnimatedSpritesheet>>>,
    animated_spritesheet_order: Arc<Mutex<Vec<String>>>,
    animated_gifs: Arc<Mutex<HashMap<String, PreloadedAnimatedGif>>>,
    animated_gif_order: Arc<Mutex<Vec<String>>>,
    sounds: Arc<Mutex<HashMap<String, Sound>>>,
    sound_order: Arc<Mutex<Vec<String>>>,
}

impl TextureManager {
    /// Create a new texture manager
    pub fn new() -> Self {
        Self {
            textures: Arc::new(Mutex::new(HashMap::new())),
            load_order: Arc::new(Mutex::new(Vec::new())),
            animated_spritesheets: Arc::new(Mutex::new(HashMap::new())),
            animated_spritesheet_order: Arc::new(Mutex::new(Vec::new())),
            animated_gifs: Arc::new(Mutex::new(HashMap::new())),
            animated_gif_order: Arc::new(Mutex::new(Vec::new())),
            sounds: Arc::new(Mutex::new(HashMap::new())),
            sound_order: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Preload a texture by its file path
    pub async fn preload(&self, path: &str) {
        // First, check if the texture already exists
        let texture_exists = {
            let textures = self.textures.lock().unwrap();
            textures.contains_key(path)
        };

        // If it doesn't exist, load it
        if !texture_exists {
            // Load the texture outside of any locks
            let (texture, mask) = set_texture_main(path).await;

            // Now update the maps with short-lived locks
            {
                let mut textures = self.textures.lock().unwrap();
                textures.insert(path.to_string(), (texture, mask));
            }

            {
                let mut load_order = self.load_order.lock().unwrap();
                load_order.push(path.to_string());
            }
        }
    }

    /// Preload multiple textures at once
    #[allow(unused)]
    pub async fn preload_all<'a, T>(&self, paths: T)
    where
        T: AsRef<[&'a str]>,
    {
        let paths = paths.as_ref();
        for path in paths {
            self.preload(path).await;
        }
    }

    /// Get a preloaded texture for use in an ImageObject
    #[allow(unused)]
    pub fn get_preload(&self, path: &str) -> Option<(Texture2D, Option<Vec<u8>>, String)> {
        let textures = self.textures.lock().unwrap();
        textures
            .get(path)
            .map(|(texture, mask)| (texture.clone(), mask.clone(), path.to_string()))
    }

    /// Get a preloaded texture by its index in the preload order
    #[allow(unused)]
    pub fn get_preload_by_index(&self, index: usize) -> Option<(Texture2D, Option<Vec<u8>>, String)> {
        let load_order = self.load_order.lock().unwrap();
        if index < load_order.len() {
            let path = &load_order[index];
            self.get_preload(path)
        } else {
            None
        }
    }

    /// Get the number of preloaded textures
    #[allow(unused)]
    pub fn texture_count(&self) -> usize {
        let load_order = self.load_order.lock().unwrap();
        load_order.len()
    }

    /// Get a list of all preloaded texture paths in load order
    #[allow(unused)]
    pub fn get_texture_paths(&self) -> Vec<String> {
        let load_order = self.load_order.lock().unwrap();
        load_order.clone()
    }

    /// Preload a sound by its file path
    #[allow(unused)]
    pub async fn preload_sound(&self, path: &str) {
        let already_loaded = {
            let sounds = self.sounds.lock().unwrap();
            sounds.contains_key(path)
        };

        if already_loaded {
            return;
        }

        let sound = match load_sound(path).await {
            Ok(sound) => sound,
            Err(_) => return,
        };

        {
            let mut sounds = self.sounds.lock().unwrap();
            sounds.insert(path.to_string(), sound);
        }

        {
            let mut sound_order = self.sound_order.lock().unwrap();
            sound_order.push(path.to_string());
        }
    }

    /// Preload multiple sounds at once
    #[allow(unused)]
    pub async fn preload_sounds<'a, T>(&self, paths: T)
    where
        T: AsRef<[&'a str]>,
    {
        let paths = paths.as_ref();
        for path in paths {
            self.preload_sound(path).await;
        }
    }

    /// Get a preloaded sound by its file path
    #[allow(unused)]
    pub fn get_preloaded_sound(&self, path: &str) -> Option<Sound> {
        let sounds = self.sounds.lock().unwrap();
        sounds.get(path).cloned()
    }

    /// Get a preloaded sound by its index in preload order
    #[allow(unused)]
    pub fn get_preloaded_sound_by_index(&self, index: usize) -> Option<Sound> {
        let sound_order = self.sound_order.lock().unwrap();
        if index < sound_order.len() {
            let path = &sound_order[index];
            self.get_preloaded_sound(path)
        } else {
            None
        }
    }

    /// Get the number of preloaded sounds
    #[allow(unused)]
    pub fn sound_count(&self) -> usize {
        let sound_order = self.sound_order.lock().unwrap();
        sound_order.len()
    }

    /// Get a list of all preloaded sound paths in load order
    #[allow(unused)]
    pub fn get_sound_paths(&self) -> Vec<String> {
        let sound_order = self.sound_order.lock().unwrap();
        sound_order.clone()
    }

    /// Preload an animated spritesheet and its transparency mask
    #[allow(unused)]
    pub async fn preload_animated_spritesheet(&self, path: &str, cols: usize, rows: usize) {
        if cols == 0 || rows == 0 {
            return;
        }

        let already_loaded = {
            let spritesheets = self.animated_spritesheets.lock().unwrap();
            spritesheets.contains_key(path)
        };

        if already_loaded {
            return;
        }

        let (texture, transparency_mask) = set_texture_main(path).await;
        let preloaded = PreloadedAnimatedSpritesheet {
            texture,
            transparency_mask,
            path: path.to_string(),
            cols,
            rows,
        };

        {
            let mut spritesheets = self.animated_spritesheets.lock().unwrap();
            spritesheets.insert(path.to_string(), preloaded);
        }

        {
            let mut order = self.animated_spritesheet_order.lock().unwrap();
            order.push(path.to_string());
        }
    }

    /// Get a preloaded animated spritesheet by path
    #[allow(unused)]
    pub fn get_preloaded_animated_spritesheet(&self, path: &str) -> Option<PreloadedAnimatedSpritesheet> {
        let spritesheets = self.animated_spritesheets.lock().unwrap();
        spritesheets.get(path).cloned()
    }

    /// Get a preloaded animated spritesheet by index in preload order
    #[allow(unused)]
    pub fn get_preloaded_animated_spritesheet_by_index(&self, index: usize) -> Option<PreloadedAnimatedSpritesheet> {
        let order = self.animated_spritesheet_order.lock().unwrap();
        if index < order.len() {
            let path = &order[index];
            self.get_preloaded_animated_spritesheet(path)
        } else {
            None
        }
    }

    /// Preload an animated GIF and all frame metadata needed for playback
    #[allow(unused)]
    pub async fn preload_animated_gif(&self, path: &str) {
        let already_loaded = {
            let gifs = self.animated_gifs.lock().unwrap();
            gifs.contains_key(path)
        };

        if already_loaded {
            return;
        }

        let file_data = match load_file(path).await {
            Ok(data) => data,
            Err(_) => return,
        };

        let preloaded = match build_preloaded_gif(path, &file_data) {
            Some(data) => data,
            None => return,
        };

        {
            let mut gifs = self.animated_gifs.lock().unwrap();
            gifs.insert(path.to_string(), preloaded);
        }

        {
            let mut order = self.animated_gif_order.lock().unwrap();
            order.push(path.to_string());
        }
    }

    /// Get a preloaded animated GIF by path
    #[allow(unused)]
    pub fn get_preloaded_animated_gif(&self, path: &str) -> Option<PreloadedAnimatedGif> {
        let gifs = self.animated_gifs.lock().unwrap();
        gifs.get(path).cloned()
    }

    /// Get a preloaded animated GIF by index in preload order
    #[allow(unused)]
    pub fn get_preloaded_animated_gif_by_index(&self, index: usize) -> Option<PreloadedAnimatedGif> {
        let order = self.animated_gif_order.lock().unwrap();
        if index < order.len() {
            let path = &order[index];
            self.get_preloaded_animated_gif(path)
        } else {
            None
        }
    }

    /// Load assets with a built-in loading screen that works well for web
    /// This method handles all the complexities of asset loading and progress display
    pub async fn preload_with_loading_screen<'a, T>(&self, assets: T, sound_assets: Option<&[&'a str]>, options: Option<LoadingScreenOptions>)
    where
        T: AsRef<[&'a str]>,
    {
        let assets = assets.as_ref();
        let sound_assets = sound_assets.unwrap_or(&[]);
        // Use default options if none provided
        let options = options.unwrap_or_default();

        // Pre-preload any GIFs specified for the loading screen
        for gif_info in &options.loading_screen_gifs {
            self.preload_animated_gif(&gif_info.gif_path).await;
        }

        // Thread-safe progress counters that can be shared between coroutines
        let loaded_counter = Arc::new(AtomicUsize::new(0));
        let total_assets = assets.len() + sound_assets.len();

        // Create animation state trackers for each GIF
        #[allow(unused)]
        struct GifAnimationState {
            current_frame: usize,
            total_frames: usize,
            time_accumulated: f32,
            frame_durations: Vec<f32>,
            texture: Texture2D,
            frame_masks: Vec<Vec<u8>>,
        }

        let mut gif_states: Vec<GifAnimationState> = Vec::new();

        // Initialize animation states for each GIF
        for gif_info in &options.loading_screen_gifs {
            if let Some(preloaded_gif) = self.get_preloaded_animated_gif(&gif_info.gif_path) {
                gif_states.push(GifAnimationState {
                    current_frame: 0,
                    total_frames: preloaded_gif.frame_masks.len().max(1),
                    time_accumulated: 0.0,
                    frame_durations: preloaded_gif.frame_delays.clone(),
                    texture: preloaded_gif.texture.clone(),
                    frame_masks: preloaded_gif.frame_masks.clone(),
                });
            }
        }

        // Start a background coroutine for loading assets WITHOUT awaiting it
        // This is the key to avoiding black flashes on web
        {
            // Convert &[&str] to Vec<String> for the coroutine to own its data
            let assets_to_load: Vec<String> = assets.iter().map(|&s| s.to_string()).collect();
            let sounds_to_load: Vec<String> = sound_assets.iter().map(|&s| s.to_string()).collect();
            let load_queue: Vec<String> = assets_to_load.iter().chain(sounds_to_load.iter()).cloned().collect();
            let counter = loaded_counter.clone();
            let loading_tm = self.clone(); // Clone the TextureManager for the coroutine

            // Important: We start the coroutine but DON'T await it
            start_coroutine(async move {
                for asset_path in assets_to_load {
                    // Determine if this is a GIF or regular texture based on file extension
                    if asset_path.to_lowercase().ends_with(".gif") {
                        loading_tm.preload_animated_gif(&asset_path).await;
                    } else {
                        // Load asset into the shared texture manager
                        loading_tm.preload(&asset_path).await;
                    }

                    // Update the counter atomically
                    counter.fetch_add(1, Ordering::SeqCst);

                    // Yielding control back to the main thread
                    next_frame().await;
                }

                for sound_path in sounds_to_load {
                    loading_tm.preload_sound(&sound_path).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                    next_frame().await;
                }
            });

            let _ = load_queue;
        }

        // Main rendering loop for the loading screen
        // This runs in the main thread and never awaits the asset loading
        loop {
            // Read the current progress atomically
            let loaded_assets = loaded_counter.load(Ordering::SeqCst);
            let progress = loaded_assets as f32 / total_assets as f32;

            // Clear the screen with custom background color
            clear_background(options.background_color);

            // Draw title if one is provided
            if let Some(title) = &options.title {
                let title_size = options.title_font_size;
                let title_dim = measure_text(title, None, title_size, 1.0);
                draw_text(
                    title,
                    screen_width() / 2.0 - title_dim.width / 2.0,
                    screen_height() / 3.0,
                    title_size as f32,
                    options.text_color,
                );
            }

            // Update and draw animated GIFs
            let delta_time = get_frame_time();
            for (idx, gif_state) in gif_states.iter_mut().enumerate() {
                if gif_state.total_frames == 0 {
                    continue;
                }

                // Update animation
                gif_state.time_accumulated += delta_time;
                let frame_duration = if gif_state.current_frame < gif_state.frame_durations.len() {
                    gif_state.frame_durations[gif_state.current_frame]
                } else {
                    0.1
                };

                if gif_state.time_accumulated >= frame_duration {
                    gif_state.time_accumulated -= frame_duration;
                    gif_state.current_frame += 1;

                    if gif_state.current_frame >= gif_state.total_frames {
                        if idx < options.loading_screen_gifs.len() && options.loading_screen_gifs[idx].loop_animation {
                            gif_state.current_frame = 0;
                        } else {
                            gif_state.current_frame = gif_state.total_frames.saturating_sub(1);
                        }
                    }
                }

                // Draw the current frame
                if let Some(gif_info) = options.loading_screen_gifs.get(idx) {
                    let frame_width = gif_state.texture.width() / gif_state.total_frames as f32;
                    let frame_height = gif_state.texture.height();

                    draw_texture_ex(
                        &gif_state.texture,
                        gif_info.x,
                        gif_info.y,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(gif_state.current_frame as f32 * frame_width, 0.0, frame_width, frame_height)),
                            dest_size: Some(Vec2::new(gif_info.width, gif_info.height)),
                            ..Default::default()
                        },
                    );
                }
            }

            // Draw progress text
            let progress_text = format!("Loading: {:.0}%", progress * 100.0);
            draw_text(
                &progress_text,
                screen_width() / 2.0 - measure_text(&progress_text, None, options.progress_font_size, 1.0).width / 2.0,
                screen_height() / 2.0,
                options.progress_font_size as f32,
                options.text_color,
            );

            // Draw loading bar
            let bar_width = screen_width() * 0.6;
            let bar_height = 30.0;
            let bar_x = screen_width() / 2.0 - bar_width / 2.0;
            let bar_y = screen_height() / 2.0 + 40.0;

            // Background bar
            draw_rectangle(bar_x, bar_y, bar_width, bar_height, options.bar_background_color);

            // Progress bar
            if progress > 0.0 {
                draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, options.bar_fill_color);
            }

            // Border
            draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 2.0, options.text_color);

            // Display current file if available
            if loaded_assets > 0 && loaded_assets < total_assets {
                let file_name = if loaded_assets < assets.len() {
                    assets[loaded_assets].split('/').last().unwrap_or("")
                } else {
                    let sound_index = loaded_assets - assets.len();
                    sound_assets[sound_index].split('/').last().unwrap_or("")
                };
                let file_text = format!("Loading: {}", file_name);
                draw_text(
                    &file_text,
                    screen_width() / 2.0 - measure_text(&file_text, None, options.filename_font_size, 1.0).width / 2.0,
                    bar_y + bar_height + 30.0,
                    options.filename_font_size as f32,
                    options.filename_color,
                );
            }

            // Check if loading is complete
            if loaded_assets >= total_assets {
                // Show completion message if enabled
                if options.show_completion_message {
                    clear_background(options.background_color);

                    // Draw final GIF frames on completion screen
                    for (idx, gif_state) in gif_states.iter().enumerate() {
                        if let Some(gif_info) = options.loading_screen_gifs.get(idx) {
                            let frame_width = gif_state.texture.width() / gif_state.total_frames as f32;
                            let frame_height = gif_state.texture.height();

                            draw_texture_ex(
                                &gif_state.texture,
                                gif_info.x,
                                gif_info.y,
                                WHITE,
                                DrawTextureParams {
                                    source: Some(Rect::new(gif_state.current_frame as f32 * frame_width, 0.0, frame_width, frame_height)),
                                    dest_size: Some(Vec2::new(gif_info.width, gif_info.height)),
                                    ..Default::default()
                                },
                            );
                        }
                    }

                    let text_size = options.progress_font_size + 20; // Slightly larger than progress font
                    let text_dimensions = measure_text(&options.completion_message, None, text_size, 1.0);
                    let text_x = screen_width() / 2.0 - text_dimensions.width / 2.0;
                    let text_y = screen_height() / 2.0;

                    draw_text(&options.completion_message, text_x, text_y, text_size as f32, options.text_color);
                    next_frame().await;

                    // Apply completion delay if specified
                    if options.completion_delay > 0.0 {
                        let start_time = get_time();
                        while get_time() - start_time < options.completion_delay as f64 {
                            next_frame().await;
                        }
                    }
                }

                // Break the loading loop and proceed with the game
                break;
            }

            // Update the screen WITHOUT awaiting asset loading
            next_frame().await;
        }
    }
}

async fn generate_mask(texture_path: &str, width: usize, height: usize) -> Option<Vec<u8>> {
    let image = load_image(texture_path).await.unwrap();
    let pixels = image.bytes;

    if pixels.len() != width * height * 4 {
        return None;
    }

    let mut has_transparency = false;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let alpha = pixels[idx + 3];
            if alpha < 255 {
                has_transparency = true;
                break;
            }
        }
        if has_transparency {
            break;
        }
    }

    if !has_transparency {
        return None;
    }

    let mut mask = vec![0; (width * height + 7) / 8];
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let alpha = pixels[idx + 3];
            let mask_byte_idx = (y * width + x) / 8;
            let bit_offset = (y * width + x) % 8;

            if alpha > 0 {
                mask[mask_byte_idx] |= 1 << (7 - bit_offset);
            } else {
                mask[mask_byte_idx] &= !(1 << (7 - bit_offset));
            }
        }
    }

    Some(mask)
}

async fn set_texture_main(texture_path: &str) -> (Texture2D, Option<Vec<u8>>) {
    let texture = load_texture(texture_path).await.unwrap();
    texture.set_filter(FilterMode::Linear);
    let tex_width = texture.width() as usize;
    let tex_height = texture.height() as usize;
    let transparency_mask = generate_mask(texture_path, tex_width, tex_height).await;
    (texture, transparency_mask)
}

fn build_preloaded_gif(path: &str, data: &[u8]) -> Option<PreloadedAnimatedGif> {
    let (frames, delays, width_px, height_px) = process_gif_data(data)?;
    if frames.is_empty() {
        return None;
    }

    let frame_count = frames.len();
    let spritesheet_width = width_px * frame_count;
    let spritesheet_height = height_px;

    let mut combined_image = Image::gen_image_color(spritesheet_width as u16, spritesheet_height as u16, Color::new(0.0, 0.0, 0.0, 0.0));

    let mut frame_masks = Vec::with_capacity(frame_count);

    for (i, frame) in frames.iter().enumerate() {
        let x_offset = i * width_px;
        let mut frame_mask = vec![0; (width_px * height_px + 7) / 8];

        for y in 0..height_px {
            for x in 0..width_px {
                let src_idx = (y * width_px + x) * 4;
                let src_r = frame[src_idx];
                let src_g = frame[src_idx + 1];
                let src_b = frame[src_idx + 2];
                let src_a = frame[src_idx + 3];

                if src_a > 0 {
                    let mask_byte_idx = (y * width_px + x) / 8;
                    let bit_offset = (y * width_px + x) % 8;
                    frame_mask[mask_byte_idx] |= 1 << (7 - bit_offset);
                }

                let dest_x = x_offset + x;
                combined_image.set_pixel(
                    dest_x as u32,
                    y as u32,
                    Color::new(src_r as f32 / 255.0, src_g as f32 / 255.0, src_b as f32 / 255.0, src_a as f32 / 255.0),
                );
            }
        }

        frame_masks.push(frame_mask);
    }

    let texture_width = combined_image.width() as usize;
    let texture_height = combined_image.height() as usize;
    let mut transparency_mask = vec![0; (texture_width * texture_height + 7) / 8];

    for y in 0..texture_height {
        for x in 0..texture_width {
            let idx = (y * texture_width + x) * 4;
            let alpha = combined_image.bytes[idx + 3];
            let mask_byte_idx = (y * texture_width + x) / 8;
            let bit_offset = (y * texture_width + x) % 8;

            if alpha > 0 {
                transparency_mask[mask_byte_idx] |= 1 << (7 - bit_offset);
            }
        }
    }

    let texture = Texture2D::from_image(&combined_image);
    texture.set_filter(FilterMode::Nearest);

    Some(PreloadedAnimatedGif {
        texture,
        transparency_mask: Some(transparency_mask),
        frame_masks,
        frame_delays: delays,
        path: path.to_string(),
    })
}

fn process_gif_data(data: &[u8]) -> Option<(Vec<Vec<u8>>, Vec<f32>, usize, usize)> {
    let decoder = GifDecoder::new(Cursor::new(data)).ok()?;
    let (width, height) = decoder.dimensions();

    let frames = decoder.into_frames().collect_frames().ok()?;
    if frames.is_empty() {
        return None;
    }

    let mut frame_bytes = Vec::with_capacity(frames.len());
    let mut delays = Vec::with_capacity(frames.len());

    for frame in frames {
        let delay: std::time::Duration = frame.delay().into();
        let delay_sec = delay.as_secs_f32().max(0.001);
        delays.push(delay_sec);
        frame_bytes.push(frame.into_buffer().into_raw());
    }

    Some((frame_bytes, delays, width as usize, height as usize))
}
