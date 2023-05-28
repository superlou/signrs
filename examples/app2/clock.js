import { fmtTime, fmtAmPm } from "event_slide.js";

export default class Clock {
  draw(date, width, height, fontSize, font, color, bg_color) {
    draw_rectangle(0, 0, width, height, bg_color)
    
    let text = fmtTime(date) + " " + fmtAmPm(date);
    
    let [w, h] = size_text(font, text, fontSize);
    draw_text(font, text, (width - w) / 2, (height - h) / 2, fontSize, color);
  }
}
