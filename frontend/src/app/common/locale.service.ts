import { Injectable } from "@angular/core";
import { DateAdapter } from "@angular/material/core";
import { TranslateService } from "@ngx-translate/core";
import { first } from 'rxjs/operators';

import { HttpErrorUtil } from "../utils/http-error.util";

import { ENV_IS_PROD, LOCALE_EN, LOCALE_LIST } from './constants';

export const LOCALE = 'locale';

@Injectable({
    providedIn: "root"
})
export class LocaleService {
    private currLocale: string | null = null;

    constructor(
        private dateAdapter: DateAdapter<Date>,
        private translate: TranslateService,
    ) {
        if (!ENV_IS_PROD) { console.log(`#1-LocaleService();`); }
    }

    // ** Locale **

    public getLocale(): string | null {
        return this.currLocale;
    }

    public setLocale(value: string | null): Promise<void> {
        const locale = this.findLocale(LOCALE_LIST, value) || LOCALE_EN;
        if (!locale || LOCALE_LIST.indexOf(locale) == -1) {
            console.error(`Invalid locale value "${locale}" (available: "${LOCALE_LIST.join('","')}").`);
            return Promise.reject();
        }
        if (this.currLocale == locale) {
            Promise.resolve();
        }
        return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
            this.translate.use(locale).pipe(first())
                .subscribe({
                    next: () => {
                        this.currLocale = locale;
                        this.dateAdapter.setLocale(locale);
                        HttpErrorUtil.setTranslate(this.translate);
                        window.localStorage.setItem(LOCALE, locale);
                        resolve();
                    },
                    error: (err) => reject(err)
                });
        });
    }

    public findLocale(localeList: string[], value: string | null): string | null {
        const localeList2: string[] = [];
        for (let index = 0; index < localeList.length; index++) {
            localeList2.push(localeList[index].toLowerCase());
        }
        return !!value && localeList2.indexOf(value.toLowerCase()) > -1 ? value : null;
    }

    public getLocaleFromLocalStorage(): string | null {
        return localStorage.getItem(LOCALE);
    }
}