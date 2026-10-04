import { DOCUMENT, inject, Injectable } from '@angular/core';
import { environment as env } from "../../environments/environment";

// Color scheme
export const SCHEME_LIGHT = "light";
export const SCHEME_DARK = "dark";
// Theme
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

export const PATH_THEME = "theme";
const COLOR_SCHEME = "color-scheme";

@Injectable({
    providedIn: 'root',
})
export class ThemeSvr {
    private document: Document = inject(DOCUMENT);

    private currTheme: string = "";

    public get themeList(): string[] { return COLOR_SCHEME_LIST; }
    public set themeList(_val: string[]) { }

    public get themeDefault(): string { return COLOR_SCHEME_LIST[0]; }
    public set themeDefault(_val: string) { }

    constructor() {
        if (env.logLevel & 4) { console.info(`ThemeSvr(); // 7 service`); }
    }

    // ** Public API **

    public getTheme(): string {
        return this.currTheme;
    }
    private getColorSchemeAndTheme(value: string): { colorScheme: string, theme: string } {
        const idx = value.indexOf("-");
        const colorScheme = idx > -1 ? value.slice(0, idx) : "";
        const theme = idx > -1 ? value.slice(idx + 1) : "";
        return { colorScheme, theme };
    }
    public setTheme(value: string | null | undefined): void {
        const newTheme = !!value && this.themeList.includes(value || "") ? value : null;
        if (!newTheme || newTheme == this.currTheme) {
            return;
        }
        // const element: HTMLElement = this.document.body;
        const element: HTMLElement = this.document.documentElement;

        const currVal = this.getColorSchemeAndTheme(this.currTheme);
        if (!!currVal.colorScheme) {
            element.classList.remove(currVal.colorScheme);
        }
        if (!!currVal.theme) {
            element.classList.remove(currVal.theme);
        }

        this.currTheme = newTheme;

        const newVal = this.getColorSchemeAndTheme(this.currTheme);

        element.style.setProperty(COLOR_SCHEME, newVal.colorScheme);
        element.style.setProperty("--" + COLOR_SCHEME, newVal.colorScheme);

        element.classList.add(newVal.colorScheme);
        element.classList.add(newVal.theme);
    }

    public getFromStorage(): string | null {
        return localStorage.getItem(PATH_THEME);
    }
    public saveToStorage(theme: string | undefined = this.getTheme()): void {
        if (!!theme) {
            window.localStorage.setItem(PATH_THEME, theme);
        } else {
            window.localStorage.removeItem(PATH_THEME);
        }
    }

    // ** Private Api **

}
