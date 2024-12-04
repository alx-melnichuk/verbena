import { DOCUMENT } from '@angular/common';
import { Inject, Injectable, Renderer2 } from '@angular/core';
import { Router } from '@angular/router';
import { DateAdapter } from '@angular/material/core';
import { TranslateService } from '@ngx-translate/core';
import { first } from 'rxjs/operators';

import { ProfileService } from '../lib-profile/profile.service';
import { AuthorizationUtil } from '../utils/authorization.util';
import { HttpErrorUtil } from '../utils/http-error.util';

import { THEME_DARK, THEME_LIGHT, THEME_LIST, THEME_SUFFIX } from './constants';
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

  get theme(): string | null {
    return this.currTheme;
  }
  set theme(value: string | null) {
  }

  constructor(
    @Inject(DOCUMENT) private document: Document,
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

    const locale = this.currLocale || this.getBrowserLanguage(LOCALE_EN).slice(0, 2);
    const language = (LANGUAGES.indexOf(locale) > -1 ? locale : LOCALE_EN);
    
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
        const locale = this.profileService.profileDto?.locale;
        if (!!locale && this.currLocale != locale) {
          await this.setLocale(locale);
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
    const index = THEME_LIST.indexOf(value || '');
    const theme = THEME_LIST[index > -1 ? index : 0];
    if (this.currTheme != theme) {
      if (!!this.currTheme) {
        renderer.removeClass(this.document.documentElement, this.currTheme);
      }
      this.currTheme = theme;
      renderer.addClass(this.document.documentElement, theme);
    }
  }
  // ** Locale ** 
  public getLocale(): string | null {
    return this.currLocale;
  }
  
  public setLocale(value: string | null): Promise<void> {
    const locale: string = value || LOCALE_EN;
    const language: string = locale.slice(0, 2);
    if (!language || LANGUAGES.indexOf(language) == -1) {
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
