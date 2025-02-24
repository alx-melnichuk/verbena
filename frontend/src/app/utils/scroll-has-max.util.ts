export class ScrollHasMaxUtil {
    /** Check the scroll shift value to the maximum. */
    public static check(scrollTop?: number, clientHeight?: number, scrollHeight?: number): boolean {
        const scrollTopAndHeight = (scrollTop || 0) + (clientHeight || 0);
        const result = Math.round((scrollTopAndHeight / (scrollHeight || 1)) * 100) / 100;
        return result > 0.98;
    }
}