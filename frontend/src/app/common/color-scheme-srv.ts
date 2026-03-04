import { DOCUMENT, inject, Injectable, Renderer2 } from "@angular/core";
import { environment } from "../../environments/environment";

// Color scheme constants

// Color scheme
export const SCHEME_LIGHT = "light";
export const SCHEME_DARK = "dark";

export const COLOR_SCHEME_LIGHT_AZURE_ORANGE = "light-azure-orange";
export const COLOR_SCHEME_LIGHT_CYAN_ORANGE = "light-cyan-orange";
export const COLOR_SCHEME_LIGHT_VIOLET_CHARTREUSE = "light-violet-chartreuse";
export const COLOR_SCHEME_LIGHT_ORANGE_MAGENTA = "light-orange-magenta";
export const COLOR_SCHEME_LIGHT_MAGENTA_CYAN = "light-magenta-cyan";

export const COLOR_SCHEME_DARK_AZURE_ORANGE = "dark-azure-orange";
export const COLOR_SCHEME_DARK_CYAN_MAGENTA = "dark-cyan-orange";
export const COLOR_SCHEME_DARK_VIOLET_CHARTREUSE = "dark-violet-chartreuse";
export const COLOR_SCHEME_DARK_ORANGE_MAGENTA = "dark-orange-magenta";
export const COLOR_SCHEME_DARK_MAGENTA_CYAN = "dark-magenta-cyan";

export const COLOR_SCHEME_LIST = [
    // `light-*`
    COLOR_SCHEME_LIGHT_AZURE_ORANGE,
    COLOR_SCHEME_LIGHT_CYAN_ORANGE,
    // COLOR_SCHEME_LIGHT_VIOLET_MAGENTA,
    COLOR_SCHEME_LIGHT_VIOLET_CHARTREUSE,
    COLOR_SCHEME_LIGHT_ORANGE_MAGENTA,
    COLOR_SCHEME_LIGHT_MAGENTA_CYAN,
    // `dark-*`
    COLOR_SCHEME_DARK_AZURE_ORANGE,
    COLOR_SCHEME_DARK_CYAN_MAGENTA,
    // COLOR_SCHEME_DARK_VIOLET_MAGENTA,
    COLOR_SCHEME_DARK_VIOLET_CHARTREUSE,
    COLOR_SCHEME_DARK_ORANGE_MAGENTA,
    COLOR_SCHEME_DARK_MAGENTA_CYAN,
];

export const THEME = "theme";
const COLOR_SCHEME = "color-scheme";

@Injectable({
    providedIn: "root",
})
export class ColorSchemeSrv {
    private document: Document = inject(DOCUMENT);

    private currColorScheme: string | null = null;

    // get theme(): string | null { return this.currColorScheme; }
    // set theme(value: string | null) { }

    constructor() {
        if (environment.logLevel > 0) { console.log(`ColorSchemeSrv(); // 5 service`); }
    }

    // ** Theme **

    public getColorScheme(): string | null {
        return this.currColorScheme;
    }

    public setColorScheme(value: string | null | undefined, renderer: Renderer2): void {
        const index = COLOR_SCHEME_LIST.indexOf(value || "");
        const theme = COLOR_SCHEME_LIST[index > -1 ? index : 0];
        if (this.currColorScheme != theme) {
            if (!!this.currColorScheme) {
                this.document.documentElement.style.setProperty(COLOR_SCHEME, null);
                this.document.documentElement.style.setProperty("--" + COLOR_SCHEME, null);
                renderer.removeClass(this.document.documentElement, this.currColorScheme);
                const scheme = this.currColorScheme.split("-")[0];
                if ([SCHEME_LIGHT, SCHEME_DARK].includes(scheme)) {
                    renderer.removeClass(this.document.documentElement, scheme);
                }
            }
            this.currColorScheme = theme;
            renderer.addClass(this.document.documentElement, theme);
            const scheme = this.currColorScheme.split("-")[0];
            if ([SCHEME_LIGHT, SCHEME_DARK].includes(scheme)) {
                this.document.documentElement.style.setProperty(COLOR_SCHEME, scheme);
                this.document.documentElement.style.setProperty("--" + COLOR_SCHEME, scheme);
                renderer.addClass(this.document.documentElement, scheme);
            }
            this.setIntoLocalStorage(theme);
        }
    }

    public getFromLocalStorage(): string | null {
        return localStorage.getItem(THEME);
    }
    public setIntoLocalStorage(theme: string): void {
        if (!!theme) {
            window.localStorage.setItem(THEME, theme);
        } else {
            window.localStorage.removeItem(THEME);
        }
    }
}
