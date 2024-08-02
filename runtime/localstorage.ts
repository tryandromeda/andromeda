// deno-lint-ignore-file no-unused-vars

/**
 * LocalStorage: a Storage class that stores data in the browser.
 */
const localStorage: Storage = {
    get length() {
        return 0;
    },
    getItem(key: string) {
        throw new Error("Not implemented");
    },
    setItem(key: string, value: string) {
        throw new Error("Not implemented");
    },
    removeItem(key: string) {
        throw new Error("Not implemented");
    },
    key(index: number) {
        throw new Error("Not implemented");
    },
    clear() {
        throw new Error("Not implemented");
    },
};
