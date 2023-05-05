function init() {
  let a = 3;
  a += 5;
}

let black = new Color(0.0, 0.0, 0.0);
let green = new Color(0.0, 1.0, 0.0, 0.8);
let blue = new Color(0.0, 0.0, 1.0, 0.8);
let white = new Color(1, 1, 1);
let t = 0;

let font = new Font("assets/Roboto-Regular.ttf");
let background = new Image("assets/background.jpg");
let seahorse = new Image("assets/seahorse.png");

let prev_fps = 0;

let data;
data = watch_json("text.json", (new_data) => {
  data = new_data;
});

function draw(dt) {
  t += dt;
  let fps = 0.1 * (1 / dt) + 0.9 * prev_fps;
  prev_fps = fps;
  clear_screen(black);
  
  draw_image(background, 0, 0, 640, 480);  
  
  draw_rectangle(
    40.0 + 50 * Math.cos(t), 40.0 + 20 * Math.sin(2*t),
    100.0, 100.0,
    green
  );

  let offset_x = 30 * Math.cos(3 * t);
  let offset_y = 30 * Math.sin(5 * t);
    
  draw_rectangle(
    50.0 + offset_x, 50.0 + offset_y,
    140.0 + offset_x, 180.0 + offset_y,
    blue
  );
  
  draw_text(font, `FPS: ${fps.toFixed(2)}`, 550, 10, white);
  
  draw_image(seahorse, 300, 100, 207, 212, 0.5 + 0.5 * Math.sin(2 * t));
  
  data.items.forEach((element, i) => {
    draw_text(font, element, 100, 350 + 18 * i, white);
  });
}