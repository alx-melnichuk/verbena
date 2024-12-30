import { Injectable } from "@angular/core";
import { DateAdapter } from "@angular/material/core";
import { TranslateService } from "@ngx-translate/core";
import { first } from 'rxjs/operators';

import { LOCALE_EN, LOCALE_LIST } from './constants';
import { HttpErrorUtil } from "../utils/http-error.util";

@Injectable({
  providedIn: "root"
})
export class LocaleService {
  private currLocale: string | null = null;

  constructor(
    private dateAdapter: DateAdapter<any>,
    private translate: TranslateService,
  ) {
    console.log(`#1-LocaleService();`); // #
  }

  // ** Locale ** 
  public getLocale(): string | null {
    return this.currLocale;
  }
  
  public setLocale(value: string | null): Promise<void> {
    const locale: string = value || LOCALE_EN;
    const language: string = locale.slice(0, 2);
    if (!language || LOCALE_LIST.indexOf(language) == -1) {
      return Promise.reject();
    }
    if (this.currLocale == locale) {
      Promise.resolve();
    }
    return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
      this.translate.use(language).pipe(first())
        .subscribe({
          next: () => {
            this.currLocale = locale;
            this.dateAdapter.setLocale(locale);
            HttpErrorUtil.setTranslate(this.translate);
            resolve();
          },
          error: (err) => reject(err) 
        });
      });
  }

}