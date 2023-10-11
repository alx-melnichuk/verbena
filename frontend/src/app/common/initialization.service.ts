import { Injectable } from '@angular/core';
import { Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { first } from 'rxjs/operators';
import { ROUTE_LOGIN } from './routes';
import { Authorization } from './authorization';
import { UserService } from '../entities/user/user.service';
import { UriConfig } from '../utils/uri-config';
import { DateAdapter } from '@angular/material/core';

@Injectable({
  providedIn: 'root',
})
export class InitializationService {
  constructor(
    private router: Router,
    private dateAdapter: DateAdapter<any>,
    private translate: TranslateService,
    private userService: UserService
  ) {
    console.log(`#2-InitializationService();`); // #
    UriConfig.initial();
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
        .subscribe({ next: () => resolve(), error: (err) => reject(err) });
    });
  }

  public async initSession(): Promise<void> {
    const isAuthorizationRequired = Authorization.isAuthorizationRequired();
    const isNotRequiredNotDenied = Authorization.isAuthorizationNotRequiredNotDenied();
    const isExistAccessToken = this.userService.hasAccessTokenInLocalStorage();
    if (isAuthorizationRequired || (isNotRequiredNotDenied && isExistAccessToken)) {
      try {
        await this.userService.getCurrentUser();
      } catch {
        this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
      }
    }
    return Promise.resolve();
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
}
