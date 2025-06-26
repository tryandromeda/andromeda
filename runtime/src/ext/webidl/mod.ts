// deno-lint-ignore-file no-unused-vars
// deno-lint-ignore no-explicit-any
function configureInterface(interface_: any) {
  configureProperties(interface_);
  configureProperties(interface_.prototype);
  Object.defineProperty(interface_.prototype, Symbol.toStringTag, {
    // @ts-ignore:
    __proto__: null,
    value: interface_.name,
    enumerable: false,
    configurable: true,
    writable: false,
  });
}

// deno-lint-ignore no-explicit-any
function configureProperties(obj: any) {
  const descriptors = Object.getOwnPropertyDescriptors(obj);
  for (const key in descriptors) {
    if (!Object.hasOwn(descriptors, key)) {
      continue;
    }
    if (key === "constructor") continue;
    if (key === "prototype") continue;
    const descriptor = descriptors[key];
    if (
      Reflect.has(descriptor, "value") &&
      typeof descriptor.value === "function"
    ) {
      Object.defineProperty(obj, key, {
        // @ts-ignore:
        __proto__: null,
        enumerable: true,
        writable: true,
        configurable: true,
      });
    } else if (Reflect.has(descriptor, "get")) {
      Object.defineProperty(obj, key, {
        // @ts-ignore:
        __proto__: null,
        enumerable: true,
        configurable: true,
      });
    }
  }
}

// TODO: comment in nova support module
// export { configureInterface };
