const isDev = import.meta.env.DEV;

export const debug = {
  log: (...args: any[]) => {
    if (isDev) console.log(...args);
  },
  warn: console.warn,  // 保留 warn
  error: console.error, // 保留 error
  info: (...args: any[]) => {
    if (isDev) console.info(...args);
  },
};
