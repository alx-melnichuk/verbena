import { DOCUMENT } from '@angular/common';
import { Inject, Injectable, Renderer2 } from '@angular/core';
import { Router } from '@angular/router';
import { DateAdapter } from '@angular/material/core';
import { TranslateService } from '@ngx-translate/core';
import { first } from 'rxjs/operators';

import { ProfileService } from '../lib-profile/profile.service';
import { AuthorizationUtil } from '../utils/authorization.util';
import { HttpErrorUtil } from '../utils/http-error.util';

import { COLOR_SCHEME_LIST, LOCALE_EN, LOCALE_LIST, SCHEME_DARK, SCHEME_LIGHT } from './constants';
import { ROUTE_LOGIN } from './routes';

const COLOR_SCHEME = 'color-scheme';

@Injectable({
  providedIn: 'root',
})
export class InitializationService {
  private currColorScheme: string | null = null;
  private currLocale: string | null = null;

  get theme(): string | null {
    return this.currColorScheme;
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
    this.translate.addLangs(LOCALE_LIST);
    this.translate.setDefaultLang(LOCALE_EN);

    const locale = this.currLocale || this.getBrowserLanguage(LOCALE_EN).slice(0, 2);
    const language = (LOCALE_LIST.indexOf(locale) > -1 ? locale : LOCALE_EN);
    
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
  public getColorScheme(): string | null {
    return this.currColorScheme;
  }
  
  public setColorScheme(value: string | null | undefined, renderer: Renderer2): void {
    const index = COLOR_SCHEME_LIST.indexOf(value || '');
    const theme = COLOR_SCHEME_LIST[index > -1 ? index : 0];
    if (this.currColorScheme != theme) {
      if (!!this.currColorScheme) {
        this.document.documentElement.style.setProperty(COLOR_SCHEME, null);
        this.document.documentElement.style.setProperty('--' + COLOR_SCHEME, null);
        renderer.removeClass(this.document.documentElement, this.currColorScheme);
      }
      this.currColorScheme = theme;
      renderer.addClass(this.document.documentElement, theme);
      const scheme = this.currColorScheme.split('-')[0];
      if ([SCHEME_LIGHT, SCHEME_DARK].includes(scheme)) {
        this.document.documentElement.style.setProperty(COLOR_SCHEME, scheme);
        this.document.documentElement.style.setProperty('--' + COLOR_SCHEME, scheme);
      }
    }
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
