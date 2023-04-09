"use strict";

({ print, alert }) => {
  globalThis.print = (args) => {
    return print(args);
  };
  window.alert = (args) => {
    return alert(args);
  };
};
