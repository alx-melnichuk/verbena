import { DOCUMENT, inject, Injectable } from '@angular/core';
import { environment as env } from "../../environments/environment";

// Color scheme
export const THEME_LIGHT = "light";
export const THEME_DARK = "dark";
// Theme
export const THEME_AZURE_ORANGE = "azure-orange";
export const THEME_CYAN_ORANGE = "cyan-orange";
export const THEME_VIOLET_CHARTREUSE = "violet-chartreuse";
export const THEME_ORANGE_MAGENTA = "orange-magenta";
export const THEME_MAGENTA_CYAN = "magenta-cyan";

export const THEME_LIST = [
    // `light-*`
    THEME_LIGHT + "-" + THEME_AZURE_ORANGE,
    THEME_LIGHT + "-" + THEME_CYAN_ORANGE,
    THEME_LIGHT + "-" + THEME_VIOLET_CHARTREUSE,
    THEME_LIGHT + "-" + THEME_ORANGE_MAGENTA,
    THEME_LIGHT + "-" + THEME_MAGENTA_CYAN,
    // `dark-*`
    THEME_DARK + "-" + THEME_AZURE_ORANGE,
    THEME_DARK + "-" + THEME_CYAN_ORANGE,
    THEME_DARK + "-" + THEME_VIOLET_CHARTREUSE,
    THEME_DARK + "-" + THEME_ORANGE_MAGENTA,
    THEME_DARK + "-" + THEME_MAGENTA_CYAN,
];

export const PATH_THEME = "theme";
const COLOR_SCHEME = "color-scheme";

@Injectable({
    providedIn: 'root',
})
export class ThemeSvr {
    private document: Document = inject(DOCUMENT);

    private currTheme: string = "";

    public get themeList(): string[] { return THEME_LIST; }
    public set themeList(_val: string[]) { }

    public get themeDefault(): string { return THEME_LIST[0]; }
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
