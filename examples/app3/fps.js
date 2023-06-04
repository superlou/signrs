export default class Fps {
  prevFps = 0;
  
  update(dt) {
    let fps = 0.1 * (1 / dt) + 0.9 * this.prevFps;
    this.prevFps = fps;
    return fps;
  }
}
