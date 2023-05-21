export default class Ticker {
  messages = [];
  nextMessageId = 0;
  items = [];
  size = 24;
  width = 640;
  speed = 100;
  
  draw(dt, font, color) {
    if (this.items.length > 0) {
      this.items.forEach((item) => item.draw(dt * this.speed, 440, color));
    }
    
    if (this.messages.length > 0) {
      let endX = 0 
      
      if (this.items.length > 0) {
        endX = this.items.at(-1).endX;
        
        if (this.items[0].endX < 0) {
          this.items.shift();
        }
      }
      
      let safety = 100;
      
      while (endX < this.width && safety-- > 0) {
        let item = new TickerItem(this.messages[this.nextMessageId], this.size, endX, font);
        this.nextMessageId = (this.nextMessageId + 1) % this.messages.length;
        
        endX = item.endX;
        this.items.push(item);
      }
    }
  }
  
  setMessages(messages) {
    this.messages = messages;
  }
}

function newTicker() {
  return new Ticker();
}

class TickerItem { 
  constructor(text, size, startX, font) {
    this.text = text
    this.x = startX;
    let [w, h] = size_text(font.normal, text, size);
    this.w = w;
    this.h = h;
    this.font = font;
  }
  
  get endX() {
    return this.x + this.w;
  }
  
  draw(dx, y, color) {
    draw_text(this.font.normal, this.text, this.x, y, this.h, color.body);
    this.x -= dx;
  }
}