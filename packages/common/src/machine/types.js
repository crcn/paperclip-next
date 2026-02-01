/**
 * Machine types - core state management infrastructure
 */
/**
 * Deep equality comparison
 */
function deepEqual(a, b) {
    if (a === b)
        return true;
    if (a == null || b == null)
        return a === b;
    if (typeof a !== typeof b)
        return false;
    if (Array.isArray(a) && Array.isArray(b)) {
        if (a.length !== b.length)
            return false;
        for (let i = 0; i < a.length; i++) {
            if (!deepEqual(a[i], b[i]))
                return false;
        }
        return true;
    }
    if (typeof a === "object" && typeof b === "object") {
        const aKeys = Object.keys(a);
        const bKeys = Object.keys(b);
        if (aKeys.length !== bKeys.length)
            return false;
        for (const key of aKeys) {
            if (!bKeys.includes(key) ||
                !deepEqual(a[key], b[key])) {
                return false;
            }
        }
        return true;
    }
    return false;
}
/**
 * Check if a selector's result has changed
 */
export function changed(prevState, newState, selector) {
    return !deepEqual(selector(prevState), selector(newState));
}
//# sourceMappingURL=types.js.map