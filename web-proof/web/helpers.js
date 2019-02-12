let timers = {};

export function start_timer() {
  let ident = Math.floor(100000000 + Math.random() * 900000000);

  timers[ident] = (new Date()).getTime();
  return ident;
}

export function finish_timer(timer_id) {
  return (new Date()).getTime() - timers[timer_id];
}
