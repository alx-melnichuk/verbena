export class ScrollElemUtil {
    /** Get the relative scroll offset value. */
    public static relativeOffset(scrollTop?: number, clientHeight?: number, scrollHeight?: number): number {
        const scrollTopAndHeight = (scrollTop || 0) + (clientHeight || 0);
        return Math.round((scrollTopAndHeight / (scrollHeight || 1)) * 100) / 100;
    }
}