import { DOCUMENT, inject, Injectable } from "@angular/core";
import { DateAdapter } from "@angular/material/core";
import { InterpolatableTranslationObject, LangChangeEvent, TranslateService, TranslationChangeEvent } from "@ngx-translate/core";
import { first, Observable } from "rxjs";
import { environment as env } from "../../environments/environment";

// Locale constants
export const PATH_LOCALE = "locale";
export const LOCALE_EN = "en-US";
export const LOCALE_DE = "de-DE";
export const LOCALE_UK = "uk-UA";
export const LOCALE_LIST = [LOCALE_EN, LOCALE_DE, LOCALE_UK];
export const LOCALE_DEFAULT = LOCALE_EN;
const ATTR_NAME_LANG = "lang";

@Injectable({
    providedIn: "root",
})
export class LocaleSrv {
    private document: Document = inject(DOCUMENT);
    private dateAdapter: DateAdapter<Date> = inject(DateAdapter<Date>);
    private translateSrv: TranslateService = inject(TranslateService);

    private currLocale: string = LOCALE_DEFAULT;

    public get onTranslationChange(): Observable<TranslationChangeEvent> {
        return this.translateSrv.onTranslationChange.asObservable();
    }
    public set onTranslationChange(_val: Observable<TranslationChangeEvent>) { }

    public get onLangChange(): Observable<LangChangeEvent> {
        return this.translateSrv.onLangChange.asObservable();
    }
    public set onLangChange(_val: Observable<LangChangeEvent>) { }

    public get localeList(): string[] { return LOCALE_LIST; }
    public set localeList(_val: string[]) { }

    public get localeDefault(): string { return LOCALE_EN; }
    public set localeDefault(_val: string) { }

    constructor() {
        if (env.logLevel & 4) { console.info(`LocaleSrv(); // 1 service`); }
    }

    // ** Public Api **

    public getLocale(): string {
        return this.currLocale;
    }

    public setLocale(value: string | null): Promise<boolean> {
        const locale = this.findLocale(LOCALE_LIST, value) || LOCALE_EN;
        if (!locale || LOCALE_LIST.indexOf(locale) == -1) {
            console.error(`Invalid locale value "${locale}" (available: "${LOCALE_LIST.join("\",\"")}").`);
            return Promise.reject();
        }
        if (this.currLocale == locale) {
            return Promise.resolve(true);
        }
        return new Promise<boolean>((resolve: (value: boolean) => void, reject: (reason: unknown) => void) => {
            this.translateSrv.use(locale).pipe(first())
                .subscribe({
                    next: () => {
                        this.currLocale = locale; // #? this.setIntoStorage(this.currLocale = locale);
                        this.dateAdapter.setLocale(locale);
                        const lang = this.currLocale.slice(0, 2);
                        const element: HTMLElement = this.document.documentElement;
                        element.setAttribute(ATTR_NAME_LANG, lang);
                        if (env.logLevel & 4) { console.info(`LocaleSrv() Locale.translate.use(${locale})...Ok`); }
                        resolve(true);
                    },
                    error: (err) => reject(err)
                });
        });
    }

    public getFromStorage(): string | null {
        return localStorage.getItem(PATH_LOCALE);
    }
    public saveToStorage(locale: string | undefined = this.getLocale()): void {
        if (!!locale) {
            window.localStorage.setItem(PATH_LOCALE, locale);
        } else {
            window.localStorage.removeItem(PATH_LOCALE);
        }
    }

    public findLocale(localeList: string[], value: string | null): string | null {
        let result: string | null = null;
        const value2 = (value || "").toLowerCase();
        for (let index = 0; !!value2 && index < localeList.length && !result; index++) {
            if (localeList[index].toLowerCase() == value2) {
                result = localeList[index];
            }
        }
        return result;
    }


    /** Get translation object by language. */
    public translationsByLang(lang: string): InterpolatableTranslationObject | undefined {
        return this.translateSrv.translations[lang];
    }

    // ** Private Api **

}


