import { Injectable, Renderer2 } from '@angular/core';
import { Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { first } from 'rxjs/operators';
import { ROUTE_LOGIN } from './routes';
import { AuthorizationUtil } from '../utils/authorization.util';
import { UserService } from '../entities/user/user.service';
import { DateAdapter } from '@angular/material/core';
import { HttpErrorUtil } from '../utils/http-error.util';

@Injectable({
  providedIn: 'root',
})
export class InitializationService {
  private isDarkTheme: boolean | undefined;

  constructor(
    private router: Router,
    private dateAdapter: DateAdapter<any>,
    private translate: TranslateService,
    private userService: UserService
  ) {
    console.log(`#2-InitializationService();`); // #
  }

  public initTranslate(): Promise<void | unknown> {
    const userLanguage = this.getUserLanguage('en');
    this.dateAdapter.setLocale(userLanguage);
    // Download translations before starting the application.
    this.translate.addLangs(['en', 'de']);
    this.translate.setDefaultLang('en');
    const browserLang = this.translate.getBrowserLang() || '';
    const lang: string = browserLang.match(/en|de/) ? browserLang : 'en';
    // return this.translate.use(lang).toPromise();
    return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
      this.translate
        .use(lang)
        .pipe(first())
        .subscribe({
          next: () => {
            HttpErrorUtil.setTranslate(this.translate)
            resolve()
          },
          error: (err) => reject(err) 
        });
    });
  }

  public async initSession(): Promise<void> {
    const isAuthorizationRequired = AuthorizationUtil.isAuthorizationRequired(window.location.pathname);
    const isAuthorizationDenied = AuthorizationUtil.isAuthorizationDenied(window.location.pathname);
    const isNotAuthorizationDenied = !isAuthorizationDenied;

    const isExistAccessToken = this.userService.hasAccessTokenInLocalStorage();
    if (isAuthorizationRequired || (isNotAuthorizationDenied && isExistAccessToken)) {
      try {
        await this.userService.getCurrentUser();
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
      this.isDarkTheme = value;
      const newClassName = this.getThemeName(this.isDarkTheme);
      const body: HTMLElement = document.body;

      renderer.removeClass(body, oldClassName);
      renderer.addClass(body, newClassName);
    }
  }

  // ** Private Api **

  private getUserLanguage(defaultValue: string): string {
    if (typeof window === 'undefined' || typeof window.navigator === 'undefined') {
      return defaultValue;
    }
    const wn = window.navigator as any;
    let lang = wn.languages ? wn.languages[0] : defaultValue;
    lang = lang || wn.language || wn.browserLanguage || wn.userLanguage;
    return lang;
  }

  private getThemeName(isDarkTheme: boolean): string {
    return !!isDarkTheme ? 'dark-theme' : 'light-theme';
  }

}
