fn offset(x, y, func) {
    push_offset(x, y);
    this.call(func);
    pop_offset();
}
