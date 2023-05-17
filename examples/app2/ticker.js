class Ticker {
  messages = [];
  nextMessageId = 0;
  items = [];
  size = 24;
  width = 640;
  speed = 100;
  
  draw(dt) {
    if (this.items.length > 0) {
      this.items.forEach((item) => item.draw(dt * this.speed, 440));
    }
    
    if (this.messages.length > 0) {
      let endX = 0 
      
      if (this.items.length > 0) {
        endX = this.items.at(-1).endX;
      }
      
      let safety = 100;
      
      while (endX < this.width && safety-- > 0) {
        let item = new TickerItem(this.messages[this.nextMessageId], this.size, endX);
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
  constructor(text, size, startX) {
    this.text = text
    this.x = startX;
    let [w, h] = size_text(font.normal, text, size);
    this.w = w;
    this.h = h;
  }
  
  get endX() {
    return this.x + this.w;
  }
  
  draw(dx, y) {
    draw_text(font.normal, this.text, this.x, y, this.h, color.body);
    this.x -= dx;
  }
}