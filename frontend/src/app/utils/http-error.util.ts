import { HttpErrorResponse } from '@angular/common/http';
import { TranslateService } from '@ngx-translate/core';

export class HttpErrorUtil {
  private static translate: TranslateService | undefined;
  public static setTranslate(translate: TranslateService | undefined): void {
    HttpErrorUtil.translate = translate;
  }
  public static getTranslate(): TranslateService | undefined {
    return HttpErrorUtil.translate;
  }
  public static getMsgs(errRes: HttpErrorResponse): string[] {
    let result: string[] = [];
    if (!!errRes && !!errRes.error) {
      const errResList = !Array.isArray(errRes.error) ? [errRes.error] : errRes.error;
      for (let index = 0; index < errResList.length; index++) {
        const appError = errResList[index];
        const code = appError['code'] || '';
        const message = appError['message'] || '';
        let value = `${code}${!!message ? ': ' + message : ''}`;
        if (!!code && !!HttpErrorUtil.translate) {
          const key = `${code}${!!message ? '.' + message : ''}`;
          const value2 = HttpErrorUtil.translate.instant(key, appError['params'] || {});
          if (value2 != key) {
            value = value2;
          }
        }
        if (!!value) {
          result.push(value);
        }
      }
    }
    if (result.length == 0 && !!HttpErrorUtil.translate) {
      result.push(HttpErrorUtil.translate.instant('error.server_api_call'));
    }
    return result;
  }
}
