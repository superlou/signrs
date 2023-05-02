function init() {
  let a = 3;
  a += 5;
}

let t = 0;

function draw(dt) {
  t += dt;
  let black = new Color(0.0, 0.0, 0.0);
  clear_screen(black);  
}