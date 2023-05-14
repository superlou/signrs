var resolution = [640, 480];
var multisampling = 1;

include("slide_manager.js");

let color = {
    black: new Color(0, 0, 0),
    white: new Color(1, 1, 1),
    background: new Color(0.8, 0.9, 1.0),
    title: new Color(0.1, 0.2, 0.5),
    body: new Color(0.1, 0.2, 0.5),
}

let font = {
    normal: new Font("assets/Roboto-Regular.ttf"),
    light: new Font("assets/Roboto-Thin.ttf"),
}

let slideManager = newSlideManager();
slideManager.add("Slide 1", "This is some text", 5);
slideManager.add("Slide 2", "This is different text", 5);
slideManager.add("Last Slide", "The very last slide has different text", 5);

function init() {}

function draw(dt) {
    clear_screen(color.background);
    slideManager.draw(dt);
}
