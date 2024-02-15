import { HttpErrorResponse } from '@angular/common/http';

export class HttpErrorUtil {
  public static getMsgs(error: HttpErrorResponse): string[] {
    let result: string[] = ['http_error.error_accessing_server_api'];
    if (!!error) {
      const errMessage = error.error?.message;
      if (!!errMessage) {
        result = (Array.isArray(errMessage) ? errMessage : [errMessage]);
      } else if (!!error.message){
        result = [error.message];
      }
    }
    return result;
  }
}
