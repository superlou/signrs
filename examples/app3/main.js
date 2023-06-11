let resolution = [960, 540];

import Fps from "fps.js";

export function init() {
  set_resolution(...resolution);
}

let black = new Color(0, 0, 0);
let white = new Color(1, 1, 1);
let font = new Font("Roboto-Regular.ttf");
let fps = new Fps();

export function draw(dt) {
  let drawStart = new Date();
  
  clear_screen(black);
  
  for (let i = 0; i < 240; i++) {
    draw_text(font, `Text string ${i}`,
              Math.floor(i / 24) * 96,
              (i % 24) * 20 + 50,
              14,
              white);
  }
  
  let fpsVal = fps.update(dt);
  draw_text(font, fpsVal.toFixed(2).toString(), 0, 0, 50, white);
  
  draw_text(font, `Cache lenth: ${font.cacheLength()}`, 200, 0, 50, white);
  
  let drawFinish = new Date();
  
  console.log("draw() time: " + (drawFinish - drawStart) + " ms");
}