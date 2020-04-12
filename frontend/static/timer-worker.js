const timers = {};

function fireTimeout(id) {
  this.postMessage({id: id, variant: 'timeout'});
  delete timers[id];
}

function fireInterval(id) {
  this.postMessage({id: id, variant: 'interval'});
}

this.addEventListener('message', function(evt) {
  const data = evt.data;
  switch (data.command) {
    case 'setTimeout':
      const time = parseInt(data.timeout || 0, 10);
      const timer = setTimeout(fireTimeout.bind(null, data.id), time);
      timers[data.id] = timer;
      break;
    case 'setInterval':
      const interval = parseInt(data.interval || 0, 10);
      const handle = setInterval(fireInterval.bind(null, data.id), interval);
      timers[data.id] = handle;
      break;
    case 'clearTimeout':
      const clearTimeoutHandle = timers[data.id];
      if (clearTimeoutHandle) {
        clearTimeout(clearTimeoutHandle);
      }
      delete timers[data.id];
      break;
    case 'clearInterval':
      const clearIntervalHandle = timers[data.id];
      if (clearIntervalHandle) {
        clearInterval(clearIntervalHandle);
      }
      delete timers[data.id];
      break;
  }
});
