export default class TextSlide {
    constructor(title, text, duration) {
        this.title = title;
        this.text = text;
        this.duration = duration;
        this.reset();
    }
    
    reset() {
        this.timeRemaining = this.duration;
    }
    
    draw(dt, font, color) {
        this.timeRemaining -= dt;   
        draw_text(font.light, this.title, 20, 24, 64, color.title);
        draw_text(font.normal, this.text, 20, 96, 18, color.body);        
    }
}