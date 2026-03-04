export class NavigatorUtil {
    public static isMobile(): boolean {
        if (typeof window === "undefined" || typeof window.navigator === "undefined") {
            return false;
        }
        return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(window.navigator.userAgent);
    }
    /**
     * Returns the language code name from the browser, e.g. "de"
     */
    public static getBrowserLang(): string | undefined {
        if (typeof window === "undefined" || !window.navigator) {
            return undefined;
        }
        const browserLang = this.getBrowserLocale();
        return browserLang ? browserLang.split(/[-_]/)[0] : undefined;
    }
    /**
     * Returns the culture language code name from the browser, e.g. "de-DE"
     */
    public static getBrowserLocale(): string | undefined {
        if (typeof window === "undefined" || typeof window.navigator === "undefined") {
            return undefined;
        }
        return window.navigator.languages
            ? window.navigator.languages[0]
            : window.navigator.language
            || (window.navigator as any).browserLanguage
            || (window.navigator as any).userLanguage;
    }
}
