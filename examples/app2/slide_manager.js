class SlideManager {
    slides = [];
    activeIndex = 0;
    timeRemaining = null;
    
    add(title, text, duration) {
        this.slides.push(new Slide(title, text, duration));    
    }
    
    draw(dt) {
        if (this.slides.length == 0) {
            return;
        }
        
        if (this.timeRemaining == null) {
            this.timeRemaining = this.slides[this.activeIndex].duration;
        }
        
        let slide = this.slides[this.activeIndex];
        
        let title = slide.title;
        let body = slide.text;
        let duration = slide.duration;
        
        draw_text(font.light, title, 20, 24, 64, color.title);
        draw_text(font.normal, body, 20, 96, 18, color.body);
        
        let w = this.timeRemaining / duration * 640;
        draw_rectangle(0, 0, 640, 4, color.black);
        draw_rectangle(640 - w, 0, w, 4, color.white);
        
        this.timeRemaining -= dt;
        
        if (this.timeRemaining < 0) {
            this.activeIndex = (this.activeIndex + 1) % this.slides.length;
            this.timeRemaining = this.slides[this.activeIndex].duration;
        }
    }
    
    clear() {
        this.slides = [];
        this.activeIndex = 0;
        this.timeRemaining = null;
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