import { Injectable } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';
import { first } from 'rxjs/operators';

@Injectable({
  providedIn: 'root',
})
export class InitTranslateService {
  constructor(private translate: TranslateService) {
    console.log(`InitTranslateService();`); // #
  }

  public init(): Promise<void | unknown> {
    // Download translations before starting the application.
    this.translate.addLangs(['en', 'de']);
    this.translate.setDefaultLang('en');

    const browserLang = this.translate.getBrowserLang() || '';
    const lang: string = browserLang.match(/en|de/) ? browserLang : 'en';

    return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
      this.translate
        .use(lang)
        .pipe(first())
        .subscribe({
          next: () => resolve(),
          error: (err) => reject(err),
        });
    });
  }

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
