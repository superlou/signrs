export default class SlideManager {
    slides = [];
    activeIndex = 0;
    timeRemaining = null;
    activeSlide = null;
    
    add(slide) {
        this.slides.push(slide);
    }
    
    draw(dt, font, color) {
        if (this.slides.length == 0) {
            return;
        }
        
        if (this.activeSlide == null) {
            this.activeSlide = this.slides[this.activeIndex];
            this.activeSlide.reset();
        }
        
        let slide = this.activeSlide;
        slide.draw(dt, font, color);
        
        let w = slide.timeRemaining / slide.duration * 960;
        draw_rectangle(0, 0, 960, 4, color.black);
        draw_rectangle(960 - w, 0, w, 4, color.white);
                
        if (this.activeSlide.timeRemaining < 0) {
            this.activeIndex = (this.activeIndex + 1) % this.slides.length;
            this.activeSlide = this.slides[this.activeIndex];
            this.activeSlide.reset();
        }
    }
    
    clear() {
        this.slides = [];
        this.activeIndex = 0;
    }
}
