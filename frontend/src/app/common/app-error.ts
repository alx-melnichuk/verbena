import { HttpErrorResponse } from '@angular/common/http';
import { TranslateService } from '@ngx-translate/core';

export interface AppError {
  errCode: string;
  errMsg: string;
  params: {
    [key: string]: string | number | null;
  };
}

export class AppErrorUtil {
  public static handleError(errRes: HttpErrorResponse, defaultValue: string, translate?: TranslateService): string[] {
    let result: string[] = [];
    if (!!errRes && !!errRes.error) {
      const errResList = !Array.isArray(errRes.error) ? [errRes.error] : errRes.error;
      for (let index = 0; index < errResList.length; index++) {
        const error = errResList[index];
        const errCode = error['errCode'] || '';
        if (!!errCode) {
          const errMsg = error['errMsg'] || '';
          const key = `${errCode}${!!errMsg ? '.' + errMsg : ''}`;
          const value = translate?.instant(key, error['params'] || {});
          result.push(value);
        }    
      }
    } else {
      result.push(defaultValue);
    }
    return result;
  }
}
