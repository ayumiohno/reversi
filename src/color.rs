pub type Color = bool;

pub fn print_color(has_stone: bool, color: Color) {
    if !has_stone {
        print!("  ");
    } else if color {
        print!("X ");
    } else {
        print!("O ");
    }
}
