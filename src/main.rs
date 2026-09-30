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
use crate::ui::label::Label;
use crate::ui::text_input::TextInput;

/// Set up window settings before the app runs
fn window_conf() -> Conf {
    Conf {
        window_title: "gui_store".to_string(),
        window_width: 1300,
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
    let img_bread = StillImage::new(
        "assets/bread.png", 200.0, // width
        200.0, // height
        50.0, // x position
    50.0,  // y position
        true,  // Enable stretching
        1.0,   // Normal zoom (100%)
    )
    .await;
    let img_milk = StillImage::new(
        "assets/milk.png", 200.0, // width
        200.0, // height
        300.0, // x position
        50.0,  // y position
        true,  // Enable stretching
        1.0,   // Normal zoom (100%)
    )
    .await;
    let img_chips = StillImage::new(
        "assets/chips.png", 200.0, // width
        200.0, // height
        550.0, // x position
        50.0,  // y position
        true,  // Enable stretching
        1.0,   // Normal zoom (100%)
    )
    .await;
    let img_cookie = StillImage::new(
        "assets/cookie.png", 200.0, // width
        200.0, // height
        800.0, // x position
        50.0,  // y position
        true,  // Enable stretching
        1.0,   // Normal zoom (100%)
    )
    .await;
    let img_cream = StillImage::new(
        "assets/cream.png", 200.0, // width
        200.0, // height
        1050.0, // x position
        50.0,  // y position
        true,  // Enable stretching
        1.0,   // Normal zoom (100%)
    )
    .await;
    let mut btn_exit = TextButton::new(1050.0, 700.0, 200.0, 50.0, "Exit", WHITE, RED, 30);
    let mut btn_calc = TextButton::new(50.0, 700.0, 200.0, 50.0, "Calculate Total", WHITE, GREEN, 30);
    let mut txt_bread = TextInput::new(50.0, 250.0, 200.0, 50.0, 25.0);
    let mut txt_milk = TextInput::new(300.0, 250.0, 200.0, 50.0, 25.0);
    let mut txt_chips = TextInput::new(550.0, 250.0, 200.0, 50.0, 25.0);
    let mut txt_cookie = TextInput::new(800.0, 250.0, 200.0, 50.0, 25.0);
    let mut txt_cream = TextInput::new(1050.0, 250.0, 200.0, 50.0, 25.0);
    let mut btn_change = TextButton::new(50.0, 600.0, 300.0, 50.0, "Calculate Change", WHITE, GREEN, 30);
    let mut btn_restart = TextButton::new(1050.0, 600.0, 200.0, 50.0, "Restart", WHITE, RED, 30);
    let mut lbl_out = Label::new("", 55.0, 450.0, 30);
    let mut txt_input = TextInput::new(50.0, 500.0, 300.0, 50.0, 25.0);
    let mut total_cost = 0.0;
    let mut milk_total = 0.0;
    let mut chips_total = 0.0;
    let mut cookie_total = 0.0;
    let mut cream_total = 0.0;
    let mut bread_total = 0.0;
    btn_calc.with_text_color(BLACK);
    btn_change.with_text_color(BLACK);
    btn_restart.with_text_color(BLACK).with_hover_text_color(WHITE);
    btn_exit.with_text_color(BLACK).with_hover_text_color(WHITE);
    txt_input.set_max_chars(10);
    txt_input.set_allowed_chars("0123456789.");
    txt_input.set_enabled(false);
    txt_input.set_prompt("Input given $").set_allowed_chars("0123456789");
    txt_bread.set_prompt("Input # of Bread").set_allowed_chars("0123456789");
    txt_milk.set_prompt("Input # of Milk").set_allowed_chars("0123456789");
    txt_chips.set_prompt("Input # of Chips").set_allowed_chars("0123456789");
    txt_cookie.set_prompt("Input # of Cookie").set_allowed_chars("0123456789");
    txt_cream.set_prompt("Input # of Cream").set_allowed_chars("0123456789");
    btn_change.enabled=false;
    loop {
        clear_background(WHITE);
        draw_grid(50.0, BROWN);
        if btn_exit.click() {
            break;
        }
        if btn_restart.click() {
            milk_total = 0.0;
            chips_total = 0.0;
            cookie_total = 0.0;
            cream_total = 0.0;
            bread_total = 0.0;
            total_cost = 0.0;
            lbl_out.set_text("Cart: Nothing");
            txt_input.set_enabled(false);
            btn_change.enabled=false;
            btn_calc.enabled = true;
        }
        if btn_calc.click() {
            if milk_total == 0.0 && chips_total == 0.0 && cookie_total == 0.0 && cream_total == 0.0 && bread_total == 0.0 {
                lbl_out.set_text("Cart is empty. Please add items.");
            } else {
                total_cost = (milk_total * 2.99) + (chips_total * 3.50) + (cookie_total * 0.50) + (cream_total * 4.25) + (bread_total * 2.25);
                lbl_out.set_text(&format!("Total Cost: ${:.2}", total_cost));
                txt_input.set_enabled(true);
                btn_change.enabled=true;
                btn_calc.enabled = false;
            }
        }
        if btn_change.click() {
            let given_amount = txt_input.get_text().parse::<f32>();
            if let Ok(given_amount) = given_amount {
                let change = total_cost - given_amount;
                if change < 0.0 {
                    lbl_out.set_text(&format!("Change to give: ${:.2}", -change));
                } else if change == 0.0 {
                    lbl_out.set_text("No change to give.");
                } else {
                    lbl_out.set_text(&format!("Amount still owed: ${:.2}", change));
                }
            } else {
                lbl_out.set_text("Invalid input for given amount.");
            }
        }
        img_bread.draw();
        img_milk.draw();
        img_chips.draw();
        img_cookie.draw();
        img_cream.draw();
        lbl_out.draw();
        txt_input.draw();
        txt_bread.draw();
        txt_milk.draw();
        txt_chips.draw();
        txt_cookie.draw();
        txt_cream.draw();
        next_frame().await;
    }
}
