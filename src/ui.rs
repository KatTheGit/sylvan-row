use macroquad::prelude::*;
use crate::common::Vector2;

pub fn button(position: Vector2, size: Vector2, text: &str) -> bool {
    draw_rectangle(position.x, position.y, size.x, size.y, LIGHTGRAY);
    draw_text(text, position.x, position.y + size.y / 2.0, 40.0, BLACK);
    let mouse: Vector2 = Vector2 {x:mouse_position().0, y: mouse_position().1};
    if mouse.x > position.x && mouse.x < (position.x + size.x) {
        if mouse.y > position.y && mouse.y < (position.y + size.y) {
            draw_rectangle(position.x, position.y, size.x, size.y,GRAY);
            draw_text(text, position.x, position.y + size.y / 2.0, 40.0, BLACK);
            if is_mouse_button_down(MouseButton::Left) {
                return true;
            }
        }
    }
    return false;
}