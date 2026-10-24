// A stand-in for the object Tauri injects into the webview. The island talks to
// Rust through it, so the tests see every command the island sends and can fire
// the events Rust would emit — through the real bridge, with no source changes.

/** Every command the island invoked, oldest first: [name, args]. */
export const calls = [];
export const subscriptions = [];

const listeners = new Map();
const listenerIds = new Map();let nextListenerId=0;
export const eventInternals={unregisterListener(event,id){if(listenerIds.get(event)===id){listeners.delete(event);listenerIds.delete(event);}}};

export const internals = {
  transformCallback: (callback) => callback,
  invoke: async (cmd, args) => {
    if (cmd === "plugin:event|listen") {
      subscriptions.push({ event: args.event, target: args.target });
      const id=++nextListenerId;listeners.set(args.event, args.handler);listenerIds.set(args.event,id);
      return id;
    }
    if(cmd === "plugin:event|unlisten") return null;
    calls.push([cmd, args]);
    return null;
  },
};

/** Fires an event the way the Rust side would. */
export function emit(event, payload) {
  const handler = listeners.get(event);
  if (!handler) throw new Error(`nothing listens to "${event}"`);
  handler({ event, id: 0, payload });
}

/** The commands invoked under one name, as a list of their arguments. */
export function sent(cmd) {
  return calls.filter(([name]) => name === cmd).map(([, args]) => args);
}
