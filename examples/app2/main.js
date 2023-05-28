export let resolution = [960, 540];
export let multisampling = 1;

import Fps from "fps.js";
import Ticker from "ticker.js";
import SlideManager from "slide_manager.js";
import TextSlide from "text_slide.js";
import EventSlide from "event_slide.js";
import Guide from "guide.js";
import Clock from "clock.js";

let fps = new Fps(850, 10, 20);
let ticker = new Ticker();
let slideManager = new SlideManager();
let guide = new Guide();
let clock = new Clock();

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

let debug = {};
watch_json("data/debug.json", data => {
    debug.forceNow = "now" in data;
    debug.now = ("now" in data) ? new Date(data.now) : new Date();
});

export function init() {
}

let i = 0;

export function draw(dt) {
    let now = new Date();
    
    if (debug.forceNow) {
        debug.now.setMilliseconds(debug.now.getMilliseconds() + 1000 * dt);
        now = debug.now;
    };
    
    // Limit expensive calls until we have a way to run them in the background
    if (i++ === 0) {
        runningSlide.setItems(guide.running(now));
    }
    i %= 180;
    
    clear_screen(color.background);
    slideManager.draw(dt, font, color);
    ticker.draw(dt, font, color);
    fps.draw(dt, font, color);
    
    with_offset(960 - 150, 540 - 50, () => {
        clock.draw(debug.now, 150, 50, 32, font.normal, color.white, color.body);
    });
}
