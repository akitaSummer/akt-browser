"use strict";

({ print }) => {
  globalThis.print = (args) => {
    return print(args);
  };
};
