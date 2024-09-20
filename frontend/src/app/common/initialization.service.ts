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
  private isDarkTheme: boolean | undefined;
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
    let language = this.currLocale || this.getBrowserLanguage(LOCALE_EN).slice(0,2);
    language = (LANGUAGES.indexOf(language) > -1 ? language : LOCALE_EN);
    // Download translations before starting the application.
    this.translate.addLangs(LANGUAGES);
    this.translate.setDefaultLang(LOCALE_EN);
    
    this.dateAdapter.setLocale(language);
    /*
    const userLanguage = this.getBrowserLanguage('en');
    this.dateAdapter.setLocale(userLanguage);
    // Download translations before starting the application.
    this.translate.addLangs(['en', 'de']);
    this.translate.setDefaultLang('en');
    const browserLang = this.translate.getBrowserLang() || '';
    const lang: string = browserLang.match(/en|de/) ? browserLang : 'en';
    // return this.translate.use(lang).toPromise();
    */
    return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
      this.translate.use(language).pipe(first())
        .subscribe({
          next: () => {
            HttpErrorUtil.setTranslate(this.translate);
            resolve();
          },
          error: (err) => reject(err) 
        });
    });
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

  public getDarkTheme(): boolean {
    return !!this.isDarkTheme;
  }
  
  public setDarkTheme(value: boolean, renderer: Renderer2): void {
    if (this.isDarkTheme !== value) {
      const oldClassName = this.getThemeName(!!this.isDarkTheme);
      this.isDarkTheme = !!value;
      const newClassName = this.getThemeName(this.isDarkTheme);

      renderer.removeClass(document.body, oldClassName);
      renderer.addClass(document.body, newClassName);
    }
  }

  public getLocale(): string | null {
    return this.currLocale;
  }

  public setLocale(language: string | null): void {
    if (!!language && LANGUAGES.indexOf(language) > -1) {
      this.currLocale = language;
      this.dateAdapter.setLocale(language);
      this.translate.use(language).pipe(first())
      .subscribe({
        next: () => {
          HttpErrorUtil.setTranslate(this.translate);
        },
      });
    }
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

  private getThemeName(isDarkTheme: boolean): string {
    return !!isDarkTheme ? `${THEME_DARK}-${THEME_SUFFIX}` : `${THEME_LIGHT}-${THEME_SUFFIX}`;
  }

}
