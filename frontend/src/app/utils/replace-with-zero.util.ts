
export class ReplaceWithZeroUtil {
    /** Replace the symbol "_" with the symbol "_" + "ZeroSpace". */
    public static replace(value: string | null | undefined, templet?: string): string {
        let result: string = value || '';
        if (!!result) {
            const templetVal = templet || '_';
            const ch1 = String.fromCharCode(0x200B); // "empty space" character for line breaks.
            if (result.indexOf(templetVal) > -1) {
                result = result.replaceAll(templetVal, templetVal + ch1);
            } else {
                const idx = Math.round(result.length / 2);
                result = result.slice(0, idx) + ch1 + result.slice(idx);
            }
        }
        return result;
    }
}