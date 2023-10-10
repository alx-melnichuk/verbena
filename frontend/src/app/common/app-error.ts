import { HttpErrorResponse } from '@angular/common/http';

export interface AppError {
  errCode: string;
  errMsg: string;
  params: {
    [key: string]: string | number | null;
  };
}

export class AppErrorUtil {
  public static handleError(errRes: HttpErrorResponse, defaultValue: string): string[] {
    let result: string[] = [defaultValue];
    if (!!errRes && !!errRes.error && !!errRes.error['errCode'] && !!errRes.error['errMsg']) {
      let appError = errRes.error as AppError;
      result = appError.errMsg.split('\n');
    }
    return result;
  }
}
