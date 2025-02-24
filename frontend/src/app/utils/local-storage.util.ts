export class LocalStorageUtil {
    // Add or remove a value from the store.
    public static update(name: string, value: string | null): void {
        if (!!name) {
            if (!!value) {
                localStorage.setItem(name, value);
            } else {
                localStorage.removeItem(name);
            }
        }
    }
}
