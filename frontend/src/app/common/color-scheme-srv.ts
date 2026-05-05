import { DOCUMENT, inject, Injectable, Renderer2 } from "@angular/core";
import { environment } from "../../environments/environment";

// Color scheme constants

// Color scheme
export const SCHEME_LIGHT = "light";
export const SCHEME_DARK = "dark";

export const SCHEME_AZURE_ORANGE = "azure-orange";
export const SCHEME_CYAN_ORANGE = "cyan-orange";
export const SCHEME_VIOLET_CHARTREUSE = "violet-chartreuse";
export const SCHEME_ORANGE_MAGENTA = "orange-magenta";
export const SCHEME_MAGENTA_CYAN = "magenta-cyan";

export const COLOR_SCHEME_LIST = [
    // `light-*`
    SCHEME_LIGHT + "-" + SCHEME_AZURE_ORANGE,
    SCHEME_LIGHT + "-" + SCHEME_CYAN_ORANGE,
    SCHEME_LIGHT + "-" + SCHEME_VIOLET_CHARTREUSE,
    SCHEME_LIGHT + "-" + SCHEME_ORANGE_MAGENTA,
    SCHEME_LIGHT + "-" + SCHEME_MAGENTA_CYAN,
    // `dark-*`
    SCHEME_DARK + "-" + SCHEME_AZURE_ORANGE,
    SCHEME_DARK + "-" + SCHEME_CYAN_ORANGE,
    SCHEME_DARK + "-" + SCHEME_VIOLET_CHARTREUSE,
    SCHEME_DARK + "-" + SCHEME_ORANGE_MAGENTA,
    SCHEME_DARK + "-" + SCHEME_MAGENTA_CYAN,
];

export const THEME = "theme";
const COLOR_SCHEME = "color-scheme";

@Injectable({
    providedIn: "root",
})
export class ColorSchemeSrv {
    private document: Document = inject(DOCUMENT);

    private schemeName: string | null = null;
    private schemeLight: string | null = null;

    constructor() {
        if (environment.logLevel > 0) { console.log(`ColorSchemeSrv(); // 5 service`); }
    }

    // ** Theme **

    public getSchemeLight(): string | null {
        return this.schemeLight;
    }
    public setSchemeLight(value: string | null, renderer: Renderer2): void {
        const element: HTMLElement = this.document.documentElement;
        if (!!value && this.schemeLight != value && this.checkSchemeLight(value)) {
            if (!!this.schemeLight) {
                renderer.removeClass(element, this.schemeLight);
            }
            this.schemeLight = value;
            element.style.setProperty(COLOR_SCHEME, value);
            element.style.setProperty("--" + COLOR_SCHEME, value);
            renderer.addClass(element, value);
        }
    }
    public checkSchemeLight(value: string | null): boolean {
        return [SCHEME_LIGHT, SCHEME_DARK].indexOf(value || "") > -1;
    }

    public getSchemeName(): string | null {
        return this.schemeName;
    }
    public setSchemeName(value: string | null, renderer: Renderer2): void {
        const element: HTMLElement = this.document.documentElement;
        if (!!value && this.schemeName != value && this.checkSchemeName(value)) {
            if (!!this.schemeName) {
                renderer.removeClass(element, this.schemeName);
            }
            this.schemeName = value;
            renderer.addClass(element, value);
        }
    }
    public checkSchemeName(value: string | null): boolean {
        return [
            SCHEME_AZURE_ORANGE, SCHEME_CYAN_ORANGE, SCHEME_VIOLET_CHARTREUSE, SCHEME_ORANGE_MAGENTA, SCHEME_MAGENTA_CYAN
        ].indexOf(value || "") > -1;
    }

    public getSchemeLightName(): string | null {
        return this.schemeLight + "-" + this.schemeName;
    }
    public setSchemeLightName(value: string | null, renderer: Renderer2): void {
        const index = COLOR_SCHEME_LIST.indexOf(value || "");
        const schemeValue = COLOR_SCHEME_LIST[index > -1 ? index : 0];

        const idx = schemeValue.indexOf("-");
        const schemeLight = idx > -1 ? schemeValue.slice(0, idx) : "";
        const schemeName = idx > -1 ? schemeValue.slice(idx + 1) : "";

        if (this.checkSchemeLight(schemeLight) && this.checkSchemeName(schemeName)) {
            this.setSchemeLight(schemeLight, renderer);
            this.setSchemeName(schemeName, renderer);
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
