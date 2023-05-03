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

function draw(dt) {
  t += dt;
  let fps = 1 / dt;
  clear_screen(black);
  
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
}