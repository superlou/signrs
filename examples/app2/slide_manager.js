class SlideManager {
    slides = [];
    activeIndex = 0;
    timeRemaining = 0;
    
    add(title, text, duration) {
        this.slides.push(new Slide(title, text, duration));    
    }
    
    draw(dt) {
        let title = this.slides[this.activeIndex].title;
        let body = this.slides[this.activeIndex].text;
        let duration = this.slides[this.activeIndex].duration;
        
        draw_text(font.light, title, 20, 20, color.title);
        draw_text(font.normal, body, 20, 50, color.body);
        
        let w = this.timeRemaining / duration * 640;
        draw_rectangle(0, 0, 640, 4, color.black);
        draw_rectangle(640 - w, 0, w, 4, color.white);
        
        this.timeRemaining -= dt;
        
        if (this.timeRemaining < 0) {
            this.activeIndex = (this.activeIndex + 1) % this.slides.length;
            this.timeRemaining = this.slides[this.activeIndex].duration;
        }
    }
}

class Slide {
    constructor(title, text, duration) {
        this.title = title;
        this.text = text;
        this.duration = duration;
    }
}

function newSlideManager() {
    return new SlideManager();
}