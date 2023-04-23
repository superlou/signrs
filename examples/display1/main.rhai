let resolution = [960, 540];
let multisampling = 1;

let color_bg = new_color_from_rgb(0.0, 0.0, 0.0);
let color_green = new_color_from_rgba(0.0, 0.8, 0.1, 0.8);
let color_blue = new_color_from_rgba(0.1, 0.1, 0.8, 0.8);
let color_white = new_color_from_rgb(1.0, 1.0, 1.0);

let font = new_font("assets/Roboto-Regular.ttf");
let image = new_image("assets/background.jpg");
let seahorse = new_image("assets/seahorse.png");
let t = 0;

let fps = 0;

let data = #{};
watch_json("text.json", |new_data| {
    data = new_data;
});

fn offset(x, y, func) {
    push_offset(x, y);
    func.call();
    pop_offset();
}

fn draw(dt) {
    t += dt;
    clear_screen(color_bg);
    
    draw_image(image, 0.0, 0.0, 960., 540.);    
    
    draw_rectangle(
        40.0 + 50 * cos(t), 40.0 + 20 * sin(2*t),
        100.0, 100.0, color_green
    );
    
    let offset_x = 30 * cos(3 * t);
    let offset_y = 30 * sin(5 * t);
    
    draw_rectangle(50.0 + offset_x, 50.0 + offset_y, 140.0 + offset_x, 180.0 + offset_y, color_blue);
    
    draw_text("Hello, World!", font, 32., color_white,
              100. + 50 * cos(10 * t), 100.);
    
    fps = 0.9 * fps + 0.1 * (1 / dt);
    
    offset(800., 10., || {
        draw_text("FPS: " + str(fps, 1), font, 18., color_white, 0., 0.);
        draw_text("Time: " + str(now(), "%-I:%M:%S %P", true), font, 18., color_white, 0., 20.);        
    });

    draw_image(seahorse, 400., 150., 207., 212., 0.5 + 0.5 * sin(2 * t));
    
    for (text, i) in data.items {
        draw_text(text, font, 18., color_white, 100., 300. + i * 20.);
    }
}
