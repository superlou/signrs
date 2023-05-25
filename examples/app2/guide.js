export default class Guide {
  items = []
  
  update(data) {
    this.items = data.map(item => {
      return new Event(item.name, item.location, item.start, item.finish);
    });
    
    this.items.sort((a, b) => a.start - b.start);
  }
  
  running(date) {
    return this.items.filter(item => {
      return item.start <= date && item.finish > date;
    });
  }
}

class Event {
  constructor(name, location, start, finish) {
    this.name = name;
    this.location = location;
    this.start = new Date(to_js_iso8061(start));
    this.finish = new Date(to_js_iso8061(finish));
    this.duration = this.finish - this.start;
  }
  
  get duration_hours() {
    return this.duration / 1000 / 3600;
  }
}

function to_js_iso8061(str) {
  return str.replace("+0000", "Z");
}