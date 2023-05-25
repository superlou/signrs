export let resolution = [960, 540];
export let multisampling = 1;

import Fps from "fps.js";
import Ticker from "ticker.js";
import SlideManager from "slide_manager.js";
import TextSlide from "text_slide.js";
import EventSlide from "event_slide.js";
import Guide from "guide.js";

let fps = new Fps(850, 10, 20);
let ticker = new Ticker();
let slideManager = new SlideManager();
let guide = new Guide();

let color = {
    black: new Color(0, 0, 0),
    white: new Color(1, 1, 1),
    red: new Color(1, 0, 0),
    yellow: new Color(1, 1, 0),
    background: new Color(0.8, 0.9, 1.0),
    title: new Color(0.1, 0.2, 0.5),
    body: new Color(0.1, 0.2, 0.5),
}

let font = {
    normal: new Font("assets/Roboto-Regular.ttf"),
    light: new Font("assets/Roboto-Thin.ttf"),
}

let runningSlide = new EventSlide("Happening Now");

watch_json("data/slides.json", (data) => {
    slideManager.clear();
    slideManager.add(runningSlide);
    
    data.forEach((slide) => {
        let text_slide = new TextSlide(slide.title, slide.body, slide.duration);
        slideManager.add(text_slide);
    })
});

watch_json("data/guide.json", (data) => {
    guide.update(data);
})

watch_json("data/ticker.json", (data) => {
   ticker.setMessages(data.messages); 
});

export function init() {
}

let i = 0;

export function draw(dt) {
    // Limit expensive calls until we have a way to run them in the background
    if (i++ === 0) {
        runningSlide.setItems(guide.running(new Date("2023-03-25T18:00:00.000000Z")));
    }
    i %= 180;
    
    clear_screen(color.background);
    slideManager.draw(dt, font, color);
    ticker.draw(dt, font, color);
    fps.draw(dt, font, color);
}
