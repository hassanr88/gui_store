/*
By: Hassan Rana
Date: 2026-09-28
Program Details: You have been hired by the owner of "Bob's General Store" to create a program that all his employees can use to find the cost and change for each purchase.
Bob's store has 5 items that it sells and each item is a different cost that you can choice.
Once the employee calculates the total for the customer it should then let them input how much money they were given and tell them the change they need to give to the customer.
The program should be easy to use and should catch all possible errors.
It should also only display 2 decimals and look nice.
*/

mod ui;
mod utils;

use crate::ui::grid::draw_grid;
use crate::ui::still_image::StillImage;
use crate::ui::text_button::TextButton;
use crate::utils::preload_image::TextureManager;
use macroquad::prelude::*;

/// Set up window settings before the app runs
fn window_conf() -> Conf {
    Conf {
        window_title: "gui_store".to_string(),
        window_width: 1024,
        window_height: 768,
        fullscreen: false,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4, // MSAA: makes shapes look smoother
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let tm = TextureManager::new();
    tm.preload_with_loading_screen(&["assets/bread.png", "assets/milk.png", "assets/chips.png", "assets/cookie.png", "assets/cream.png"], None, None).await;
    let img = StillImage::new(
        "assets/register.png", 200.0, // width
        200.0, // height
        200.0, // x position
        60.0,  // y position
        true,  // Enable stretching
        1.0,   // Normal zoom (100%)
    )
    .await;
    let mut btn_exit = TextButton::new(800.0, 700.0, 200.0, 50.0, "Exit", WHITE, RED, 30);
    let mut btn_calc = TextButton::new(100.0, 700.0, 200.0, 50.0, "Calculate", WHITE, GREEN, 30);
    btn_calc.with_text_color(BLACK);
    btn_exit.with_text_color(BLACK).with_hover_text_color(WHITE);
    loop {
        clear_background(WHITE);
        draw_grid(50.0, BROWN);
        if btn_exit.click() {
            break;
        }
        if btn_calc.click() {
            
        }
        img.draw();
        next_frame().await;
    }
}
