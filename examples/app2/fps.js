export default class Fps {
  prevFps = 0;
  
  constructor(x, y, size) {
    this.x = x;
    this.y = y;
    this.size = size;
  }
  
  draw(dt, font, color) {
    let fps = 0.1 * (1 / dt) + 0.9 * this.prevFps;
    this.prevFps = fps;
    
    let c = color.white;
    
    if (fps < 58) {
      c = color.red;
    } else if (fps > 62) {
      c = color.yellow;
    }

    draw_text(font.normal, `FPS: ${fps.toFixed(2)}`, this.x, this.y, this.size, c);  
  }
}

function newFps(x, y, size) {
  return new Fps(x, y, size);
}