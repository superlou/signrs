import "util" as util;
import "slide_manager" as slide_manager;

fn init() {
    this.resolution = [960, 540];
    this.multisampling = 1;
    this.color = #{
        bg: new_color_from_rgb(0., 0., 0.)
    };
    
    this.font = #{
        normal: new_font("assets/Roboto-Regular.ttf"),
        light: new_font("assets/Roboto-Thin.ttf"),
    };
    
    this.slide_manager = slide_manager::new_slide_manager();
    
    // this.slide_manager = #{
    //   draw: || print("here"),  
    // };
}

fn draw(dt) {
    clear_screen(this.color.bg);
    // this.slide_manager.draw();
    print(this.slide_manager.draw());
}
