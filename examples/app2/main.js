var resolution = [640, 480];
var multisampling = 1;

include("slide_manager.js");
include("ticker.js");
include("fps.js");

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

let slideManager = newSlideManager();
let ticker = newTicker();
let fps = newFps(540, 10, 20);

watch_json("slides.json", (data) => {
    slideManager.clear();
    data.forEach((slide) => {
        slideManager.add(slide.title, slide.body, slide.duration);
    })
});

watch_json("ticker.json", (data) => {
   ticker.setMessages(data.messages); 
});

function init() {}

function draw(dt) {
    clear_screen(color.background);
    slideManager.draw(dt);
    ticker.draw(dt);
    fps.draw(dt);
}
