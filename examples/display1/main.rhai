let resolution = [640, ];

let color_bg = new_color_from_rgb(0.0, 0.0, 0.0);
let color_green = new_color_from_rgba(0.0, 0.8, 0.1, 0.8);
let color_blue = new_color_from_rgba(0.1, 0.1, 0.8, 0.8);

let t = 0;

fn draw(dt) { 
    t += dt;
    clear_screen(color_bg);
    draw_rectangle(
        40.0 + 50 * cos(t), 40.0 + 20 * sin(2*t),
        100.0, 100.0, color_green
    );
    
    let offset_x = 30 * cos(3 * t);
    let offset_y = 30 * sin(5 * t);
    
    draw_rectangle(50.0 + offset_x, 50.0 + offset_y, 140.0 + offset_x, 180.0 + offset_y, color_blue);
}
