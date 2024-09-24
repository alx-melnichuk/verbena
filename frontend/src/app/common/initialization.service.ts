import { Injectable, Renderer2 } from '@angular/core';
import { Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { DateAdapter } from '@angular/material/core';
import { first } from 'rxjs/operators';

import { ProfileService } from '../lib-profile/profile.service';
import { AuthorizationUtil } from '../utils/authorization.util';
import { HttpErrorUtil } from '../utils/http-error.util';

import { THEME_DARK, THEME_LIGHT, THEME_SUFFIX } from './constants';
import { ROUTE_LOGIN } from './routes';

export const LOCALE_EN = 'en';
export const LOCALE_DE = 'de';
export const LOCALE_UK = 'uk';
export const LANGUAGES = [LOCALE_EN, LOCALE_DE, LOCALE_UK];

@Injectable({
  providedIn: 'root',
})
export class InitializationService {
  private currTheme: string | null = null;
  private currLocale: string | null = null;

  constructor(
    private router: Router,
    private dateAdapter: DateAdapter<any>,
    private translate: TranslateService,
    private profileService: ProfileService,
  ) {
    console.log(`#2-InitializationService();`); // #
  }

  public initTranslate(): Promise<void | unknown> {
    // Download translations before starting the application.
    this.translate.addLangs(LANGUAGES);
    this.translate.setDefaultLang(LOCALE_EN);

    const languageValue = this.currLocale || this.getBrowserLanguage(LOCALE_EN).slice(0,2);
    const language = (LANGUAGES.indexOf(languageValue) > -1 ? languageValue : LOCALE_EN);
    
    return this.setLocale(language);
  }

  public async initSession(): Promise<void> {
    const isAuthorizationRequired = AuthorizationUtil.isAuthorizationRequired(window.location.pathname);
    const isAuthorizationDenied = AuthorizationUtil.isAuthorizationDenied(window.location.pathname);
    const isNotAuthorizationDenied = !isAuthorizationDenied;

    const isExistAccessToken = this.profileService.hasAccessTokenInLocalStorage();
    if (isAuthorizationRequired || (isNotAuthorizationDenied && isExistAccessToken)) {
      try {
        await this.profileService.getCurrentProfile();
        const language = this.profileService.profileDto?.locale;
        if (!!language && language != this.currLocale) {
          this.currLocale = language.slice(0,2);
          this.setLocale(this.currLocale);
        }
      } catch {
        this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
      }
    }
    return Promise.resolve();
  }

  // ** Theme **
  public getTheme(): string | null {
    return this.currTheme;
  }
  
  public setTheme(value: string | null | undefined, renderer: Renderer2): void {
    const theme = value || THEME_LIGHT;
    if ([THEME_DARK, THEME_LIGHT].indexOf(theme) > -1 && this.currTheme != theme) {
      const oldClassName = `${this.currTheme}-${THEME_SUFFIX}`;
      renderer.removeClass(document.body, oldClassName);
      this.currTheme = theme;
      const newClassName = `${theme}-${THEME_SUFFIX}`;
      renderer.addClass(document.body, newClassName);
    }
  }
  // ** Locale ** 
  public getLocale(): string | null {
    return this.currLocale;
  }
  
  public setLocale(value: string | null): Promise<void> {
    const language: string = value || LOCALE_EN;
    if (!language || LANGUAGES.indexOf(language) == -1) {
      return Promise.reject();
    }
    if (this.currLocale == language) {
      Promise.resolve();
    }

    return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
      this.translate.use(language).pipe(first())
        .subscribe({
          next: () => {
            this.currLocale = language;
            this.dateAdapter.setLocale(language);
            HttpErrorUtil.setTranslate(this.translate);
            resolve();
          },
          error: (err) => reject(err) 
        });
      });
    }

  // ** Private Api **

  private getBrowserLanguage(defaultValue: string): string {
    if (typeof window === 'undefined' || typeof window.navigator === 'undefined') {
      return defaultValue;
    }
    const wn = window.navigator as any;
    let lang = wn.languages ? wn.languages[0] : defaultValue;
    lang = lang || wn.language || wn.browserLanguage || wn.userLanguage;
    return lang;
  }
}
