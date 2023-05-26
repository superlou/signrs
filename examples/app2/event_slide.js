export default class EventSlide {
  itemsPerPage = 6;
  pageDuration = 5;
  _items = [];
  
  constructor(title) {
    this.title = title;
    this.reset();
  }
  
  setItems(items) {
    this._items = items;
  }
  
  reset() {
    this.items = this._items;
    this.numPages = Math.ceil(this.items.length / this.itemsPerPage);
    this.activePage = 0;
    this.timeRemaining = this.numPages * this.pageDuration;
    this.duration = this.timeRemaining;
    this.pageTimeRemaining = this.pageDuration;
  }

  draw(dt, font, color) {
    this.timeRemaining -= dt;
    this.pageTimeRemaining -= dt;
    
    if (this.pageTimeRemaining <= 0) {
      this.activePage += 1;
      this.pageTimeRemaining = this.pageDuration;
    }
    
    let pageItems = this.items.slice(
      this.activePage * this.itemsPerPage,
      Math.min((this.activePage + 1) * this.itemsPerPage, this.items.length)
    );
    
    let y0 = 100;
    
    draw_text(font.light, this.title, 20, 24, 64, color.title);
    
    pageItems.slice(this.firstItem).forEach((item, i) => {
      with_offset(0, y0 + i * 60, () => {
        draw_rectangle(15, 0, 70, 55, color.body);
        let [w, h] = size_text(font.normal, fmtTime(item.start), 28);
        
        draw_text(font.normal, fmtTime(item.start), 20 + (60 - w), 4, 28, color.white);
        draw_text(font.normal, fmtAmPm(item.start), 50, 28, 24, color.white);
        
        draw_text(font.normal, item.name, 90, 0, 36, color.body);
        draw_text(font.normal, item.location, 90, 30, 24, color.body);
      });
    });
  }
}

function fmtTime(date) {
  let h = date.getHours();
  let m = date.getMinutes();
  let ampm = "am";
  
  if (h === 0) {
    h = 12;
  } else if (h === 12) {
    ampm = "pm"
  } else if (h > 12) {
    h -= 12;
    ampm = "pm";
  }
  
  let h_str = h.toString().padStart(2);
  let m_str = m.toString().padStart(2, "0");
  
  return `${h_str}:${m_str}`;
}

function fmtAmPm(date) {
  if (date.getHours() >= 12) {
    return "pm";
  } else {
    return "am";
  }
}