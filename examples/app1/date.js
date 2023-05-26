export function fmt_clock(date) {
  let h = date.getHours();
  let m = date.getMinutes();
  let s = date.getSeconds();
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
  let s_str = s.toString().padStart(2, "0");
  
  return `${h_str}:${m_str}:${s_str} ${ampm}`;
}