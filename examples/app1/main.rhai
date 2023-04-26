import "util" as util;

fn init() {
    this.resolution = [960, 540];
    this.multisampling = 1;    
    
    this.color_bg = new_color_from_rgb(0.0, 0.0, 0.0);
    this.color_green = new_color_from_rgba(0.0, 0.8, 0.1, 0.8);
    this.color_blue = new_color_from_rgba(0.1, 0.1, 0.8, 0.8);
    this.color_white = new_color_from_rgb(1.0, 1.0, 1.0);
    
    this.font = new_font("assets/Roboto-Regular.ttf");
    this.image = new_image("assets/background.jpg");
    this.seahorse = new_image("assets/seahorse.png");
    this.t = 0;
    
    this.fps = 0;

    this.data = watch_json("text.json", |new_data| {
        this.data = new_data;
    });
}

fn draw(dt) {
    this.t += dt;
    let t = this.t;
    clear_screen(this.color_bg);

    draw_image(this.image, 0.0, 0.0, 960., 540.);    
    
    draw_rectangle(
        40.0 + 50 * cos(t), 40.0 + 20 * sin(2*t),
        100.0, 100.0, this.color_green
    );

    let offset_x = 30 * cos(3 * t);
    let offset_y = 30 * sin(5 * t);
    
    draw_rectangle(50.0 + offset_x, 50.0 + offset_y, 140.0 + offset_x, 180.0 + offset_y, this.color_blue);

    draw_text("Hello, World!", this.font, 32., this.color_white,
              100. + 50 * cos(10 * t), 100.);

    this.fps = 0.9 * this.fps + 0.1 * (1 / dt);
    
    push_offset(800., 10.);
    draw_text("FPS: " + str(this.fps, 1), this.font, 18., this.color_white, 0., 0.);
    draw_text("Time: " + str(now(), "%-I:%M:%S %P", true), this.font, 18., this.color_white, 0., 20.);
    pop_offset();

    draw_image(this.seahorse, 400., 150., 207., 212., 0.5 + 0.5 * sin(2 * t));
    
    for (text, i) in this.data.items {
        draw_text(text, this.font, 18., this.color_white, 100., 300. + i * 20.);
    }
}
